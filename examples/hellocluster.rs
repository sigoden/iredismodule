use iredismodule::cluster::MsgType;
use iredismodule::prelude::*;
use iredismodule::raw;
use iredismodule_macros::{rcmd, rwrap};

const MSGTYPE_PING: MsgType = 1;
const MSGTYPE_PONG: MsgType = 2;

/// HELLOCLUSTER.PINGALL
#[rcmd("hellocluster.pingall", "readonly")]
fn hellocluster_pingall(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    ctx.send_cluster_message("", MSGTYPE_PING, "Hey".as_bytes())?;
    Ok("Ok".into())
}

/// HELLOCLUSTER.LIST
#[rcmd("hellocluster.list", "readonly")]
fn hellocluster_list(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let ids = ctx.get_cluster_nodes_list();
    if ids.len() == 0 {
        return Err(Error::new("ERR cluster not enabled"));
    }
    let value = ids.into_iter().map(|v| Value::from(v)).collect();
    Ok(Value::Array(value))
}

/// Callback for message MSGTYPE_PING
#[rwrap("cluster_msg")]
fn on_ping(ctx: &Context, sender_id: &str, msg_type: MsgType, payload: &[u8]) {
    let msg = std::str::from_utf8(payload).unwrap();
    ctx.notice(format!(
        "PING (type {}) RECEIVED from {} {}",
        msg_type, sender_id, msg
    ));
}

/// Callback for message MSGTYPE_PONG.
#[rwrap("cluster_msg")]
fn on_pong(ctx: &Context, sender_id: &str, msg_type: MsgType, payload: &[u8]) {
    let msg = std::str::from_utf8(payload).unwrap();
    ctx.notice(format!(
        "PING (type {}) RECEIVED from {} {}",
        msg_type, sender_id, msg
    ));
}

#[rwrap("call")]
fn init(ctx: &mut Context, _: Vec<RStr>) -> Result<(), Error> {
    // Disable Redis Cluster sharding and redirections. This way every node
    // will be able to access every possible key, regardless of the hash slot.
    // This way the PING message handler will be able to increment a specific
    // variable. Normally you do that in order for the distributed system
    // you create as a module to have total freedom in the keyspace
    // manipulation.
    ctx.set_cluster_flags(raw::REDISMODULE_CLUSTER_FLAG_NO_REDIRECTION);

    // Register our handlers for different message types.
    ctx.register_cluster_message_receiver(MSGTYPE_PING, Some(on_ping_c));
    ctx.register_cluster_message_receiver(MSGTYPE_PONG, Some(on_pong_c));
    Ok(())
}

define_module! {
    name: "hellocluster",
    version: 1,
    data_types: [],
    init_funcs: [
        init_c,
    ],
    commands: [
        hellocluster_pingall_cmd,
        hellocluster_list_cmd,
    ]
}
