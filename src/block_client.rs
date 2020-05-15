//! Block client implentation

use crate::context::ThreadSafeContext;
use crate::error::Error;
use crate::raw;
use crate::{handle_status, FromPtr, GetPtr};
use std::os::raw::c_void;

/// Wrap the pointer of a RedisModuleBlockedClient
#[repr(C)]
#[derive(Copy, Clone)]
pub struct BlockClient {
    ptr: *mut raw::RedisModuleBlockedClient,
}

unsafe impl Send for BlockClient {}
unsafe impl Sync for BlockClient {}

impl GetPtr for BlockClient {
    type PtrType = raw::RedisModuleBlockedClient;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.ptr
    }
}

impl FromPtr for BlockClient {
    type PtrType = raw::RedisModuleBlockedClient;
    fn from_ptr(ptr: *mut raw::RedisModuleBlockedClient) -> BlockClient {
        BlockClient { ptr }
    }
}

impl BlockClient {
    /// Unblock a client blocked.
    ///
    /// This will trigger the reply callbacks to be called in order to reply to
    /// the client.The 'privdata' argument will be accessible by the reply callback, so
    /// the caller of this function can pass any value that is needed in order to
    /// actually reply to the client.
    ///
    /// A common usage for 'privdata' is a thread that computes something that
    /// needs to be passed to the client, included but not limited some slow
    /// to compute reply or some reply obtained via networking.
    ///
    /// Note 1: this function can be called from threads spawned by the module.
    ///
    /// Note 2: when we unblock a client that is blocked for keys using
    /// the API `Context::block_client_on_keys`, the privdata argument here is
    /// not used, and the reply callback is called with the privdata pointer that
    /// was passed when blocking the client.
    ///
    /// Unblocking a client that was blocked for keys using this API will still
    /// require the client to get some reply, so the function will use the
    /// "timeout" handler in order to do so.
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
    /// Abort a blocked client blocking operation: the client will be unblocked
    /// without firing any callback.
    pub fn abort(&self) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_AbortBlock.unwrap()(self.ptr) },
            "fail to abort the blockclient",
        )
    }
    /// Set a callback that will be called if a blocked client disconnects
    /// before the module has a chance to call `BlockClient::unblock`
    ///
    /// Usually what you want to do there, is to cleanup your module state
    /// so that you can call `BlockClient::unblock` safely, otherwise
    /// the client will remain blocked forever if the timeout is large.
    ///
    /// Notes:
    ///
    /// 1. It is not safe to call Reply* family functions here, it is also
    ///    useless since the client is gone.
    ///
    /// 2. This callback is not called if the client disconnects because of
    ///    a timeout. In such a case, the client is unblocked automatically
    ///    and the timeout callback is called.
    pub fn set_disconnect_callback(
        &self,
        callback: unsafe extern "C" fn(
            *mut raw::RedisModuleCtx,
            *mut raw::RedisModuleBlockedClient,
        ),
    ) {
        unsafe { raw::RedisModule_SetDisconnectCallback.unwrap()(self.ptr, Some(callback)) }
    }

    /// Return a context which can be used inside threads to make Redis context
    /// calls with certain modules APIs.
    pub fn get_threadsafe_context(&self) -> ThreadSafeContext {
        let ctx: *mut raw::RedisModuleCtx =
            unsafe { raw::RedisModule_GetThreadSafeContext.unwrap()(self.ptr) };
        ThreadSafeContext::from_ptr(ctx)
    }
}
