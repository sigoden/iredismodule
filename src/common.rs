use std::collections::HashSet;
use std::ffi::CString;
use std::os::raw::{c_int, c_void};
use std::slice;
use std::time::Duration;

use crate::raw;
use crate::error::Error;
use crate::string::RStr;

pub trait Ptr {
    type PtrType;
    fn get_ptr(&self) -> *mut Self::PtrType;
}

pub fn milliseconds() -> Duration {
    Duration::from_millis(unsafe { raw::RedisModule_Milliseconds.unwrap()() } as u64)
}

pub fn parse_args<'a>(argv: *mut *mut raw::RedisModuleString, argc: c_int) -> Vec<RStr> {
    unsafe { slice::from_raw_parts(argv, argc as usize) }
        .into_iter()
        .map(|&arg| RStr::from_ptr(arg))
        .collect()
}

pub fn handle_status(status: i32, message: &str) -> Result<(), Error> {
    if status == raw::REDISMODULE_OK as i32 {
        Ok(())
    } else {
        Err(Error::new(message))
    }
}

pub fn is_module_busy(name: &str) -> Result<(), Error> {
    let name = CString::new(name).unwrap();
    handle_status(
        unsafe { raw::RedisModule_IsModuleNameBusy.unwrap()(name.as_ptr()) },
        "fail to check busy",
    )
}

pub fn reset_dataset(restart_aof: bool, async_: bool) {
    unsafe { raw::RedisModule_ResetDataset.unwrap()(restart_aof as i32, async_ as i32) }
}

pub fn get_client_info_by_id(id: u64) -> Result<&'static raw::RedisModuleClientInfo, Error> {
    let ptr: *mut raw::RedisModuleClientInfo = std::ptr::null_mut();
    handle_status(
        unsafe { raw::RedisModule_GetClientInfoById.unwrap()(ptr as *mut c_void, id) },
        "fail to get client info"
    )?;
    Ok(unsafe { &(*ptr) })
}

pub fn avoid_replica_traffic() -> Result<(), Error> {
    handle_status(
        unsafe { raw::RedisModule_AvoidReplicaTraffic.unwrap()() },
        "fail to call avoid_replica_traffic"
    )
}
pub fn latency_add_sample(name: &str, ms: Duration) {
    let name = CString::new(name).unwrap();
    unsafe {
        raw::RedisModule_LatencyAddSample.unwrap()(name.as_ptr(), ms.as_millis() as i64)
     }
}

pub fn get_notify_keyspace_events() -> i32 {
    unsafe { raw::RedisModule_GetNotifyKeyspaceEvents.unwrap()() }
}

pub fn get_used_memory_ratio() -> f32 {
    unsafe { raw::RedisModule_GetUsedMemoryRatio.unwrap()() }
}

pub enum StatusCode {
    Ok = raw::REDISMODULE_OK as isize,
    Err = raw::REDISMODULE_ERR as isize,
}
impl From<c_int> for StatusCode {
    fn from(v: c_int) -> Self {
        if v == raw::REDISMODULE_OK as c_int {
            StatusCode::Ok
        } else {
            StatusCode::Err
        }
    }
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

#[derive(Debug)]
pub struct ArgvFlags(HashSet<char>);

impl ArgvFlags {
    pub fn new() -> ArgvFlags {
        let mut s = HashSet::new();
        s.insert('v');
        ArgvFlags(s)
    }
    pub fn replicate(&mut self) -> &mut ArgvFlags {
        self.0.insert('!');
        self
    }
    pub fn no_aof(&mut self) -> &mut ArgvFlags {
        self.0.insert('A');
        self
    }
    pub fn no_replicas(&mut self) -> &mut ArgvFlags {
        self.0.insert('R');
        self
    }
}

impl Into<CString> for ArgvFlags {
    fn into(self) -> CString {
        let v = self.0.into_iter().collect::<String>();
        CString::new(v).unwrap()
    }
}

pub enum ServerEvent {
    ReplicationRoleChanged = raw::REDISMODULE_EVENT_REPLICATION_ROLE_CHANGED as isize,
    Persistence = raw::REDISMODULE_EVENT_PERSISTENCE as isize,
    FlushDB = raw::REDISMODULE_EVENT_FLUSHDB as isize,
    Loading = raw::REDISMODULE_EVENT_LOADING as isize,
    ClientChange = raw::REDISMODULE_EVENT_CLIENT_CHANGE as isize,
    Shutdown = raw::REDISMODULE_EVENT_SHUTDOWN as isize,
    ReplicaChange = raw::REDISMODULE_EVENT_REPLICA_CHANGE as isize,
    CronLoop = raw::REDISMODULE_EVENT_CRON_LOOP as isize,
    MasterLinkChange = raw::REDISMODULE_EVENT_MASTER_LINK_CHANGE as isize,
    ModuleChange = raw::REDISMODULE_EVENT_MODULE_CHANGE as isize,
    LoadingProgress = raw::REDISMODULE_EVENT_LOADING_PROGRESS as isize,
}

impl Into<raw::RedisModuleEvent> for ServerEvent {
    fn into(self) -> raw::RedisModuleEvent {
        match self {
            ServerEvent::ReplicationRoleChanged => raw::RedisModuleEvent_ReplicationRoleChanged,
            ServerEvent::Persistence => raw::RedisModuleEvent_Persistence,
            ServerEvent::FlushDB => raw::RedisModuleEvent_FlushDB,
            ServerEvent::Loading => raw::RedisModuleEvent_Loading,
            ServerEvent::ClientChange => raw::RedisModuleEvent_ClientChange,
            ServerEvent::Shutdown => raw::RedisModuleEvent_Shutdown,
            ServerEvent::ReplicaChange => raw::RedisModuleEvent_ReplicaChange,
            ServerEvent::CronLoop => raw::RedisModuleEvent_CronLoop,
            ServerEvent::MasterLinkChange => raw::RedisModuleEvent_MasterLinkChange,
            ServerEvent::ModuleChange => raw::RedisModuleEvent_ModuleChange,
            ServerEvent::LoadingProgress => raw::RedisModuleEvent_LoadingProgress,
        }
    }
}