use std::ffi::CString;
use std::os::raw::{c_int, c_void};
use std::time::Duration;

use crate::error::Error;
use crate::raw;
use crate::string::RStr;

/// Get the inner ptr from a wrapper struct
pub trait GetPtr {
    type PtrType;
    fn get_ptr(&self) -> *mut Self::PtrType;
}
/// Generate a wrapper struct from raw ptr
pub trait FromPtr {
    type PtrType;
    fn from_ptr(ptr: *mut Self::PtrType) -> Self;
}

/// Return the current UNIX time in milliseconds.
pub fn milliseconds() -> Duration {
    Duration::from_millis(unsafe { raw::RedisModule_Milliseconds.unwrap()() } as u64)
}
/// Check whether module name is used
pub fn is_module_busy(name: &str) -> bool {
    let name = CString::new(name).unwrap();
    let ret = unsafe { raw::RedisModule_IsModuleNameBusy.unwrap()(name.as_ptr()) };
    ret == 1
}
/// Performs similar operation to FLUSHALL, and optionally start a new AOF file (if enabled)
///
/// If restart_aof is true, you must make sure the command that triggered this call is not
/// propagated to the AOF file.
/// When async is set to true, db contents will be freed by a background thread.
pub fn reset_dataset(restart_aof: bool, async_: bool) {
    unsafe { raw::RedisModule_ResetDataset.unwrap()(restart_aof as i32, async_ as i32) }
}

/// Return information about the client with the specified ID
///
/// If the client exists, Ok is returned, otherwise Err is returned.
///
/// When the client exist and the `ci` pointer is not NULL, but points to
/// a structure of type RedisModuleClientInfo, previously initialized with
/// the correct REDISMODULE_CLIENTINFO_INITIALIZER, the structure is populated
/// with the following fields:
///
///      uint64_t flags;         // REDISMODULE_CLIENTINFO_FLAG_*
///      uint64_t id;            // Client ID
///      char addr[46];          // IPv4 or IPv6 address.
///      uint16_t port;          // TCP port.
///      uint16_t db;            // Selected DB.
///
/// Note: the client ID is useless in the context of this call, since we
///       already know, however the same structure could be used in other
///       contexts where we don't know the client ID, yet the same structure
///       is returned.
///
/// With flags having the following meaning:
///
///     REDISMODULE_CLIENTINFO_FLAG_SSL          Client using SSL connection.
///     REDISMODULE_CLIENTINFO_FLAG_PUBSUB       Client in Pub/Sub mode.
///     REDISMODULE_CLIENTINFO_FLAG_BLOCKED      Client blocked in command.
///     REDISMODULE_CLIENTINFO_FLAG_TRACKING     Client with keys tracking on.
///     REDISMODULE_CLIENTINFO_FLAG_UNIXSOCKET   Client using unix domain socket.
///     REDISMODULE_CLIENTINFO_FLAG_MULTI        Client in MULTI state.
///
/// However passing NULL is a way to just check if the client exists in case
/// we are not interested in any additional information.
pub fn get_client_info_by_id(id: u64) -> Result<&'static raw::RedisModuleClientInfo, Error> {
    let ptr: *mut raw::RedisModuleClientInfo = std::ptr::null_mut();
    handle_status(
        unsafe { raw::RedisModule_GetClientInfoById.unwrap()(ptr as *mut c_void, id) },
        "fail to get client info",
    )?;
    Ok(unsafe { &(*ptr) })
}

/// Returns true if some client sent the CLIENT PAUSE command to the server or
/// if Redis Cluster is doing a manual failover, and paused tue clients.
///
/// This is needed when we have a master with replicas, and want to write,
/// without adding further data to the replication channel, that the replicas
/// replication offset, match the one of the master. When this happens, it is
/// safe to failover the master without data loss.
///
/// However modules may generate traffic by calling `Context::call` with
/// the "!" flag, or by calling `Context::replicate`, in a context outside
/// commands execution, for instance in timeout callbacks, threads safe
/// contexts, and so forth. When modules will generate too much traffic, it
/// will be hard for the master and replicas offset to match, because there
/// is more data to send in the replication channel.
///
/// So modules may want to try to avoid very heavy background work that has
/// the effect of creating data to the replication channel, when this function
/// returns true. This is mostly useful for modules that have background
/// garbage collection tasks, or that do writes and replicate such writes
/// periodically in timer callbacks or other periodic callbacks.
pub fn avoid_replica_traffic() -> Result<(), Error> {
    handle_status(
        unsafe { raw::RedisModule_AvoidReplicaTraffic.unwrap()() },
        "fail to call avoid_replica_traffic",
    )
}

/// Allows adding event to the latency monitor to be observed by the LATENCY
/// command.
///
/// The call is skipped if the latency is smaller than the configured
/// latency-monitor-threshold.
pub fn latency_add_sample(name: &str, ms: Duration) {
    let name = CString::new(name).unwrap();
    unsafe { raw::RedisModule_LatencyAddSample.unwrap()(name.as_ptr(), ms.as_millis() as i64) }
}

/// Get the configured bitmap of notify-keyspace-events (Could be used
/// for additional filtering in RedisModuleNotificationFunc)
pub fn get_notify_keyspace_events() -> i32 {
    unsafe { raw::RedisModule_GetNotifyKeyspaceEvents.unwrap()() }
}

/// Return the a number between 0 to 1 indicating the amount of memory
/// currently used, relative to the Redis "maxmemory" configuration.
///
/// 0 - No memory limit configured.
/// Between 0 and 1 - The percentage of the memory used normalized in 0-1 range.
/// Exactly 1 - Memory limit reached.
/// Greater 1 - More memory used than the configured limit.
pub fn get_used_memory_ratio() -> f32 {
    unsafe { raw::RedisModule_GetUsedMemoryRatio.unwrap()() }
}

/// Redis log level
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

/// Controls the Whether replicate the command
pub enum CallFlag {
    /// Tells the function to replicate the command to replicas and AOF
    AofAndReplicas,
    /// Tells the function to replicate the command to AOF only
    Aof,
    /// Tells the function to replicate the command to replicas
    Replicas,
}

impl Into<CString> for CallFlag {
    fn into(self) -> CString {
        match self {
            CallFlag::AofAndReplicas => CString::new("v!").unwrap(),
            CallFlag::Aof => CString::new("vR").unwrap(),
            CallFlag::Replicas => CString::new("vA").unwrap(),
        }
    }
}

/// Events kind used in `Context::subscribe_to_server_event`
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

/// Parse the argv/argc of redis command func
pub fn parse_args<'a>(argv: *mut *mut raw::RedisModuleString, argc: c_int) -> Vec<RStr> {
    unsafe { std::slice::from_raw_parts(argv, argc as usize) }
        .into_iter()
        .map(|&arg| RStr::from_ptr(arg))
        .collect()
}

/// Check ret return code of redis module api
pub fn handle_status<T: AsRef<str>>(status: i32, message: T) -> Result<(), Error> {
    if status == raw::REDISMODULE_OK as i32 {
        Ok(())
    } else {
        Err(Error::new(message.as_ref()))
    }
}
