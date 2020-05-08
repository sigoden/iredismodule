use std::time::Duration;

use crate::raw;
use crate::{Ctx, Error, BlockClient, RedisString, take_data};

impl Ctx {
    pub fn block_client<F, G>(
        &self,
        reply_callbck: F,
        timeout_callback: F,
        free_privdata: G,
        timeout: Duration,
    ) -> BlockClient
    where
        F: FnOnce(&Ctx, )
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
