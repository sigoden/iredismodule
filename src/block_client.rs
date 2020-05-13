use crate::raw;
use crate::{handle_status, Error, MutexContext, Ptr};
use std::os::raw::c_void;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BlockClient {
    ptr: *mut raw::RedisModuleBlockedClient,
}

unsafe impl Send for BlockClient {}
unsafe impl Sync for BlockClient {}

impl Ptr for BlockClient {
    type PtrType = raw::RedisModuleBlockedClient;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.ptr
    }
}

impl BlockClient {
    pub fn from_ptr(ptr: *mut raw::RedisModuleBlockedClient) -> BlockClient {
        BlockClient { ptr }
    }
    pub fn unblock<T>(&self, privdata: Option<T>) -> Result<(), Error> {
        let data = match privdata {
            Some(v) => Box::into_raw(Box::from(v)) as *mut c_void,
            None => 0 as *mut c_void,
        };
        handle_status(
            unsafe { raw::RedisModule_UnblockClient.unwrap()(self.ptr, data) },
            "fail to unblock the blockclient",
        )
    }
    pub fn abort(&self) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_AbortBlock.unwrap()(self.ptr) },
            "fail to abort the blockclient",
        )
    }

    pub fn set_disconnect_callback(
        &self,
        callback: unsafe extern "C" fn(
            *mut raw::RedisModuleCtx,
            *mut raw::RedisModuleBlockedClient,
        ),
    ) {
        unsafe { raw::RedisModule_SetDisconnectCallback.unwrap()(self.ptr, Some(callback)) }
    }
    pub fn get_threadsafe_context(&self) -> MutexContext {
        let ctx: *mut raw::RedisModuleCtx =
            unsafe { raw::RedisModule_GetThreadSafeContext.unwrap()(self.ptr) };
        MutexContext::from_ptr(ctx)
    }
}
