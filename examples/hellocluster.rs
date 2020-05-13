use redismodule_macros::{rcmd, rwrap};
use redismodule::prelude::*;
use redismodule::raw;
use redismodule::cluster::MsgType;

const MSGTYPE_PING: MsgType = 1;
const MSGTYPE_PONG: MsgType = 2;

#[rcmd("hellocluster.pingall", "readonly")]
fn hellocluster_pingall(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    ctx.send_cluster_message_all(MSGTYPE_PING, "Hey".as_bytes())?;
    Ok("Ok".into())
}

#[rcmd("hellocluster.list", "readonly")]
fn hellocluster_list(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let ids = ctx.get_cluster_nodes_list();
    if ids.is_none() {
        return Err(Error::new("ERR cluster not enabled"));
    }
    let values = ids
        .unwrap()
        .value()
        .iter()
        .map(|v| Value::from(v.to_string()))
        .collect();
    Ok(Value::Array(values))
}

#[rwrap("cluster_msg")]
fn on_ping(ctx: &Context, sender_id: &str, msg_type: MsgType, payload: &[u8]) {
    let msg = std::str::from_utf8(payload).unwrap();
    ctx.notice(format!(
        "PING (type {}) RECEIVED from {} {}",
        msg_type, sender_id, msg
    ));
}

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
    ctx.set_cluster_flags(raw::REDISMODULE_CLUSTER_FLAG_NO_REDIRECTION);
    ctx.register_cluster_message_receiver(MSGTYPE_PING, on_ping_c);
    ctx.register_cluster_message_receiver(MSGTYPE_PONG, on_pong_c);
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
