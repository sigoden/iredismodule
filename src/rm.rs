use std::fmt;
use std::time::Duration;
use std::cell::RefCell;

use std::os::raw::{c_char, c_double, c_int, c_long, c_longlong};
use crate::raw;
use crate::error::Error;


/// wrap RedisModule_Milliseconds
pub fn milliseconds() -> Duration {
    unimplemented!()
}

pub fn handle_status(status: i32, message: &str) -> Result<(), Error> {
    if status == raw::REDISMODULE_OK as i32 {
        Ok(())
    } else {
        Err(Error::generic(message))
    }
}

/// wrap RedisModule_GetMyClusterID
pub fn get_cluster_id() -> String {
    unimplemented!()
}

/// wrap RedisModule_GetClusterSize
pub fn get_cluster_size() -> usize {
    unimplemented!()
}

/// wrap RedisModule_ZsetAddFlagsToCoreFlags
pub fn zset_add_flags_to_core_flags(flag: i32) -> i32 {
    unimplemented!()
}

/// wrap RedisModule_ZsetAddFlagsFromCoreFlags
pub fn zset_add_flags_from_core_flags(flag: i32) -> i32 {
    unimplemented!()
}

// wrap RedisModule_GetRandomBytes
pub fn get_random_bytes() -> String {
    unimplemented!()
}

// wrap RedisModule_GetRandomHexChars
pub fn get_random_hex_chars() -> String {
    unimplemented!()
}

#[derive(Debug, PartialEq)]
pub enum CmdStrFlags {
    Write,
    Readonly,
    Admin,
    DenyOOM,
    DenyScript,
    AllowLoading,
    Pubsub,
    Random,
    AllowStale,
    NoMonitor,
    Fast,
    GetkeysApi,
    NoCluster,
}

impl fmt::Display for CmdStrFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Write => write!(f, "write"),
            Readonly => write!(f, "readonly"),
            Admin => write!(f, "admin"),
            DenyOOM => write!(f, "deny-oom"),
            DenyScript => write!(f, "deny-script"),
            AllowLoading => write!(f, "allow-loading"),
            Pubsub => write!(f, "pubsub"),
            Random => write!(f, "random"),
            AllowStale => write!(f, "allow-stale"),
            NoMonitor => write!(f, "no-monitor"),
            Fast => write!(f, "fast"),
            GetkeysApi => write!(f, "getkeys-api"),
            NoCluster => write!(f, "no-cluster"),
        }
    }
}

impl CmdStrFlags {
    pub fn none() -> String {
        "".to_string()
    }
    pub fn one(flag: CmdStrFlags) -> String {
        flag.to_string()
    }
    pub fn multi(flags: &[CmdStrFlags]) -> String {
        flags.into_iter().map(|v| v.to_string()).collect::<Vec<String>>().join(" ")
    }
}

pub(crate) const OK: i32 = raw::REDISMODULE_OK as i32;
pub(crate) const ERR: i32 = raw::REDISMODULE_ERR as i32;

pub enum CmdFmtFlags { C, S, B, L, V, A, R, X } 

impl Default for CmdFmtFlags {
    fn default() -> Self {
        CmdFmtFlags::V
    }
}

impl fmt::Display for CmdFmtFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            C=> write!(f, "c"),
            S=> write!(f, "s"),
            B=> write!(f, "b"),
            L=> write!(f, "l"),
            V=> write!(f, "v"),
            A=> write!(f, "a"),
            R=> write!(f, "r"),
            X=> write!(f, "!"),
        }
    }
}

impl CmdFmtFlags {
    pub fn multi(flags: &[CmdFmtFlags]) -> String {
        flags.into_iter().map(|v| v.to_string()).collect::<Vec<String>>().join("")
    }
}

pub struct CtxFlags(u32);

impl CtxFlags {
    pub fn new(i: u32) -> Self {
        CtxFlags(i)
    }
    pub fn is_lua(&self) -> bool {
        self.0 & raw::REDISMODULE_CTX_FLAGS_LUA != 0
    }
    pub fn is_multi(&self) -> bool {
        self.0 & raw::REDISMODULE_CTX_FLAGS_MULTI != 0
    }
    pub fn is_master(&self) -> bool {
        self.0 & raw::REDISMODULE_CTX_FLAGS_MASTER != 0
    }
    pub fn is_slave(&self) -> bool {
        self.0 & raw::REDISMODULE_CTX_FLAGS_SLAVE != 0
    }
    pub fn is_readonly(&self) -> bool {
        self.0 & raw::REDISMODULE_CTX_FLAGS_READONLY != 0
    }
    pub fn is_cluster(&self) -> bool {
        self.0 & raw::REDISMODULE_CTX_FLAGS_CLUSTER != 0
    }
    pub fn is_aof(&self) -> bool {
        self.0 & raw::REDISMODULE_CTX_FLAGS_AOF != 0
    }
    pub fn is_rdb(&self) -> bool {
        self.0 & raw::REDISMODULE_CTX_FLAGS_RDB != 0
    }
    pub fn is_max_memory(&self) -> bool {
        self.0 & raw::REDISMODULE_CTX_FLAGS_MAXMEMORY != 0
    }
    pub fn is_evict(&self) -> bool {
        self.0 & raw::REDISMODULE_CTX_FLAGS_EVICT != 0
    }
    pub fn is_oom(&self) -> bool {
        self.0 & raw::REDISMODULE_CTX_FLAGS_OOM != 0
    }
    pub fn is_oom_warning(&self) -> bool {
        self.0 & raw::REDISMODULE_CTX_FLAGS_OOM_WARNING != 0
    }
    pub fn is_replicated(&self) -> bool {
        self.0 & raw::REDISMODULE_CTX_FLAGS_REPLICATED != 0
    }
    pub fn is_loading(&self) -> bool {
        self.0 & raw::REDISMODULE_CTX_FLAGS_LOADING != 0
    }
}

pub type TimerID = i32;
pub struct ClusterNodeList {
}

pub type ClusterNode = String;
pub type MsgType = u8;

impl Drop for ClusterNodeList {
    fn drop(&mut self) {
        // unsafe { raw::RedisModule_FreeClusterNodesList().unwrap()(self.inner) }
    }
}

pub enum KeySpaceTypes {
}

pub struct RedisType {
    name: &'static str,
    version: i32,
    type_methods: raw::RedisModuleTypeMethods,
    pub raw_type: RefCell<*mut raw::RedisModuleType>,
}