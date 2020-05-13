pub mod raw;


mod alloc;
pub mod block_client;
pub mod buffer;
pub mod call_reply;
mod common;
pub mod error;
pub mod macros;
pub mod cluster;
pub mod context;
pub mod io;
pub mod key;
pub mod rtype;
pub mod string;
pub mod value;
pub mod user;
pub mod scan_cursor;
pub mod prelude;

#[global_allocator]
static ALLOC: crate::alloc::RedisAlloc = crate::alloc::RedisAlloc;

pub use common::{
    handle_status, is_module_busy, milliseconds, parse_args, reset_dataset, get_client_info_by_id,
    avoid_replica_traffic, latency_add_sample, get_notify_keyspace_events, get_used_memory_ratio,
    ArgvFlags, LogLevel, Ptr, StatusCode, ServerEvent
};

/// Result of redis comamnd call
pub type RResult = Result<value::Value, error::Error>;