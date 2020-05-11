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
            "can not subscribe to keyspace events",
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
            "can not subscribe to keyspace events",
        )
    }
}

pub enum ServerEvent {
    ReplicationRoleChanged,
    Persistence,
    FlushDB,
    Loading,
    ClientChange,
    Shutdown,
    ReplicaChange,
    CronLoop,
    MasterLinkChange,
    ModuleChange,
    LoadingProgress,
}

impl Into<raw::RedisModuleEvent> for ServerEvent {
    fn into(self) -> raw::RedisModuleEvent {
        unsafe {
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
}