#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;

pub mod raw;
mod error;
mod rm;
mod rm_ctx;
mod rm_str;
mod rm_value;
mod rm_call_reply;
mod rm_key;
mod rm_io;
mod rm_block_client;

pub use error::Error;
pub use rm::{
    CmdFmtFlags,
    CmdStrFlags,
    handle_status,
    milliseconds,
    CtxFlags,
    TimerID,
    ClusterNodeList,
    ClusterNode,
    MsgType,
    KeySpaceTypes,
    RedisType,
};

pub use rm_ctx::Ctx;
pub use rm_str::Str;
pub use rm_value::{RedisResult, RedisValue, REDIS_OK};
pub use rm_call_reply::{CallReply};
pub use rm_key::{ReadKey, WriteKey};
pub use rm_io::{IO};
pub use rm_block_client::{BlockClient};