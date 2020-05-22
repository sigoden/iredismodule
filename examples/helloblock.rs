use iredismodule::block_client::BlockClient;
use iredismodule::prelude::*;
use iredismodule::raw;
use iredismodule_macros::{rcmd, rwrap};

use rand::random;
use std::thread;
use std::time::Duration;

/// Reply callback for blocking command HELLO.BLOCK
#[rwrap("call")]
fn helloblock_reply(ctx: &mut Context, _: Vec<RStr>) -> RResult {
    let myint: &mut i32 = ctx.get_block_client_private_data();
    Ok((*myint as i64).into())
}

/// Timeout callback for blocking command HELLO.BLOCK
#[rwrap("call")]
fn helloblock_timeout(_ctx: &mut Context, _: Vec<RStr>) -> RResult {
    Ok("Request timeout".into())
}

/// Private data freeing callback for HELLO.BLOCK command.
#[rwrap("free")]
fn helloblock_free(_: &mut Context, _: Box<i32>) {}

/// An example blocked client disconnection callback.
///
/// Note that in the case of the HELLO.BLOCK command, the blocked client is now
/// owned by the thread calling sleep(). In this specific case, there is not
/// much we can do, however normally we could instead implement a way to
/// signal the thread that the client disconnected, and sleep the specified
/// amount of seconds with a while loop calling sleep(1), so that once we
/// detect the client disconnection, we can terminate the thread ASAP.
extern "C" fn helloblock_disconnected(
    ctx: *mut raw::RedisModuleCtx,
    bc: *mut raw::RedisModuleBlockedClient,
) {
    let context = Context::from_ptr(ctx);
    context.warning(format!("Block client {:p} disconnected!", bc));
}

/// The thread entry point that actually executes the blocking part
/// of the command HELLO.BLOCK.
fn helloblock_thread_main(bc: BlockClient, delay: u64) {
    thread::sleep(Duration::from_secs(delay));
    let r: i32 = random();

    // Here you should cleanup your state / threads, and if possible
    // call unblock, or notify the thread that will
    // call the function ASAP.
    bc.unblock(Some(r)).unwrap();
}

/// HELLO.BLOCK <delay> <timeout> -- Block for <count> seconds, then reply with
/// a random number. Timeout is the command timeout, so that you can test
/// what happens when the delay is greater than the timeout.
#[rcmd("hello.block")]
fn helloblock_rediscommand(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    if args.len() != 3 {
        return Err(Error::WrongArity);
    }
    let delay = args[1]
        .get_integer()
        .map_err(|_| Error::new("ERR invalid delay"))? as u64;
    let timeout = args[2]
        .get_integer()
        .map_err(|_| Error::new("ERR invalid timeout"))? as u64;
    let bc = ctx
        .block_client(
            Some(helloblock_reply_c),
            Some(helloblock_timeout_c),
            Some(helloblock_free_c),
            Duration::from_secs(timeout),
        )
        .unwrap();
    // Here we set a disconnection handler, however since this module will
    // block in sleep() in a thread, there is not much we can do in the
    // callback, so this is just to show you the API.
    bc.set_disconnect_callback(helloblock_disconnected);
    if thread::Builder::new()
        .spawn(move || helloblock_thread_main(bc, delay))
        .is_err()
    {
        bc.abort()?;
        return Ok("-ERR can't start thread".into());
    }
    Ok(Value::NoReply)
}

/// The thread entry point that actually executes the blocking part
/// of the command HELLO.KEYS.
///
/// Note: this implementation is very simple on purpose, so no duplicated
/// keys (returned by SCAN) are filtered. However adding such a functionality
/// would be trivial just using any data structure implementing a dictionary
/// in order to filter the duplicated items.
fn hellokeys_thread_main(bc: BlockClient) {
    let context = bc.get_threadsafe_context();
    let mut cursor = 0;
    let mut reply_data: Vec<Value> = vec![];
    loop {
        let reply = {
            let ctx = context.get_ctx().lock().unwrap();
            ctx.call("SCAN", None, &[&cursor.to_string()]).unwrap()
        };
        let cr_cursor = reply.get_array_element(0).unwrap();
        let cr_keys = reply.get_array_element(1).unwrap();
        cursor = cr_cursor.get_string().unwrap().parse::<i32>().unwrap();
        let items = cr_keys.get_length();
        for i in 0..items {
            reply_data.push(RResult::from(cr_keys.get_array_element(i).unwrap().into()).unwrap())
        }
        if cursor == 0 {
            break;
        }
    }
    context.reply(Ok(Value::Array(reply_data)));
    bc.unblock::<i32>(None).unwrap();
}

/// HELLO.KEYS -- Return all the keys in the current database without blocking
/// the server. The keys do not represent a point-in-time state so only the keys
/// that were in the database from the start to the end are guaranteed to be
/// there.
#[rcmd("hello.keys")]
fn hellokeys_rediscommand(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    if args.len() != 1 {
        return Err(Error::WrongArity);
    }
    // Note that when blocking the client we do not set any callback: no
    // timeout is possible since we passed '0', nor we need a reply callback
    // because we'll use the thread safe context to accumulate a reply.
    let bc = ctx
        .block_client(None, None, None, Duration::from_millis(0))
        .unwrap();

    // Now that we setup a blocking client, we need to pass the control
    // to the thread. However we need to pass arguments to the thread:
    // the reference to the blocked client handle.
    if thread::Builder::new()
        .spawn(move || hellokeys_thread_main(bc))
        .is_err()
    {
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
