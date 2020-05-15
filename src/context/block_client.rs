use super::Context;
use crate::block_client::BlockClient;
use crate::raw;
use crate::string::RStr;
use crate::{FromPtr, GetPtr};

use std::os::raw::c_void;
use std::time::Duration;

impl Context {
    /// Get the key that is ready when the reply callback is called in the context
    /// of a client blocked by `Context::block_client_on_keys`.
    pub fn get_blocked_client_ready_key(&self) -> Option<RStr> {
        let p: *mut raw::RedisModuleString =
            unsafe { raw::RedisModule_GetBlockedClientReadyKey.unwrap()(self.ptr) };
        if p.is_null() {
            None
        } else {
            Some(RStr::from_ptr(p))
        }
    }

    /// This call is similar to `Context::block_client`, however in this case we
    /// don't just block the client, but also ask Redis to unblock it automatically
    /// once certain keys become "ready", that is, contain more data.
    ///
    /// Basically this is similar to what a typical Redis command usually does,
    /// like BLPOP or ZPOPMAX: the client blocks if it cannot be served ASAP,
    /// and later when the key receives new data (a list push for instance), the
    /// client is unblocked and served.
    ///
    /// However in the case of this module API, when the client is unblocked?
    ///
    /// 1. If you block ok a key of a type that has blocking operations associated,
    ///    like a list, a sorted set, a stream, and so forth, the client may be
    ///    unblocked once the relevant key is targeted by an operation that normally
    ///    unblocks the native blocking operations for that type. So if we block
    ///    on a list key, an RPUSH command may unblock our client and so forth.
    /// 2. If you are implementing your native data type, or if you want to add new
    ///    unblocking conditions in addition to "1", you can call the modules API
    ///    `Context::signal_key_as_ready`.
    ///
    /// Anyway we can't be sure if the client should be unblocked just because the
    /// key is signaled as ready: for instance a successive operation may change the
    /// key, or a client in queue before this one can be served, modifying the key
    /// as well and making it empty again. So when a client is blocked with
    /// `Context::block_client_on_keys` the reply callback is not called after
    /// `BlockClient::unlock` is called, but every time a key is signaled as ready:
    /// if the reply callback can serve the client, it returns OK and the client
    /// is unblocked, otherwise it will return ERR and we'll try again later.
    ///
    /// The reply callback can access the key that was signaled as ready by
    /// calling the API `Context::get_blocked_client_ready_key`, that returns
    /// just the string name of the key as a `string::RStr` object.
    ///
    /// Thanks to this system we can setup complex blocking scenarios, like
    /// unblocking a client only if a list contains at least 5 items or other
    /// more fancy logics.
    ///
    /// Note that another difference with `Context::block_client` is that here
    /// we pass the private data directly when blocking the client: it will
    /// be accessible later in the reply callback. Normally when blocking with
    /// `Context::block_client` the private data to reply to the client is
    /// passed when calling `BlockClient::unblock` but here the unblocking
    /// is performed by Redis itself, so we need to have some private data before
    /// hand. The private data is used to store any information about the specific
    /// unblocking operation that you are implementing. Such information will be
    /// freed using the free_privdata callback provided by the user.
    ///
    /// However the reply callback will be able to access the argument vector of
    /// the command, so the private data is often not needed.
    ///
    /// Note: Under normal circumstances `BlockClient::unblock` should not be
    ///       called for clients that are blocked on keys (Either the key will
    ///       become ready or a timeout will occur). If for some reason you do want
    ///       to call `BlockClient::unblock` it is possible: Client will be
    ///       handled as if it were timed-out (You must implement the timeout
    ///       callback in that case).
    ///
    pub fn block_client_on_keys<T>(
        &self,
        reply_callbck: raw::RedisModuleCmdFunc,
        timeout_callback: raw::RedisModuleCmdFunc,
        free_privdata: raw::FreePrivateDataFunc,
        timeout_ms: Duration,
        keys: &[&RStr],
        privdata: T,
    ) -> Option<BlockClient> {
        let mut keys: Vec<*mut raw::RedisModuleString> = keys.iter().map(|s| s.get_ptr()).collect();

        let data = Box::into_raw(Box::from(privdata));

        let bc: *mut raw::RedisModuleBlockedClient = unsafe {
            raw::RedisModule_BlockClientOnKeys.unwrap()(
                self.ptr,
                reply_callbck,
                timeout_callback,
                free_privdata,
                timeout_ms.as_millis() as i64,
                keys.as_mut_ptr(),
                keys.len() as i32,
                data as *mut c_void,
            )
        };
        if bc.is_null() {
            None
        } else {
            Some(BlockClient::from_ptr(bc))
        }
    }
    /// Block a client in the context of a blocking command, returning an handle
    /// which will be used, later, in order to unblock the client with a call to
    /// `BlockClient::unblock`. The arguments specify callback functions
    /// and a timeout after which the client is unblocked.
    ///
    /// The callbacks are called in the following contexts:
    ///     reply_callback:  called after a successful `BlockClient.unblock`
    ///                      call in order to reply to the client and unblock it.
    ///     reply_timeout:   called when the timeout is reached in order to send an
    ///                      error to the client.
    ///     free_privdata:   called in order to free the private data that is passed
    ///                      by `BlockClient.unblock` call.
    /// Note: `BlockClient.unblock` should be called for every blocked client,
    ///       even if client was killed, timed-out or disconnected. Failing to do so
    ///       will result in memory leaks.
    pub fn block_client(
        &self,
        reply_callbck: raw::RedisModuleCmdFunc,
        timeout_callback: raw::RedisModuleCmdFunc,
        free_privdata: raw::FreePrivateDataFunc,
        timeout_ms: Duration,
    ) -> Option<BlockClient> {
        let bc: *mut raw::RedisModuleBlockedClient = unsafe {
            raw::RedisModule_BlockClient.unwrap()(
                self.ptr,
                reply_callbck,
                timeout_callback,
                free_privdata,
                timeout_ms.as_millis() as i64,
            )
        };
        if bc.is_null() {
            None
        } else {
            Some(BlockClient::from_ptr(bc))
        }
    }
    /// Return false if a module command was called in order to fill the
    /// reply for a blocked client.
    pub fn is_blocked_reply_request(&self) -> bool {
        let ret = unsafe { raw::RedisModule_IsBlockedReplyRequest.unwrap()(self.ptr) };
        ret != 0
    }
    ///  Return false if a module command was called in order to fill the
    /// reply for a blocked client that timed out.
    pub fn is_blocked_timeout_request(&self) -> bool {
        let ret = unsafe { raw::RedisModule_IsBlockedTimeoutRequest.unwrap()(self.ptr) };
        ret != 0
    }
    /// Get the private data set by `BlockClient.unlock`
    pub fn get_block_client_private_data<T>(&self) -> &mut T {
        let data: *mut c_void =
            unsafe { raw::RedisModule_GetBlockedClientPrivateData.unwrap()(self.ptr) };
        unsafe { &mut *(data as *mut T) }
    }
    /// Get the blocked client associated with a given context.
    /// This is useful in the reply and timeout callbacks of blocked clients,
    /// before sometimes the module has the blocked client handle references
    /// around, and wants to cleanup it
    pub fn get_block_client_handle(&self) -> Option<BlockClient> {
        let bc: *mut raw::RedisModuleBlockedClient =
            unsafe { raw::RedisModule_GetBlockedClientHandle.unwrap()(self.ptr) };
        if bc.is_null() {
            None
        } else {
            Some(BlockClient::from_ptr(bc))
        }
    }
    /// Return true if when the free callback of a blocked client is called,
    /// the reason for the client to be unblocked is that it disconnected
    /// while it was blocked.
    pub fn blocked_client_disconnected(&self) -> bool {
        let ret = unsafe { raw::RedisModule_BlockedClientDisconnected.unwrap()(self.ptr) };
        ret != 0
    }
}
