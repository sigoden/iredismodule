//! Implementation a specical version of context with lock and unlock functionality
use super::Context;
use crate::raw;
use crate::{FromPtr, GetPtr};
use std::ops::Deref;

/// Wrap and extend `context::Context` with lock and unlock functionality
pub struct MutexContext {
    ctx: Context,
}

impl GetPtr for MutexContext {
    type PtrType = raw::RedisModuleCtx;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.ctx.get_ptr()
    }
}

impl FromPtr for MutexContext {
    type PtrType = raw::RedisModuleCtx;
    fn from_ptr(ptr: *mut Self::PtrType) -> Self {
        MutexContext {
            ctx: Context::from_ptr(ptr)
        }
    }
}

impl MutexContext {
    /// Acquire the server lock before executing a thread safe API call.
    /// This is not needed for `Context::reply` calls when there is
    /// a blocked client connected to the thread safe context.
    pub fn lock(&mut self) {
        unsafe {
            raw::RedisModule_ThreadSafeContextLock.unwrap()(self.ctx.get_ptr());
        }
    }
    /// Release the server lock after a thread safe API call was executed.
    pub fn unlock(&mut self) {
        unsafe {
            raw::RedisModule_ThreadSafeContextUnlock.unwrap()(self.ctx.get_ptr());
        }
    }
}

impl Deref for MutexContext {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl Drop for MutexContext {
    fn drop(&mut self) {
        unsafe {
            raw::RedisModule_FreeThreadSafeContext.unwrap()(self.ctx.get_ptr());
        }
    }
}
