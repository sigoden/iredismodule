use iredismodule::prelude::*;
use iredismodule_macros::rcmd;
use iredismodule::call_reply::ReplyType;

macro_rules! check {
    ($cond:expr) => {
        if $cond { } else { return Err(Error::new(format!("failed at line {}", line!()))) }
    };
    ($cond:expr, $desc:expr) => {
        if $cond { } else { return Err(Error::new(format!("{} at line {}", $desc, line!()))) }
    };
}

#[rcmd("test.example_simple")]
fn test_example_simple(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    ctx.call("hello.simple", None, &[])?;
    Ok("OK".into())
}

#[rcmd("test.example_helloworld")]
fn test_example_helloworld(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let reply = ctx.call("hello.simple", None, &[])?;
    check!(reply.get_type() == ReplyType::Integer);
    let reply = ctx.call("hello.push.native", None, &["test:helloword:key1", "1"])?;
    check!(reply.get_type() == ReplyType::Integer);
    let reply = ctx.call("hello.push.call", None, &["test:helloword:key1", "2"])?;
    check!(reply.get_type() == ReplyType::Integer);
    let reply = ctx.call("hello.push.call2", None, &["test:helloword:key1", "3"])?;
    check!(reply.get_type() == ReplyType::Integer);
    let reply = ctx.call("hello.push.sum.len", None, &["test:helloword:key1"])?;
    check!(reply.get_type() == ReplyType::Integer);
    let reply = ctx.call("hello.list.splice", None, &["test:helloword:key1", "test:helloword:key2", "2"])?;
    check!(reply.get_type() == ReplyType::Integer);
    let reply = ctx.call("hello.rand.array", None, &["5"])?;
    check!(reply.get_type() == ReplyType::Array);
    let reply = ctx.call("hello.repl1", None, &[])?;
    check!(reply.get_type() == ReplyType::Integer);
    let reply = ctx.call("hello.repl2", None, &["test:helloword:key2"])?;
    check!(reply.get_type() == ReplyType::Integer);
    ctx.call("set", None, &["test:helloworld:key3", "abc"])?;
    let reply = ctx.call("hello.toggle.case", None, &["test:helloworld:key3"])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("hello.more.expire", None, &["test:helloworld:key3", "10000"])?;
    check!(reply.get_type() == ReplyType::String);
    ctx.call("zadd", None, &[
        "test:helloworld:key4", "1", "a", "2", "b", "3", "c", "4", "d"
    ])?;
    let reply = ctx.call("hello.zsumrange", None, &["test:helloworld:key4", "1", "4"])?;
    check!(reply.get_type() == ReplyType::Array);
    let reply = ctx.call("hello.lexrange", None, &["test:helloworld:key4", "-", "[c"])?;
    check!(reply.get_type() == ReplyType::Array);
    ctx.call("hset", None, &["test:helloworld:key5", "field1", "abc"])?;
    let reply = ctx.call("hello.hcopy", None, &["test:helloworld:key5", "field1", "field2"])?;
    check!(reply.get_type() == ReplyType::Integer);
    let reply = ctx.call("hello.leftpad", None, &["123", "8", "0"])?;
    check!(reply.get_type() == ReplyType::String);
    Ok("OK".into())
}


#[rcmd("test.example_hellotype")]
fn test_example_hellotype(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let reply = ctx.call("hellotype.insert", None, &["test:hellotype:key1", "1"])?;
    check!(reply.get_type() == ReplyType::Integer);
    ctx.call("hellotype.insert", None, &["test:hellotype:key1", "2"])?;
    ctx.call("hellotype.insert", None, &["test:hellotype:key1", "3"])?;
    ctx.call("hellotype.insert", None, &["test:hellotype:key1", "4"])?;
    let reply = ctx.call("hellotype.range", None, &["test:hellotype:key1", "1", "2"])?;
    check!(reply.get_type() == ReplyType::Array);
    let reply = ctx.call("hellotype.len", None, &["test:hellotype:key1"])?;
    check!(reply.get_type() == ReplyType::Integer);
    ctx.call("hellotype.brange", None, &["test:hellotype:key1", "1", "2", "5"])?;
    Ok("OK".into())
}

#[rcmd("test.example_hellotimer")]
fn test_example_hellotimer(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let reply = ctx.call("hellotimer.timer", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    Ok("OK".into())
}

#[rcmd("test.example_helloblock")]
fn test_example_helloblock(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    ctx.call("hello.block", None, &["1", "2"])?;
    ctx.call("hello.block", None, &["2", "1"])?;
    ctx.call("hello.keys", None, &["2", "1"])?;
    Ok("OK".into())
}

#[rcmd("test.testbase")]
fn test_testbase(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let reply = ctx.call("test.misc", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("test.key", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("test.call_reply", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("test.value", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    Ok("OK".into())
}

#[rcmd("test.testtype")]
fn test_testtype(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let reply = ctx.call("test.type", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    Ok("OK".into())
}

#[rcmd("test.all")]
fn test_all(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let reply = ctx.call("test.testbase", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("test.testtype", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("test.example_simple", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("test.example_helloworld", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("test.example_hellotype", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("test.example_hellotimer", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("test.example_helloblock", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    Ok("OK".into())
}

define_module! {
    name: "testall",
    version: 1,
    data_types: [],
    init_funcs: [],
    commands: [
        test_example_simple_cmd,
        test_example_helloworld_cmd,
        test_example_hellotype_cmd,
        test_example_hellotimer_cmd,
        test_example_helloblock_cmd,
        test_testbase_cmd,
        test_testtype_cmd,
        test_all_cmd, 
    ]
}

