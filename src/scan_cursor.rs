//! Define ScanCursor struct

use crate::{raw, Ptr};

/// A cursor be used with scan
pub struct ScanCursor {
    ptr: *mut raw::RedisModuleScanCursor,
}

impl ScanCursor {
    pub fn new() -> Self {
        ScanCursor {
            ptr: unsafe { raw::RedisModule_ScanCursorCreate.unwrap()() }
        }
    }
    /// Restart an existing cursor. The keys will be rescanned.
    pub fn restart(&mut self) {
        unsafe { raw::RedisModule_ScanCursorRestart.unwrap()(self.ptr) }
    }
}

impl Ptr for ScanCursor {
    type PtrType = raw::RedisModuleScanCursor;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.ptr
    }
}

impl Drop for ScanCursor {
    fn drop(&mut self) {
        unsafe { raw::RedisModule_ScanCursorDestroy.unwrap()(self.ptr) }
    }
}