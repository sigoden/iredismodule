//! This crate provides an idiomatic Rust API for the [Redis Modules API](https://redis.io/topics/modules-intro).
//! It allows writing Redis modules in Rust, without needing to use raw pointers or unsafe code.
//!
//! # Example
//! ```rust,no_run
//! use iredismodule_macros::rcmd;
//! use iredismodule::prelude::*; 
//! 
//! /// Define command
//! #[rcmd("hello.simple", "readonly", 0, 0, 0)] 
//! fn hello_simple(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
//!     let db = ctx.get_select_db();
//!     Ok(db.into())
//! }
//! 
//! // Register module
//! define_module! {
//!     name: "simple",
//!     version: 1,
//!     data_types: [],
//!     init_funcs: [],
//!     commands: [
//!         hello_simple_cmd,
//!     ]
//! }
//! ```
pub mod raw;

mod alloc;
pub mod block_client;
pub mod call_reply;
pub mod cluster;
mod common;
pub mod context;
pub mod error;
pub mod io;
pub mod key;
mod macros;
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
    get_used_memory_ratio, handle_status, is_module_busy, latency_add_sample, log, milliseconds,
    parse_args, reset_dataset, FromPtr, GetPtr, LogLevel, CallFlag, ServerEvent,
};

/// Result of redis comamnd call
pub type RResult = Result<value::Value, error::Error>;
