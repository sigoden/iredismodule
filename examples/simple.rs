use redismodule::define_module;
use redismodule_macros::{rcall, rcmd};

use redismodule::{Context, RResult, RStr};

#[rcmd(
    name = "hello.simple",
    flags = "readonly",
    first_key = 0,
    last_key = 0,
    key_step = 0
)]
pub fn hello_simple(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let db = ctx.get_select_db();
    Ok(db.into())
}

#[rcall]
pub fn foo(_ctx: &mut Context, _args: Vec<RStr>) -> redismodule::RResult {
    Ok(().into())
}

define_module! {
    name: "simple",
    version: 1,
    data_types: [],
    init_funcs: [],
    commands: [
        hello_simple_cmd,
    ]
}
