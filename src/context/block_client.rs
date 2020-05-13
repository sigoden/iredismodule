use super::Context;
use crate::string::RStr;
use crate::raw;
use crate::block_client::BlockClient;
use crate::Ptr;
use crate::error::Error;

use std::time::Duration;
use std::os::raw::c_void;

impl Context {
    pub fn get_blocked_client_ready_key(&self) -> Result<RStr, Error> {
        let p: *mut raw::RedisModuleString =
            unsafe { raw::RedisModule_GetBlockedClientReadyKey.unwrap()(self.ptr) };
        if p.is_null() {
            Err(Error::new("fail to get read key"))
        } else {
            Ok(RStr::from_ptr(p))
        }
    }
    pub fn block_client_on_keys<T>(
        &mut self,
        reply_callbck: raw::RedisModuleCmdFunc,
        timeout_callback: raw::RedisModuleCmdFunc,
        free_privdata: raw::FreePrivateDataFunc,
        timeout: Duration,
        keys: &[&RStr],
        privdata: T,
    ) -> BlockClient {
        let mut keys: Vec<*mut raw::RedisModuleString> = keys.iter().map(|s| s.get_ptr()).collect();

        let data = Box::into_raw(Box::from(privdata));

        let bc: *mut raw::RedisModuleBlockedClient = unsafe {
            raw::RedisModule_BlockClientOnKeys.unwrap()(
                self.ptr,
                reply_callbck,
                timeout_callback,
                free_privdata,
                timeout.as_millis() as i64,
                keys.as_mut_ptr(),
                keys.len() as i32,
                data as *mut c_void,
            )
        };
        BlockClient::from_ptr(bc)
    }
    pub fn block_client(
        &mut self,
        reply_callbck: raw::RedisModuleCmdFunc,
        timeout_callback: raw::RedisModuleCmdFunc,
        free_privdata: raw::FreePrivateDataFunc,
        timeout: Duration,
    ) -> BlockClient {
        let bc: *mut raw::RedisModuleBlockedClient = unsafe {
            raw::RedisModule_BlockClient.unwrap()(
                self.ptr,
                reply_callbck,
                timeout_callback,
                free_privdata,
                timeout.as_millis() as i64,
            )
        };
        BlockClient::from_ptr(bc)
    }
    pub fn is_blocked_reply_request(&self) -> bool {
        let ret = unsafe { raw::RedisModule_IsBlockedReplyRequest.unwrap()(self.ptr) };
        ret != 0
    }
    pub fn is_blocked_timeout_request(&self) -> bool {
        let ret = unsafe { raw::RedisModule_IsBlockedTimeoutRequest.unwrap()(self.ptr) };
        ret != 0
    }
    pub fn get_block_client_private_data<T>(&self) -> &mut T {
        let data: *mut c_void =
            unsafe { raw::RedisModule_GetBlockedClientPrivateData.unwrap()(self.ptr) };
        unsafe { &mut *(data as *mut T) }
    }
    pub fn get_block_client_handle(&self) -> BlockClient {
        let bc: *mut raw::RedisModuleBlockedClient =
            unsafe { raw::RedisModule_GetBlockedClientHandle.unwrap()(self.ptr) };
        BlockClient::from_ptr(bc)
    }
    pub fn blocked_client_disconnected(&self) -> bool {
        let ret = unsafe { raw::RedisModule_BlockedClientDisconnected.unwrap()(self.ptr) };
        ret != 0
    }
}
