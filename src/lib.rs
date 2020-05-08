#[macro_use]
extern crate enum_primitive_derive;
extern crate bitflags;
extern crate num_traits;

mod error;
pub mod raw;
mod rm;
mod rm_block_client;
mod rm_buffer;
mod rm_call_reply;
mod rm_ctx;
mod rm_io;
mod rm_key;
mod rm_string;
mod rm_value;

pub use error::Error;
pub use rm::{
    get_cluster_size, handle_status, is_module_busy, milliseconds, get_my_cluster_id,
    ClusterNode, ClusterNodeList, CmdFmtFlags, CmdStrFlags, CtxFlags, KeySpaceTypes, LogLevel, 
    MsgType, RedisType,
};

pub use raw::RedisModuleTimerID as TimerID;

pub use rm_block_client::BlockClient;
pub use rm_buffer::RedisBuffer;
pub use rm_call_reply::CallReply;
pub use rm_ctx::{ClusterFlags, Ctx};
pub use rm_io::{IO, Digest};
pub use rm_key::{KeyType, ListWhere, ReadKey, WriteKey, ZaddInputFlag, ZaddOutputFlag};
pub use rm_string::{RedisString, RedisStr};
pub use rm_value::{RedisResult, RedisValue};
