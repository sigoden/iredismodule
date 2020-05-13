//! Cluster related structs and functions

use crate::error::Error;
use crate::raw;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub type MsgType = u8;

#[derive(Debug)]
pub struct ClusterNodeInfo {
    pub ip: String,
    pub master_id: String,
    pub port: i32,
    pub flags: i32,
}

pub struct ClusterNodeList {
    data: Vec<CString>,
    ptr: *mut *mut c_char,
}

impl ClusterNodeList {
    pub fn new(ptr: *mut *mut c_char, len: usize) -> ClusterNodeList {
        let data = unsafe {
            Vec::from_raw_parts(ptr, len, len)
                .into_iter()
                .map(|v| CString::from_raw(v))
                .collect()
        };
        ClusterNodeList { data, ptr }
    }
    pub fn value(&self) -> Vec<&str> {
        self.data.iter().map(|v| v.to_str().unwrap()).collect()
    }
}

impl Drop for ClusterNodeList {
    fn drop(&mut self) {
        unsafe { raw::RedisModule_FreeClusterNodesList.unwrap()(self.ptr) }
    }
}

/// wrap RedisModule_GetMyClusterID
pub fn get_my_cluster_id() -> Result<String, Error> {
    let c_buf: *const c_char = unsafe { raw::RedisModule_GetMyClusterID.unwrap()() };
    if c_buf.is_null() {
        Err(Error::new("cluster is disabled"))
    } else {
        let c_str: &CStr = unsafe { CStr::from_ptr(c_buf) };
        Ok(c_str.to_str()?.to_owned())
    }
}

/// wrap RedisModule_GetClusterSize
pub fn get_cluster_size() -> usize {
    unsafe { raw::RedisModule_GetClusterSize.unwrap()() }
}
