use super::Context;
use crate::cluster::{ClusterNodeInfo, MsgType};
use crate::error::Error;
use crate::handle_status;
use crate::raw;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uchar};

impl Context {
    /// Return an list cluster node id.
    ///
    /// However if this function is called by a module not running an a Redis
    /// instance with Redis Cluster enabled.
    ///
    /// The IDs returned can be used with `Context::get_cluster_node_info` in order
    /// to get more information about single nodes.
    pub fn get_cluster_nodes_list(&self) -> Vec<String> {
        let mut len = 0;
        let ptr = unsafe { raw::RedisModule_GetClusterNodesList.unwrap()(self.ptr, &mut len) };
        if ptr.is_null() {
            return Vec::new();
        }
        let data: &[u8] = unsafe { std::slice::from_raw_parts(*ptr as *const c_uchar, len) };
        let result = data
            .chunks(raw::REDISMODULE_NODE_ID_LEN as usize)
            .map(|v| std::str::from_utf8(v).unwrap().to_owned())
            .collect::<Vec<String>>();
        unsafe { raw::RedisModule_FreeClusterNodesList.unwrap()(ptr) }
        result
    }
    /// Set Redis Cluster flags in order to change the normal behavior of
    /// Redis Cluster, especially with the goal of disabling certain functions.
    /// This is useful for modules that use the Cluster API in order to create
    /// a different distributed system, but still want to use the Redis Cluster
    /// message bus. Flags that can be set:
    ///
    ///  CLUSTER_MODULE_FLAG_NO_FAILOVER
    ///  CLUSTER_MODULE_FLAG_NO_REDIRECTION
    ///
    /// With the following effects:
    ///  NO_FAILOVER: prevent Redis Cluster slaves to failover a failing master.
    ///               Also disables the replica migration feature.
    ///  NO_REDIRECTION: Every node will accept any key, without trying to perform
    ///                  partitioning according to the user Redis Cluster algorithm.
    ///                  Slots informations will still be propagated across the
    ///                  cluster, but without effects.
    pub fn set_cluster_flags(&self, flags: u32) {
        unsafe { raw::RedisModule_SetClusterFlags.unwrap()(self.ptr, flags as u64) }
    }
    /// Get the specified info for the node having as ID the specified 'id'.
    ///
    /// if the node ID does not exist from the POV of this local node, Err will be returned.
    pub fn get_cluster_node_info<T: AsRef<str>>(&self, id: T) -> Result<ClusterNodeInfo, Error> {
        let ip: *mut c_char = std::ptr::null_mut();
        let master_id: *mut c_char = std::ptr::null_mut();
        let port: *mut c_int = std::ptr::null_mut();
        let flags: *mut c_int = std::ptr::null_mut();
        handle_status(
            unsafe {
                raw::RedisModule_GetClusterNodeInfo.unwrap()(
                    self.ptr,
                    id.as_ref().as_ptr() as *const i8,
                    ip,
                    master_id,
                    port,
                    flags,
                )
            },
            "fail to get cluster node info",
        )?;
        Ok(ClusterNodeInfo {
            ip: char_ptr_value(ip, raw::REDISMODULE_NODE_ID_LEN as usize),
            master_id: char_ptr_value(master_id, raw::REDISMODULE_NODE_ID_LEN as usize),
            port: int_ptr_value(port),
            flags: int_ptr_value(flags),
        })
    }
    /// Register a callback receiver for cluster messages of type 'type'. If there
    /// was already a registered callback, this will replace the callback function
    /// with the one provided, otherwise if the callback is set to None and there
    /// is already a callback for this function, the callback is unregistered
    /// (so this API call is also used in order to delete the receiver).
    pub fn register_cluster_message_receiver(
        &self,
        msg_type: MsgType,
        callback: raw::RedisModuleClusterMessageReceiver,
    ) {
        unsafe {
            raw::RedisModule_RegisterClusterMessageReceiver.unwrap()(self.ptr, msg_type, callback)
        }
    }
    /// Send a message to all the nodes in the cluster if `target` is empty, otherwise
    /// at the specified target.
    ///
    /// The function returns Ok if the message was successfully sent,
    /// otherwise if the node is not connected or such node ID does not map to any
    /// known cluster node, Err is returned.
    pub fn send_cluster_message<T: AsRef<[u8]>>(
        &self,
        target_id: &str,
        msg_type: MsgType,
        msg: T,
    ) -> Result<(), Error> {
        let c_msg = CString::new(msg.as_ref()).unwrap();
        let target_id = if target_id.is_empty() {
            0 as *mut c_char
        } else {
            target_id.as_ptr() as *mut c_char
        };
        handle_status(
            unsafe {
                raw::RedisModule_SendClusterMessage.unwrap()(
                    self.ptr,
                    target_id,
                    msg_type,
                    c_msg.as_ptr() as *mut c_uchar,
                    msg.as_ref().len() as u32,
                )
            },
            "fail to send cluster message",
        )
    }
}

fn char_ptr_value(ptr: *mut c_char, len: usize) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    Some(unsafe {
        std::str::from_utf8(std::slice::from_raw_parts(ptr as *const c_uchar, len))
            .unwrap()
            .to_owned()
    })
}

fn int_ptr_value(ptr: *mut c_int) -> Option<i32> {
    if ptr.is_null() {
        return None;
    }
    Some(unsafe { *ptr })
}
