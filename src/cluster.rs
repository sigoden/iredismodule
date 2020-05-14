//! Cluster related structs and functions

use crate::error::Error;
use crate::raw;
use std::ffi::CStr;
use std::os::raw::c_char;

/// User-defiend cluster msg type
pub type MsgType = u8;

/// Cluster node info
#[derive(Debug)]
pub struct ClusterNodeInfo {
    pub ip: Option<String>,
    pub master_id: Option<String>,
    pub port: Option<i32>,
    /// The list of flags reported is the following:
    ///
    /// * REDISMODULE_NODE_MYSELF        This node
    /// * REDISMODULE_NODE_MASTER        The node is a master
    /// * REDISMODULE_NODE_SLAVE         The node is a replica
    /// * REDISMODULE_NODE_PFAIL         We see the node as failing
    /// * REDISMODULE_NODE_FAIL          The cluster agrees the node is failing
    /// * REDISMODULE_NODE_NOFAILOVER    The slave is configured to never failover
    pub flags: Option<i32>,
}

/// Return this node ID
pub fn get_my_cluster_id() -> Result<String, Error> {
    let c_buf: *const c_char = unsafe { raw::RedisModule_GetMyClusterID.unwrap()() };
    if c_buf.is_null() {
        Err(Error::new("cluster is disabled"))
    } else {
        let c_str: &CStr = unsafe { CStr::from_ptr(c_buf) };
        Ok(c_str.to_str()?.to_owned())
    }
}

/// Return the number of nodes in the cluster, regardless of their state
/// (handshake, noaddress, ...) so that the number of active nodes may actually
/// be smaller, but not greater than this number. If the instance is not in
/// cluster mode, zero is returned.
pub fn get_cluster_size() -> usize {
    unsafe { raw::RedisModule_GetClusterSize.unwrap()() }
}
