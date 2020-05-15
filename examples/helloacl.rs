use iredismodule_macros::{rcmd, rwrap};
use lazy_static::lazy_static;
use std::os::raw::c_void;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use iredismodule::block_client::BlockClient;
use iredismodule::prelude::*;
use iredismodule::user::User;

const TIMEOUT_TIME: Duration = Duration::from_millis(1000);

lazy_static! {
    static ref GLOBAL_USER: Mutex<Option<User>> = Mutex::new(None);
    static ref GLOBAL_ID: Mutex<u64> = Mutex::new(0);
}
/// HELLOACL.RESET 
/// Synchronously delete and re-create a module user.
#[rcmd("helloacl.reset")]
fn helloacl_reset(_ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let mut user_ = GLOBAL_USER.lock().unwrap();
    let user_ = &mut (*user_);
    let user = create_user()?;
    std::mem::replace(user_, Some(user));
    Ok("OK".into())
}

/// HELLOACL.REVOKE 
/// Synchronously revoke access from a user.
#[rcmd("helloacl.revoke")]
fn helloacl_revoke(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let id = GLOBAL_ID.lock().unwrap();
    if *id == 0 {
        return Err(Error::new("Global user currently not used"));
    }
    ctx.deauthenticate_and_close_client(*id);
    Ok("Ok".into())
}
/// Callback handler for user changes, use this to notify a module of 
/// changes to users authenticated by the module
extern "C" fn helloac_user_changed(_client_id: u64, _privdata: *mut c_void) {
    update_global_auth_client_id(0);
}

/// HELLOACL.AUTHGLOBAL 
/// Synchronously assigns a module user to the current context. 
#[rcmd("helloacl.authglobal", "no-auth")]
fn helloacl_authglobal(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let id = GLOBAL_ID.lock().unwrap();
    if *id == 0 {
        return Err(Error::new("Global user currently not used"));
    }
    let user_ = GLOBAL_USER.lock().unwrap();
    if let Some(ref user) = *user_ {
        let new_id =
            ctx.authenticate_client_with_user::<i32>(user, Some(helloac_user_changed), None)?;
        update_global_auth_client_id(new_id);
    }

    Ok("Ok".into())
}
/// Reply callback for auth command HELLOACL.AUTHASYNC 
#[rwrap("call")]
fn helloacl_reply(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let name = ctx.get_block_client_private_data::<String>();
    ctx.authenticate_client_with_acl_user::<i32>(name, None, None)?;
    Ok("Ok".into())
}

/// Timeout callback for auth command HELLOACL.AUTHASYNC 
#[rwrap("call")]
fn helloacl_timeout(_ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    Ok("Request timeout".into())
}
/// Private data frees data for HELLOACL.AUTHASYNC command.
#[rwrap("free")]
fn helloacl_free(_: &mut Context, _: Box<i32>) {}

/// Background authentication can happen here.
fn helloacl_thread_main(bc: BlockClient, user: String) {
    bc.unblock(Some(user)).unwrap()
}

/// HELLOACL.AUTHASYNC 
/// Asynchronously assigns an ACL user to the current context.
#[rcmd("helloacl.authasync", "no-auth")]
fn helloacl_authasync(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 2);
    let bc = ctx
        .block_client(
            Some(helloacl_reply_c),
            Some(helloacl_timeout_c),
            Some(helloacl_free_c),
            TIMEOUT_TIME,
        )
        .unwrap();

    let name = args[1].to_str()?.to_owned();
    if thread::Builder::new()
        .spawn(move || helloacl_thread_main(bc, name))
        .is_err()
    {
        bc.abort()?;
        return Ok("-ERR can't start thread".into());
    }
    Ok(Value::NoReply)
}

#[rwrap("call")]
fn init(_ctx: &mut Context, _args: Vec<RStr>) -> Result<(), Error> {
    update_global_auth_client_id(0);
    let user = create_user()?;
    let mut user_ = GLOBAL_USER.lock().unwrap();
    *user_ = Some(user);
    Ok(())
}

fn create_user() -> Result<User, Error> {
    let mut user = User::new("global");
    user.set_acl("allcommands")?;
    user.set_acl("allkeys")?;
    user.set_acl("on")?;
    Ok(user)
}

fn update_global_auth_client_id(id: u64) -> u64 {
    let mut id_ = GLOBAL_ID.lock().unwrap();
    let old_id = *id_;
    *id_ = id;
    old_id
}

define_module! {
    name: "helloacl",
    version: 1,
    data_types: [],
    init_funcs: [
        init_c,
    ],
    commands: [
        helloacl_reset_cmd,
        helloacl_revoke_cmd,
        helloacl_authglobal_cmd,
        helloacl_authasync_cmd,
    ]
}
