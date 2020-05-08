use redis_module::{redis_module, redis_command};

use redis_module::{Ctx, Error, RedisResult};

fn simple_mul(ctx: &Ctx, args: Vec<String>) -> RedisResult {
    if args.len() < 2 {
        return Err(Error::WrongArity);
    }

    let nums = args
        .into_iter()
        .skip(1)
        .map(|s| s.parse::<i64>().map_err(|e| e.into()))
        .collect::<Result<Vec<i64>, Error>>()?;

    let product = nums.iter().product();

    let mut response = Vec::from(nums);
    response.push(product);

    return Ok(response.into());
}

redis_module! {
    name: "simple",
    version: 1,
    data_types: [],
    commands: [
        ["simple.mul", simple_mul, "", 0, 0, 0],
    ],
}
