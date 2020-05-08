use std::time::Duration;


use crate::{take_data, BlockClient, Context, Error, RedisString};

impl Context {
    pub fn block_client<F, G>(
        &self,
        _reply_callbck: F,
        _timeout_callback: F,
        _free_privdata: G,
        _timeout: Duration,
    ) -> BlockClient
    where
        F: FnOnce(&Context),
    {
        unimplemented!()
    }
    pub fn is_blocked_reply_request(&self) -> bool {
        unimplemented!()
    }
    pub fn is_blocked_timeout_request(&self) -> bool {
        unimplemented!()
    }
    pub fn get_block_client_handle(&self) -> BlockClient {
        unimplemented!()
    }
}
