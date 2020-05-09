use bitflags::bitflags;
use num_traits::FromPrimitive;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::slice;
use std::time::Duration;

use crate::raw;
use crate::{Error, RedisStr};

pub trait Ptr {
    type PtrType;
    fn get_ptr(&self) -> *mut Self::PtrType;
}

/// wrap RedisModule_Milliseconds
pub fn milliseconds() -> Duration {
    Duration::from_millis(unsafe { raw::RedisModule_Milliseconds.unwrap()() } as u64)
}

pub fn parse_args<'a>(
    argv: *mut *mut raw::RedisModuleString,
    argc: c_int,
) -> Vec<RedisStr> {
    unsafe { slice::from_raw_parts(argv, argc as usize) }
        .into_iter()
        .map(|&arg| RedisStr::from_ptr(arg))
        .collect()
}

pub fn handle_status(status: i32, message: &str) -> Result<(), Error> {
    if status == raw::REDISMODULE_OK as i32 {
        Ok(())
    } else {
        Err(Error::generic(message))
    }
}

pub fn is_module_busy(name: &str) -> Result<(), Error> {
    let name = CString::new(name).unwrap();
    handle_status(
        unsafe { raw::RedisModule_IsModuleNameBusy.unwrap()(name.as_ptr()) },
        "Cloud not check busy",
    )
}

/// wrap RedisModule_GetMyClusterID
pub fn get_my_cluster_id() -> Result<String, Error> {
    let c_buf: *const c_char = unsafe { raw::RedisModule_GetMyClusterID.unwrap()() };
    if c_buf.is_null() {
        Err(Error::generic("Cluster is disabled"))
    } else {
        let c_str: &CStr = unsafe { CStr::from_ptr(c_buf) };
        Ok(c_str.to_str()?.to_owned())
    }
}

/// wrap RedisModule_GetClusterSize
pub fn get_cluster_size() -> usize {
    unsafe { raw::RedisModule_GetClusterSize.unwrap()() }
}

pub(crate) const FMT: *const i8 = b"v\0".as_ptr() as *const i8;

#[derive(Primitive, Debug, PartialEq)]
pub enum StatusCode {
    Ok = raw::REDISMODULE_OK as isize,
    Err = raw::REDISMODULE_ERR as isize,
}
impl From<c_int> for StatusCode {
    fn from(v: c_int) -> Self {
        StatusCode::from_i32(v).unwrap()
    }
}

bitflags! {
    pub struct CtxFlags: u32 {
        const LUA = raw::REDISMODULE_CTX_FLAGS_LUA;
        const MULTI = raw::REDISMODULE_CTX_FLAGS_MULTI;
        const MASTER = raw::REDISMODULE_CTX_FLAGS_MASTER;
        const SLAVE = raw::REDISMODULE_CTX_FLAGS_SLAVE;
        const READONLY = raw::REDISMODULE_CTX_FLAGS_READONLY;
        const CLUSTER = raw::REDISMODULE_CTX_FLAGS_CLUSTER;
        const AOF = raw::REDISMODULE_CTX_FLAGS_AOF;
        const RDB = raw::REDISMODULE_CTX_FLAGS_RDB;
        const MAXMEMORY = raw::REDISMODULE_CTX_FLAGS_MAXMEMORY;
        const EVICT = raw::REDISMODULE_CTX_FLAGS_EVICT;
        const OOM = raw::REDISMODULE_CTX_FLAGS_OOM;
        const OOM_WARNING = raw::REDISMODULE_CTX_FLAGS_OOM_WARNING;
        const REPLICATED = raw::REDISMODULE_CTX_FLAGS_REPLICATED;
        const LOADING = raw::REDISMODULE_CTX_FLAGS_LOADING;
        const REPLICA_IS_STALE = raw::REDISMODULE_CTX_FLAGS_REPLICA_IS_STALE;
        const REPLICA_IS_CONNECTING = raw::REDISMODULE_CTX_FLAGS_REPLICA_IS_CONNECTING;
        const REPLICA_IS_TRANSFERRING = raw::REDISMODULE_CTX_FLAGS_REPLICA_IS_TRANSFERRING;
        const REPLICA_IS_ONLINE = raw::REDISMODULE_CTX_FLAGS_REPLICA_IS_ONLINE;
        const ACTIVE_CHILD = raw::REDISMODULE_CTX_FLAGS_ACTIVE_CHILD;
        const MULTI_DIRTY = raw::REDISMODULE_CTX_FLAGS_MULTI_DIRTY;
    }
}

pub enum KeySpaceTypes {}

#[derive(Clone, Copy, Debug)]
pub enum LogLevel {
    Debug,
    Notice,
    Verbose,
    Warning,
}

impl Into<CString> for LogLevel {
    fn into(self) -> CString {
        CString::new(format!("{:?}", self).to_lowercase()).unwrap()
    }
}
