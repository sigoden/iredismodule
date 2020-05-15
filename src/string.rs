//! Represent Redis module string
use crate::raw;
use crate::{handle_status, FromPtr, GetPtr};

use crate::error::Error;
use std::ffi::CString;
use std::ops::Deref;
use std::os::raw::c_char;
use std::str;

/// Repersent module owned RedisModuleString
pub struct RString {
    rstr: RStr,
}

impl FromPtr for RString {
    type PtrType = raw::RedisModuleString;
    fn from_ptr(ptr: *mut raw::RedisModuleString) -> RString {
        let rstr = RStr::from_ptr(ptr);
        RString { rstr }
    }
}

impl RString {
    /// Generate RString from str
    pub fn from_str<T: AsRef<str>>(value: T) -> RString {
        let value_ = CString::new(value.as_ref()).unwrap();
        let ptr = unsafe {
            raw::RedisModule_CreateString.unwrap()(
                0 as *mut raw::RedisModuleCtx,
                value_.as_ptr(),
                value.as_ref().len(),
            )
        };
        Self::from_ptr(ptr)
    }
    /// Get RStr repersentation
    pub fn get_rstr(&self) -> &RStr {
        &self.rstr
    }
    /// Append the specified buffer to the string 'str'.
    pub fn append(&mut self, s: &str) -> Result<(), Error> {
        handle_status(
            unsafe {
                raw::RedisModule_StringAppendBuffer.unwrap()(
                    0 as *mut raw::RedisModuleCtx,
                    self.rstr.ptr,
                    s.as_ptr() as *mut c_char,
                    s.len(),
                )
                .into()
            },
            "fail to append buffer",
        )
    }
    pub fn len(&self) -> usize {
        let mut len = 0;
        unsafe { raw::RedisModule_StringPtrLen.unwrap()(self.ptr, &mut len) };
        len
    }
}

impl Deref for RString {
    type Target = RStr;
    fn deref(&self) -> &Self::Target {
        &self.rstr
    }
}

impl std::fmt::Display for RString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.to_str().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", value)
    }
}

impl Drop for RString {
    fn drop(&mut self) {
        unsafe { raw::RedisModule_FreeString.unwrap()(0 as *mut raw::RedisModuleCtx, self.ptr) }
    }
}

/// Repersent non-owned RedisModuleString
#[repr(C)]
pub struct RStr {
    ptr: *mut raw::RedisModuleString,
}

impl GetPtr for RStr {
    type PtrType = raw::RedisModuleString;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.ptr
    }
}

impl FromPtr for RStr {
    type PtrType = raw::RedisModuleString;
    fn from_ptr(ptr: *mut raw::RedisModuleString) -> RStr {
        RStr { ptr }
    }
}

impl AsRef<str> for RStr {
    fn as_ref(&self) -> &str {
        self.to_str().unwrap()
    }
}

impl AsRef<[u8]> for RStr {
    fn as_ref(&self) -> &[u8] {
        self.to_str().unwrap().as_ref()
    }
}

impl RStr {
    pub fn get_integer(&self) -> Result<i64, Error> {
        let mut ll: i64 = 0;
        handle_status(
            unsafe { raw::RedisModule_StringToLongLong.unwrap()(self.ptr, &mut ll) },
            "fail to get integer",
        )?;
        Ok(ll)
    }
    pub fn assert_integer(&self, check: fn(i64) -> bool) -> Result<u64, Error> {
        let ll = self.get_integer()?;
        if !check(ll) {
            return Err(Error::new("fail to get integer"));
        }
        Ok(ll as u64)
    }
    pub fn get_double(&self) -> Result<f64, Error> {
        let mut d: f64 = 0.0;
        handle_status(
            unsafe { raw::RedisModule_StringToDouble.unwrap()(self.ptr, &mut d) },
            "fail to get double",
        )?;
        Ok(d)
    }
    pub fn get_buffer(&self) -> &[u8] {
        let mut len = 0;
        let bytes = unsafe { raw::RedisModule_StringPtrLen.unwrap()(self.ptr, &mut len) };
        unsafe { std::slice::from_raw_parts(bytes as *const u8, len) }
    }
    pub fn to_str(&self) -> Result<&str, Error> {
        let buffer = self.get_buffer();
        Ok(std::str::from_utf8(&buffer)?)
    }
}

impl std::fmt::Display for RStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.to_str().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", value)
    }
}
