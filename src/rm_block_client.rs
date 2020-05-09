use crate::raw;
use crate::{handle_status, RedisCtx, RedisError, Ptr};

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
    pub fn unblock<T>(&self, _data: T) -> Result<(), RedisError> {
        unimplemented!()
    }
    pub fn abort(&self) -> Result<(), RedisError> {
        handle_status(
            unsafe { raw::RedisModule_AbortBlock.unwrap()(self.inner) },
            "can not abort block client",
        )
    }
    pub fn set_disconnect_callback<F, T>(&self, _callback: F)
    where
        F: FnOnce(&RedisCtx, T),
    {
        unimplemented!()
    }
    pub fn get_thread_save_context(&self) {
        unimplemented!()
    }
    pub fn disconnected(&self) -> bool {
        unimplemented!()
    }
    pub fn private_data<T>(&self) -> T {
        unimplemented!()
    }
}
