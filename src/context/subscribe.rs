use super::Context;
use crate::raw;
use crate::{handle_status, Ptr, ServerEvent};
use crate::error::Error;
use crate::string::RStr;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};

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
                raw::RedisModule_SubscribeToKeyspaceEvents.unwrap()(
                    self.ptr,
                    types,
                    Some(callback),
                )
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
        ),
    ) -> Result<(), Error> {
        handle_status(
            unsafe {
                raw::RedisModule_SubscribeToServerEvent.unwrap()(
                    self.ptr,
                    events.into(),
                    Some(callback),
                )
            },
            "fail to subscribe to keyspace events",
        )
    }

    pub fn notify_keyspace_event(&self, type_: i32, event: &str, key: &RStr) -> Result<(), Error> {
        let event = CString::new(event).unwrap();
        handle_status(
            unsafe { raw::RedisModule_NotifyKeyspaceEvent.unwrap()( self.ptr, type_, event.as_ptr(), key.get_ptr()) },
            "fail to notify keyspace event"
        )
    }
}
