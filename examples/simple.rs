use redismodule::{redis_module2};
use redismodule_macros::{cmd};

use redismodule::{
    parse_args, raw, Context, Error, ListPosition, RResult, StatusCode, RStr, Value, KeyType, ArgvFlags,
    HashGetFlag, HashSetFlag, ZsetRangeDirection,
};

#[cmd(name="hello.simple",flags="readonly",first_key=0,last_key=0,key_step=0)]
pub fn hello_simple(ctx: &Context, _args: Vec<RStr>) -> RResult {
    let db = ctx.get_select_db();
    Ok(db.into())
}

redis_module2! {
    name: "simple",
    version: 1,
    data_types: [],
    init_funcs: [],
    commands: [
        create_hello_simple,
    ]
}
