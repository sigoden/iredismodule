use redis_module::{redis_command, redis_module};
use redis_module::{Context, RedisResult, RedisStr};
use rand::random;
use std::time::Duration;

fn timer_handler(ctx: &Context, data: String) {
    ctx.log_debug(&data);
}

fn hello_timer(ctx: &Context, args: Vec<RedisStr>) -> RedisResult {
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

