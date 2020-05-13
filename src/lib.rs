pub mod raw;

mod alloc;
pub mod block_client;
pub mod buffer;
pub mod call_reply;
pub mod cluster;
mod common;
pub mod context;
pub mod error;
pub mod io;
pub mod key;
pub mod macros;
pub mod prelude;
pub mod rtype;
pub mod scan_cursor;
pub mod string;
pub mod user;
pub mod value;

#[global_allocator]
static ALLOC: crate::alloc::RedisAlloc = crate::alloc::RedisAlloc;

pub use common::{
    avoid_replica_traffic, get_client_info_by_id, get_notify_keyspace_events,
    get_used_memory_ratio, handle_status, is_module_busy, latency_add_sample, milliseconds,
    parse_args, reset_dataset, ArgvFlags, LogLevel, Ptr, ServerEvent, StatusCode,
};

/// Result of redis comamnd call
pub type RResult = Result<value::Value, error::Error>;
