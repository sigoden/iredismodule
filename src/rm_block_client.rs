use crate::raw;
use crate::{handle_status, Ctx, Error};


pub struct BlockClient {
    pub inner: *mut raw::RedisModuleBlockedClient,
}

impl BlockClient {
    pub fn unblock<T>(&self, _data: T) -> Result<(), Error> {
        unimplemented!()
    }
    pub fn abort(&self) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_AbortBlock.unwrap()(self.inner) },
            "Cloud not abort block client",
        )
    }
    pub fn set_disconnect_callback<F, T>(&self, _callback: F)
    where
        F: FnOnce(&Ctx, T),
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
