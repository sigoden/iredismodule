#[macro_use]
extern crate enum_primitive_derive;
extern crate bitflags;
extern crate num_traits;

mod error;
mod macros;
pub mod raw;
mod rm;
mod rm_block_client;
mod rm_buffer;
mod rm_call_reply;
mod rm_context;
mod rm_io;
mod rm_key;
mod rm_key_type;
mod rm_string;
mod rm_value;

use rm::FMT;

pub use error::Error;
pub use rm::{
    get_cluster_size, get_my_cluster_id, handle_status, is_module_busy, milliseconds, parse_args,
    CtxFlags, KeySpaceTypes, LogLevel, StatusCode,
};
use rm_context::take_data;

pub use rm_context::cluster::{ClusterNode, ClusterNodeList, MsgType};
pub use rm_context::{ClusterFlags, Context};

pub use raw::RedisModuleTimerID as TimerID;

pub use rm_block_client::BlockClient;
pub use rm_buffer::RedisBuffer;
pub use rm_call_reply::CallReply;
pub use rm_io::{RedisDigest, RedisIO};
pub use rm_key::{
    AccessMode, KeyType, ListPosition, ReadKey, WriteKey, ZaddInputFlag, ZaddOutputFlag,
};
pub use rm_key_type::RedisType;
pub use rm_string::RedisString;
pub use rm_value::{RedisResult, RedisValue};
