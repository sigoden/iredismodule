use iredismodule::prelude::*;
use iredismodule::raw;
use iredismodule::ServerEvent;
use iredismodule_macros::rwrap;

/// Client state change callback.
extern "C" fn client_change_callback_c(
    _ctx: *mut raw::RedisModuleCtx,
    _eid: raw::RedisModuleEvent,
    subevent: u64,
    data: *mut ::std::os::raw::c_void,
) {
    let ci: &mut raw::RedisModuleClientInfo =
        unsafe { &mut *(data as *mut raw::RedisModuleClientInfo) };
    let addr: String = ci.addr.iter().map(|v| (*v as u8) as char).collect();
    println!(
        "Client {} event for client #{} {}:{}\n",
        if subevent == raw::REDISMODULE_SUBEVENT_CLIENT_CHANGE_CONNECTED as u64 {
            "connection"
        } else {
            "disconnection"
        },
        ci.id,
        addr,
        ci.port,
    );
}

extern "C" fn flushdb_callback_c(
    ctx: *mut raw::RedisModuleCtx,
    _eid: raw::RedisModuleEvent,
    subevent: u64,
    data: *mut ::std::os::raw::c_void,
) {
    let context = Context::from_ptr(ctx);
    let ci: &mut raw::RedisModuleFlushInfo =
        unsafe { &mut *(data as *mut raw::RedisModuleFlushInfo) };
    if subevent == raw::REDISMODULE_SUBEVENT_FLUSHDB_START as u64 {
        if ci.dbnum != -1 {
            let reply = context.call("DBSIZE", None, &vec![]).unwrap();
            let num_keys = reply.get_integer().unwrap();
            println!(
                "FLUSHDB event of database {} started ({} keys in DB)\n",
                ci.dbnum, num_keys
            );
        } else {
            println!("FLUSHALL event started\n");
        }
    } else {
        if ci.dbnum != -1 {
            println!("FLUSHDB event of database {} ended\n", ci.dbnum);
        } else {
            println!("FLUSHALL event ened\n");
        }
    }
}

#[rwrap("call")]
fn init(ctx: &mut Context, _args: Vec<RStr>) -> Result<(), Error> {
    ctx.subscribe_to_server_event(ServerEvent::ClientChange, Some(client_change_callback_c))?;
    ctx.subscribe_to_server_event(ServerEvent::FlushDB, Some(flushdb_callback_c))?;
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
