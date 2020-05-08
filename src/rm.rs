use std::cell::RefCell;
use std::ffi::{CString, CStr};
use std::fmt;
use std::time::Duration;
use std::os::raw::{c_char};

use crate::raw;
use crate::{Error};

/// wrap RedisModule_Milliseconds
pub fn milliseconds() -> Duration {
    Duration::from_millis(
        unsafe {
            raw::RedisModule_Milliseconds.unwrap()()
        } as u64
    )
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
            Self::Write => write!(f, "write"),
            Self::Readonly => write!(f, "readonly"),
            Self::Admin => write!(f, "admin"),
            Self::DenyOOM => write!(f, "deny-oom"),
            Self::DenyScript => write!(f, "deny-script"),
            Self::AllowLoading => write!(f, "allow-loading"),
            Self::Pubsub => write!(f, "pubsub"),
            Self::Random => write!(f, "random"),
            Self::AllowStale => write!(f, "allow-stale"),
            Self::NoMonitor => write!(f, "no-monitor"),
            Self::Fast => write!(f, "fast"),
            Self::GetkeysApi => write!(f, "getkeys-api"),
            Self::NoCluster => write!(f, "no-cluster"),
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
        flags
            .into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

pub(crate) const CODE_OK: i32 = raw::REDISMODULE_OK as i32;
pub(crate) const CODE_ERR: i32 = raw::REDISMODULE_ERR as i32;

pub enum CmdFmtFlags {
    C,
    S,
    B,
    L,
    V,
    A,
    R,
    X,
}

impl Default for CmdFmtFlags {
    fn default() -> Self {
        CmdFmtFlags::V
    }
}

impl fmt::Display for CmdFmtFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::C => write!(f, "c"),
            Self::S => write!(f, "s"),
            Self::B => write!(f, "b"),
            Self::L => write!(f, "l"),
            Self::V => write!(f, "v"),
            Self::A => write!(f, "a"),
            Self::R => write!(f, "r"),
            Self::X => write!(f, "!"),
        }
    }
}

impl CmdFmtFlags {
    pub fn multi(flags: &[CmdFmtFlags]) -> String {
        flags
            .into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join("")
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

pub struct ClusterNodeList {}

pub type ClusterNode = String;
pub type MsgType = u8;

impl Drop for ClusterNodeList {
    fn drop(&mut self) {
        // unsafe { raw::RedisModule_FreeClusterNodesList().unwrap()(self.inner) }
    }
}

pub enum KeySpaceTypes {}

pub struct RedisType {
    name: &'static str,
    version: i32,
    type_methods: raw::RedisModuleTypeMethods,
    pub raw_type: RefCell<*mut raw::RedisModuleType>,
}

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
