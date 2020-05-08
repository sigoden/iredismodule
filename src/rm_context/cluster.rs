use crate::raw;
use crate::{Context, Error};

impl Context {
    pub fn get_cluster_nodes_list() -> Option<ClusterNodeList> {
        unimplemented!()
    }
    pub fn set_cluster_flags(&self, flags: u64) {
        unsafe { raw::RedisModule_SetClusterFlags.unwrap()(self.inner, flags) }
    }
    pub fn register_cluster_message_receiver<F>(&self, _msg_type: MsgType, _callback: F) {
        unimplemented!()
    }
    pub fn send_cluster_message(
        &self,
        _target_id: Option<ClusterNode>,
        _msg_type: MsgType,
        _msg: &str,
    ) -> Result<(), Error> {
        unimplemented!()
    }
}

pub struct ClusterNodeList {}

pub type ClusterNode = String;
pub type MsgType = u8;

impl Drop for ClusterNodeList {
    fn drop(&mut self) {
        // unsafe { raw::RedisModule_FreeClusterNodesList().unwrap()(self.inner) }
    }
}
