use redismodule::{define_module, assert_len};
use redismodule_macros::{rcmd, rwrap};
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::os::raw::c_void;
use std::thread;
use std::time::Duration;

use redismodule::prelude::*;
use redismodule::user::User;
use redismodule::block_client::BlockClient;

const TIMEOUT_TIME: Duration = Duration::from_millis(1000);

lazy_static! {
    static ref GLOBAL_USER: Mutex<Option<User>> = Mutex::new(None);
    static ref GLOBAL_ID: Mutex<u64> = Mutex::new(0);
}

#[rcmd("helloacl.reset")]
fn helloacl_reset(_ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let mut user_ = GLOBAL_USER.lock().unwrap();
    let user_ = &mut (*user_);
    let user = create_user()?;
    std::mem::replace(user_, Some(user));
    Ok("OK".into())
}

#[rcmd("helloacl.revoke")]
fn helloacl_revoke(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let id = GLOBAL_ID.lock().unwrap();
    if *id == 0 {
        return Err(Error::new("Global user currently not used"))
    }
    ctx.deauthenticate_and_close_client(*id);
    Ok("Ok".into())
}

extern "C" fn helloac_user_changed(_client_id: u64, _privdata: *mut c_void) {
    update_global_auth_client_id(0);
}

#[rcmd("helloacl.authglobal", "no-auth")]
fn helloacl_authglobal(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let id = GLOBAL_ID.lock().unwrap();
    if *id == 0 {
        return Err(Error::new("Global user currently not used"))
    }
    let user_ = GLOBAL_USER.lock().unwrap();
    if let Some(ref user) = *user_ {
        let new_id = ctx.authenticate_client_with_user::<i32>(user, Some(helloac_user_changed), None, *id)?;
        update_global_auth_client_id(new_id);
    } 
    
    Ok("Ok".into())
}

#[rwrap("call")]
fn helloacl_reply(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let name = ctx.get_block_client_private_data::<String>();
    ctx.authenticate_client_with_acl_user::<i32>(name, None, None, 0)?;
    Ok("Ok".into())
}

#[rwrap("call")]
fn helloacl_timeout(_ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    Ok("Request timeout".into())
}

#[rwrap("free")]
fn helloacl_free(_: &mut Context, _: Box<i32>) {}


fn helloacl_thread_main(bc: BlockClient, user: String) {
    bc.unblock(Some(user)).unwrap()
}

#[rcmd("helloacl.authasync", "no-auth")]
fn helloacl_authasync(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 2);
    let bc = ctx.block_client(Some(helloacl_reply_c), Some(helloacl_timeout_c), Some(helloacl_free_c), TIMEOUT_TIME);

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
    let user= create_user()?;
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

