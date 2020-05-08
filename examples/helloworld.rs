use redis_module::{redis_command, redis_module};

use redis_module::{
    parse_args, raw, Ctx, Error, ListPosition, RedisResult, RedisString, RedisValue, StatusCode,
};
use std::os::raw::c_int;

fn hello_simple(ctx: &Ctx, args: Vec<String>) -> RedisResult {
    let db = ctx.get_select_db();
    Ok(db.into())
}

fn hello_push_native(ctx: &Ctx, args: Vec<String>) -> RedisResult {
    if args.len() != 3 {
        return Err(Error::WrongArity);
    }
    let mut key = ctx.open_write_key(&args[1]);
    key.list_push(ListPosition::Tail, &args[2])?;
    let len = key.value_length();
    Ok(len.into())
}

fn hello_push_call(ctx: &Ctx, args: Vec<String>) -> RedisResult {
    if args.len() != 3 {
        return Err(Error::WrongArity);
    }
    ctx.call("RPUSH", &args[1..])
}

pub extern "C" fn init(
    ctx: *mut raw::RedisModuleCtx,
    argv: *mut *mut raw::RedisModuleString,
    argc: c_int,
) -> c_int {
    let args = parse_args(argv, argc).unwrap();
    let ctx_ = Ctx::new(ctx);
    ctx_.log_debug(&format!(
        "Module loaded with ARGV[{}] = {:?}\n",
        args.len(),
        args
    ));
    StatusCode::Ok as c_int
}

redis_module! {
    name: "simple",
    version: 1,
    data_types: [],
    init: init,
    commands: [
        ["hello.simple", hello_simple, "readonly" , 0, 0, 0],
        ["hello.push.native", hello_push_native, "write deny-oom", 1, 1, 1],
        ["hello.push.call", hello_push_call, "write deny-oom", 1, 1, 1],
        // ["hello.push.call2", hello_push_call2, "", 1, 1, 1],
        // ["hello.list.sum.len", hello_list_sum_len, "", 1, 1, 1],
        // ["hello.list.splice", hello_list_splice, "", 1, 2, 1],
        // ["hello.list.splice.auto", hello_list_splice_auto, "", 1, 2, 1],
        // ["hello.rand.array", hello_list_rand_array, "", 0, 0, 0],
        // ["hello.repl1", hello_list_repl1, "", 0, 0, 0],
        // ["hello.repl2", hello_list_repl2, "", 1, 1, 1],
        // ["hello.toggle.case", hello_toggle_case, "", 1, 1, 1],
        // ["hello.more.expire", hello_more_expire, "", 1, 1, 1],
        // ["hello.zsumrange", hello_zsumrange, "", 1, 1, 1],
        // ["hello.lexrange", hello_lexrange, "", 1, 1, 1],
        // ["hello.hcopy", hello_hcopy, "", 1, 1, 1],
        // ["hello.leftpad", hello_leftpad, "", 1, 1, 1],
    ],
}
