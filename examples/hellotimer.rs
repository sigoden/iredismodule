use redismodule::{redis_module};
use redismodule::{Context, RResult, RStr};
use rand::random;
use std::time::Duration;

fn timer_handler(ctx: &Context, data: String) {
    ctx.log_debug(&data);
}

fn hello_timer(ctx: &Context, args: Vec<RStr>) -> RResult {
    for _ in 0..10 {
        let delay: u32 = random::<u32>() % 5000;
        ctx.create_timer(Duration::from_millis(delay as u64), timer_handler, format!("After {}", delay))?;
    }
    Ok("OK".into())
}

redis_module! {
    name: "hellotimer",
    version: 1,
    data_types: [],
    commands: [
        ["hellotimer.timer", hello_timer, "readonly", 0, 0, 0],
    ],
}

