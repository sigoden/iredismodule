use redismodule::{define_module};
use redismodule_macros::{rcmd, rcall, rfree};

use redismodule::{ Context, RStr, RResult, assert_len, Error, BlockClient, Ptr, LogLevel, raw, ArgvFlags, Value};
use std::time::Duration;
use std::thread;
use rand::random;

#[rcall]
fn helloblock_reply(ctx: &mut Context, _: Vec<RStr>) -> RResult {
    let myint: &mut i32 = ctx.get_block_client_private_data();
    Ok((*myint as i64).into())
}

#[rcall]
fn helloblock_timeout(ctx: &mut Context, _: Vec<RStr>) -> RResult {
    Ok("Request timeout".into())
}

#[rfree]
fn helloblock_free(_: &mut Context, _: Box<i32>) {}

extern "C" fn helloblock_disconnected(ctx: *mut raw::RedisModuleCtx, bc: *mut raw::RedisModuleBlockedClient) {
    let context = Context::from_ptr(ctx);
    context.log(LogLevel::Warning, &format!("Block client {:p} disconnected!", bc))
}

fn helloblock_thread_main(bc: BlockClient, delay: u64) {
    thread::sleep(Duration::from_secs(delay));
    let r: i32 = random();
    bc.unblock_data(r).unwrap();
}

#[rcmd(name="hello.block",flags="",first_key=0,last_key=0,key_step=0)]
fn helloblock_rediscommand(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 3);
    let delay = args[1].get_long_long().map_err(|_| Error::generic("invalid delay"))? as u64;
    let timeout = args[2].get_long_long().map_err(|_| Error::generic("invalid timeout"))? as u64;
    let bc = ctx.block_client(
        Some(helloblock_reply_c),
        Some(helloblock_timeout_c), 
        Some(helloblock_free_c), 
        Duration::from_secs(timeout)
    );
    bc.set_disconnect_callback(helloblock_disconnected);
    if thread::Builder::new().spawn(move || {
        helloblock_thread_main(bc, delay)
    }).is_err() {
        bc.abort()?;
        return Ok("-ERR can't start thread".into());
    }
    Ok(Value::NoReply)
}

fn hellokeys_thread_main(bc: BlockClient) {
    let mut ctx = bc.get_threadsafe_context();
    let mut cursor = 0;
    let mut reply_data: Vec<Value> = vec![];
    while cursor != 0 {
        ctx.lock();
        let reply = ctx.call_str("SCAN", ArgvFlags::new(), &[cursor.to_string()]);
        ctx.unlock();
        let cr_cursor = reply.get_array_element(0);
        let cr_keys = reply.get_array_element(1);
        cursor = cr_cursor.get_integer();
        let items = reply.get_length();
        for i in 0..items {
            reply_data.push(RResult::from(cr_keys.get_array_element(i).into()).unwrap())
        }
    }
    ctx.reply(Ok(Value::Array(reply_data)));
    bc.unblock().unwrap();
}

#[rcmd(name="hello.keys",flags="",first_key=0,last_key=0,key_step=0)]
fn hellokeys_rediscommand(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 1);

    let bc = ctx.block_client(
        None,
        None,
        None,
        Duration::from_millis(0)
    );

    if thread::Builder::new().spawn(move || {
        hellokeys_thread_main(bc)
    }).is_err() {
        bc.abort()?;
        return Ok("-ERR Can't start thread".into());
    }
    Ok(Value::NoReply)
}


define_module! {
    name: "helloblock",
    version: 1,
    data_types: [],
    init_funcs: [],
    commands: [
        helloblock_rediscommand_cmd,
        hellokeys_rediscommand_cmd,
    ]
}

