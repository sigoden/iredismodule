use crate::raw;
use crate::{Context, Error, handle_status};
use std::os::raw::{c_int, c_char};

impl Context {
    pub fn subscribe_to_keyspace_events<F>(
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

    pub fn subscribe_to_server_event<F>(
        &self,
        events: raw::RedisModuleEvent,
        callback: unsafe extern "C" fn(
            ctx: *mut raw::RedisModuleCtx,
            eid: raw::RedisModuleEvent,
            subevent: u64,
            data: *mut ::std::os::raw::c_void,
        )
    ) -> Result<(), Error> {
        handle_status(
            unsafe {
                raw::RedisModule_SubscribeToServerEvent.unwrap()(self.inner, events, Some(callback))
            },
            "can not subscribe to keyspace events",
        )
    }
}