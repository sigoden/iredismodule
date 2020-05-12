use crate::raw;
use crate::{Context, Error, handle_status};
use std::os::raw::{c_int, c_char};

impl Context {
    pub fn subscribe_to_keyspace_events(
        &self,
        types: i32,
        callback: unsafe extern "C" fn(
            ctx: *mut raw::RedisModuleCtx,
            type_: c_int,
            event: *const c_char,
            key: *mut raw::RedisModuleString,
        ) -> c_int,
    ) -> Result<(), Error> {
        handle_status(
            unsafe {
                raw::RedisModule_SubscribeToKeyspaceEvents.unwrap()(self.inner, types, Some(callback))
            },
            "fail to subscribe to keyspace events",
        )
    }

    pub fn subscribe_to_server_event(
        &self,
        events: ServerEvent,
        callback: unsafe extern "C" fn(
            ctx: *mut raw::RedisModuleCtx,
            eid: raw::RedisModuleEvent,
            subevent: u64,
            data: *mut ::std::os::raw::c_void,
        )
    ) -> Result<(), Error> {
        handle_status(
            unsafe {
                raw::RedisModule_SubscribeToServerEvent.unwrap()(self.inner, events.into(), Some(callback))
            },
            "fail to subscribe to keyspace events",
        )
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