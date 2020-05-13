use crate::{raw, Ptr};

pub struct ScanCursor {
    ptr: *mut raw::RedisModuleScanCursor,
}

impl ScanCursor {
    pub fn new() -> Self {
        ScanCursor {
            ptr: unsafe { raw::RedisModule_ScanCursorCreate.unwrap()() }
        }
    }
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