use crate::cluster::{ClusterNodeList, MsgType};
use crate::raw;
use crate::{handle_status, Context, Error};
use std::ffi::CString;
use std::os::raw::{c_char, c_uchar};

impl Context {
    pub fn get_cluster_nodes_list(&self) -> Option<ClusterNodeList> {
        let mut len = 0;
        let ptr = unsafe { raw::RedisModule_GetClusterNodesList.unwrap()(self.ptr, &mut len) };
        if ptr.is_null() {
            return None;
        }
        Some(ClusterNodeList::new(ptr, len))
    }
    pub fn set_cluster_flags(&self, flags: u32) {
        unsafe { raw::RedisModule_SetClusterFlags.unwrap()(self.ptr, flags as u64) }
    }
    pub fn register_cluster_message_receiver(
        &self,
        msg_type: MsgType,
        callback: unsafe extern "C" fn(
            ctx: *mut raw::RedisModuleCtx,
            sender_id: *const c_char,
            type_: u8,
            payload: *const c_uchar,
            len: u32,
        ),
    ) {
        unsafe {
            raw::RedisModule_RegisterClusterMessageReceiver.unwrap()(
                self.ptr,
                msg_type,
                Some(callback),
            )
        }
    }
    pub fn send_cluster_message(
        &self,
        target_id: &str,
        msg_type: MsgType,
        msg: &[u8],
    ) -> Result<(), Error> {
        let c_target_id = CString::new(target_id).unwrap();
        let c_msg = CString::new(msg).unwrap();
        handle_status(
            unsafe {
                raw::RedisModule_SendClusterMessage.unwrap()(
                    self.ptr,
                    c_target_id.as_ptr() as *mut c_char,
                    msg_type,
                    c_msg.as_ptr() as *mut c_uchar,
                    msg.len() as u32,
                )
            },
            "fail to send cluster message",
        )
    }
    pub fn send_cluster_message_all(&self, msg_type: MsgType, msg: &[u8]) -> Result<(), Error> {
        let c_msg = CString::new(msg).unwrap();
        handle_status(
            unsafe {
                raw::RedisModule_SendClusterMessage.unwrap()(
                    self.ptr,
                    0 as *mut c_char,
                    msg_type,
                    c_msg.as_ptr() as *mut c_uchar,
                    msg.len() as u32,
                )
            },
            "fail to send cluster message",
        )
    }
}
