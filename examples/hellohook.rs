use redismodule::define_module;
use redismodule_macros::rcall;

use redismodule::{raw, Context, Error, RResult, RStr};

#[rcall]
fn init(_ctx: &mut Context, _args: Vec<RStr>) -> Result<(), Error> {
    Ok(())
}

define_module! {
    name: "hellohook",
    version: 1,
    data_types: [],
    init_funcs: [
        init_c,
    ],
    commands: []
}
