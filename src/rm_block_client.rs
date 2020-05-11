use crate::raw;
use crate::{handle_status, MutexContext, Error, Ptr};
use std::os::raw::c_void;

#[repr(C)]
pub struct BlockClient {
    inner: *mut raw::RedisModuleBlockedClient,
}

impl Ptr for BlockClient {
    type PtrType = raw::RedisModuleBlockedClient;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.inner
    }
}

impl BlockClient {
    pub fn from_ptr(inner: *mut raw::RedisModuleBlockedClient) -> BlockClient {
        BlockClient { inner }
    }
    pub fn unblock<T>(&self, privdata: T) -> Result<(), Error> {
        let data = Box::into_raw(Box::from(privdata));
        handle_status(
            unsafe { raw::RedisModule_UnblockClient.unwrap()(self.inner, data as *mut c_void) },
            "can not unblock the blockclient",
        )
    }
    pub fn abort(&self) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_AbortBlock.unwrap()(self.inner) },
            "can not abort the blockclient",
        )
    }
    pub fn set_disconnect_callback(&mut self, callback: raw::RedisModuleDisconnectFunc)
    {
        unsafe { raw::RedisModule_SetDisconnectCallback.unwrap()(self.inner, callback) }
    }
    pub fn get_threadsafe_context(&self) -> MutexContext {
        let ctx: *mut raw::RedisModuleCtx = unsafe {
            raw::RedisModule_GetThreadSafeContext.unwrap()(self.inner)
        };
        MutexContext::from_ptr(ctx)
    }
}
