use redismodule::define_module;
use redismodule_macros::{rcmd};

use redismodule::{Context, RResult, RStr};

#[rcmd("hello.simple", "readonly", 0, 0, 0)]
fn hello_simple(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let db = ctx.get_select_db();
    Ok(db.into())
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
