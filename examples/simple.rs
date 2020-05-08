use redis_module::{redis_command, redis_module};
use redis_module::{Context, Error, RedisResult, RedisStr};

fn simple_mul(_ctx: &Context, args: Vec<RedisStr>) -> RedisResult {
    if args.len() < 2 {
        return Err(Error::WrongArity);
    }

    let nums = args
        .into_iter()
        .skip(1)
        .map(|s| s.get_longlong())
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
