use crate::raw;
use crate::{Error};

pub struct BlockClient {
    pub inner: *mut raw::RedisModuleBlockedClient,
}

impl BlockClient {
    pub fn abort(&self) ->  Result<(), Error> {
        unimplemented!()
    }
    pub fn unblock<T>(&self, data: T) -> Result<(), Error> {
        unimplemented!();
    }
    pub fn set_disconnect_callback<F>(&self, callback: F) {
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

