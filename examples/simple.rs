use redismodule::{define_module};
use redismodule_macros::{rcommand};

use redismodule::{ Context, RStr, RResult };

#[rcommand(name="hello.simple",flags="readonly",first_key=0,last_key=0,key_step=0)]
pub fn hello_simple(ctx: &Context, _args: Vec<RStr>) -> RResult {
    let db = ctx.get_select_db();
    Ok(db.into())
}

define_module! {
    name: "simple",
    version: 1,
    data_types: [],
    init_funcs: [],
    commands: [
        create_hello_simple,
    ]
}
