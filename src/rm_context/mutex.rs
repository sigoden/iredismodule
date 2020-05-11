use crate::raw;
use crate::{Context, Ptr};

pub struct MutexContext {
    inner: Context,
}

impl MutexContext {
    pub fn from_ptr(ctx: *mut raw::RedisModuleCtx) -> Self {
        MutexContext { inner: Context::from_ptr(ctx) }
    }
    pub fn lock(&mut self) {
        unsafe {
            raw::RedisModule_ThreadSafeContextLock.unwrap()(self.inner.get_ptr());
         }
    }
    pub fn unlock(&mut self) {
        unsafe {
            raw::RedisModule_ThreadSafeContextUnlock.unwrap()(self.inner.get_ptr());
        }
    }
}

impl Drop for MutexContext {
    fn drop(&mut self) {
        unsafe {
            raw::RedisModule_FreeThreadSafeContext.unwrap()(self.inner.get_ptr());
        }
    }
}