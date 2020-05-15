use iredismodule::prelude::*;
use iredismodule_macros::rcmd;

#[rcmd("simple.hello", "readonly", 0, 0, 0)] 
fn simple_hello(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let db = ctx.get_select_db();
    Ok(db.into())
}

// Register module
define_module! {
    name: "simple",
    version: 1,
    data_types: [],
    init_funcs: [],
    commands: [
        simple_hello_cmd,
    ]
}
