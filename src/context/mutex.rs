use crate::raw;
use crate::{Context, Ptr};
use std::ops::Deref;

pub struct MutexContext {
    ctx: Context,
}

impl MutexContext {
    pub fn from_ptr(ctx: *mut raw::RedisModuleCtx) -> Self {
        MutexContext {
            ctx: Context::from_ptr(ctx),
        }
    }
    pub fn lock(&mut self) {
        unsafe {
            raw::RedisModule_ThreadSafeContextLock.unwrap()(self.ctx.get_ptr());
        }
    }
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
