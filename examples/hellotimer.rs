use rand::random;
use redismodule::define_module;
use redismodule::{Context, RResult, RStr};
use redismodule_macros::rcmd;
use std::time::Duration;

fn timer_handler(ctx: &Context, data: String) {
    ctx.log_debug(&data);
}

#[rcmd(
    name = "hellotimer.timer",
    flags = "readonly",
    first_key = 0,
    last_key = 0,
    key_step = 0
)]
fn hello_timer(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    for _ in 0..10 {
        let delay: u32 = random::<u32>() % 5000;
        ctx.create_timer(
            Duration::from_millis(delay as u64),
            timer_handler,
            format!("After {}", delay),
        )?;
    }
    Ok("OK".into())
}

define_module! {
    name: "hellotimer",
    version: 1,
    data_types: [],
    init_funcs: [],
    commands: [
        hello_timer_cmd,
    ],
}
