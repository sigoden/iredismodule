use redismodule::define_module;
use redismodule_macros::{rcmd, rwrap};
use lazy_static::lazy_static;
use std::sync::Mutex;

use redismodule::{Context, RResult, Error, RStr, User};

lazy_static! {
    static ref GLOBAL_USER: Mutex<Option<User>> = Mutex::new(None);
    static ref GLOBAL_ID: Mutex<u64> = Mutex::new(0);
}

#[rcmd("helloacl.reset")]
fn helloacl_reset(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let id = GLOBAL_ID.lock().unwrap();
    let id = *id;
    if id.eq(&0) {
        return Err(Error::generic("Global user currently not used"))
    }

    ctx.deauthenticate_and_close_client(id);
    Ok("Ok".into())
}

#[rcmd("helloacl.revoke")]
fn helloacl_revoke(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    unimplemented!()
}

#[rcmd("helloacl.authglobal", "no-auth")]
fn helloacl_authglobal(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    unimplemented!()
}

#[rcmd("helloacl.authasync", "no-auth")]
fn helloacl_authasync(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    unimplemented!()
}

#[rwrap("call")]
fn init(_ctx: &mut Context, _args: Vec<RStr>) -> Result<(), Error> {
    let mut user = User::new("global");
    user.set_acl("allcommands")?;
    user.set_acl("allkeys")?;
    user.set_acl("on")?;
    *GLOBAL_ID.get_mut().unwrap() = 0;
    *GLOBAL_USER.get_mut().unwrap() = Some(user);
    Ok(())
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

