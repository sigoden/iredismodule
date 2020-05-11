use std::collections::HashSet;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::slice;
use std::time::Duration;

use crate::raw;
use crate::{Error, RStr};

pub trait Ptr {
    type PtrType;
    fn get_ptr(&self) -> *mut Self::PtrType;
}

/// wrap RedisModule_Milliseconds
pub fn milliseconds() -> Duration {
    Duration::from_millis(unsafe { raw::RedisModule_Milliseconds.unwrap()() } as u64)
}

pub fn parse_args<'a>(argv: *mut *mut raw::RedisModuleString, argc: c_int) -> Vec<RStr> {
    unsafe { slice::from_raw_parts(argv, argc as usize) }
        .into_iter()
        .map(|&arg| RStr::from_ptr(arg))
        .collect()
}

pub fn handle_status(status: i32, message: &str) -> Result<(), Error> {
    if status == raw::REDISMODULE_OK as i32 {
        Ok(())
    } else {
        Err(Error::generic(message))
    }
}

pub fn is_module_busy(name: &str) -> Result<(), Error> {
    let name = CString::new(name).unwrap();
    handle_status(
        unsafe { raw::RedisModule_IsModuleNameBusy.unwrap()(name.as_ptr()) },
        "can not check busy",
    )
}

/// wrap RedisModule_GetMyClusterID
pub fn get_my_cluster_id() -> Result<String, Error> {
    let c_buf: *const c_char = unsafe { raw::RedisModule_GetMyClusterID.unwrap()() };
    if c_buf.is_null() {
        Err(Error::generic("Cluster is disabled"))
    } else {
        let c_str: &CStr = unsafe { CStr::from_ptr(c_buf) };
        Ok(c_str.to_str()?.to_owned())
    }
}

/// wrap RedisModule_GetClusterSize
pub fn get_cluster_size() -> usize {
    unsafe { raw::RedisModule_GetClusterSize.unwrap()() }
}

pub enum StatusCode {
    Ok = raw::REDISMODULE_OK as isize,
    Err = raw::REDISMODULE_ERR as isize,
}
impl From<c_int> for StatusCode {
    fn from(v: c_int) -> Self {
        if v == raw::REDISMODULE_OK as c_int {
            StatusCode::Ok
        } else {
            StatusCode::Err
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LogLevel {
    Debug,
    Notice,
    Verbose,
    Warning,
}

impl Into<CString> for LogLevel {
    fn into(self) -> CString {
        CString::new(format!("{:?}", self).to_lowercase()).unwrap()
    }
}

#[derive(Debug)]
pub struct ArgvFlags(HashSet<char>);

impl ArgvFlags {
    pub fn new() -> ArgvFlags {
        let mut s = HashSet::new();
        s.insert('v');
        ArgvFlags(s)
    }
    pub fn replicate(&mut self) -> &mut ArgvFlags {
        self.0.insert('!');
        self
    }
    pub fn no_aof(&mut self) -> &mut ArgvFlags {
        self.0.insert('A');
        self
    }
    pub fn no_replicas(&mut self) -> &mut ArgvFlags {
        self.0.insert('R');
        self
    }
}

impl Into<CString> for ArgvFlags {
    fn into(self) -> CString {
        let v = self.0.into_iter().collect::<String>();
        CString::new(v).unwrap()
    }
}
