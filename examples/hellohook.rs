use redismodule::{define_module};
use redismodule_macros::{rcall};

use redismodule::{ Context, RStr, RResult, Error, raw };

#[rcall]
fn init(ctx: &mut Context, mut args: Vec<RStr>) -> Result<(), Error> {
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
