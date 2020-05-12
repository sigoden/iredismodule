#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::os::raw::{c_char, c_int};
#[allow(improper_ctypes)]
#[link(name = "redismodule", kind = "static")]
extern "C" {
    pub fn Export_RedisModule_Init(
        ctx: *mut RedisModuleCtx,
        module_name: *const c_char,
        module_version: c_int,
        api_version: c_int,
    ) -> c_int;
}

pub type FreePrivateDataFunc = std::option::Option<
    unsafe extern "C" fn(arg1: *mut RedisModuleCtx, arg2: *mut ::std::os::raw::c_void),
>;

pub static RedisModuleEvent_ReplicationRoleChanged: RedisModuleEvent = RedisModuleEvent {
    id: REDISMODULE_EVENT_REPLICATION_ROLE_CHANGED as u64,
    dataver: 1,
};

pub static RedisModuleEvent_Persistence: RedisModuleEvent = RedisModuleEvent {
    id: REDISMODULE_EVENT_PERSISTENCE as u64,
    dataver: 1,
};

pub static RedisModuleEvent_FlushDB: RedisModuleEvent = RedisModuleEvent {
    id: REDISMODULE_EVENT_FLUSHDB as u64,
    dataver: 1,
};

pub static RedisModuleEvent_Loading: RedisModuleEvent = RedisModuleEvent {
    id: REDISMODULE_EVENT_LOADING as u64,
    dataver: 1,
};

pub static RedisModuleEvent_ClientChange: RedisModuleEvent = RedisModuleEvent {
    id: REDISMODULE_EVENT_CLIENT_CHANGE as u64,
    dataver: 1,
};

pub static RedisModuleEvent_Shutdown: RedisModuleEvent = RedisModuleEvent {
    id: REDISMODULE_EVENT_SHUTDOWN as u64,
    dataver: 1,
};

pub static RedisModuleEvent_ReplicaChange: RedisModuleEvent = RedisModuleEvent {
    id: REDISMODULE_EVENT_REPLICA_CHANGE as u64,
    dataver: 1,
};

pub static RedisModuleEvent_CronLoop: RedisModuleEvent = RedisModuleEvent {
    id: REDISMODULE_EVENT_CRON_LOOP as u64,
    dataver: 1,
};

pub static RedisModuleEvent_MasterLinkChange: RedisModuleEvent = RedisModuleEvent {
    id: REDISMODULE_EVENT_MASTER_LINK_CHANGE as u64,
    dataver: 1,
};

pub static RedisModuleEvent_ModuleChange: RedisModuleEvent = RedisModuleEvent {
    id: REDISMODULE_EVENT_MODULE_CHANGE as u64,
    dataver: 1,
};

pub static RedisModuleEvent_LoadingProgress: RedisModuleEvent = RedisModuleEvent {
    id: REDISMODULE_EVENT_LOADING_PROGRESS as u64,
    dataver: 1,
};
