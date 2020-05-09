use crate::raw;
use crate::{handle_status, Error, Ptr};

use std::ops::Deref;
use std::ffi::CString;
use std::os::raw::c_char;
use std::fmt;

use std::slice;
use std::str;

pub struct RedisString {
    redis_str: RedisStr,
    ctx: *mut raw::RedisModuleCtx,
}

impl RedisString {
    pub fn new(ctx: *mut raw::RedisModuleCtx, inner: *mut raw::RedisModuleString) -> RedisString {
        let redis_str = RedisStr::from_ptr(inner);
        RedisString { ctx, redis_str }
    }
    pub fn from_str(ctx: *mut raw::RedisModuleCtx, value: &str) -> RedisString {
        let str = CString::new(value).unwrap();
        let inner =
            unsafe { raw::RedisModule_CreateString.unwrap()(ctx, str.as_ptr(), value.len()) };
        Self::new(ctx, inner)
    }
    pub unsafe fn from_raw_parts(ctx: *mut raw::RedisModuleCtx, data: *mut u8, len: usize) -> RedisString {
        let value = std::slice::from_raw_parts(data, len);
        let str = CString::new(value).unwrap();
        let inner = raw::RedisModule_CreateString.unwrap()(ctx, str.as_ptr(), len);
        Self::new(ctx, inner)
    }
    pub fn get_redis_str(&self) -> &RedisStr {
        &self.redis_str
    }
    pub fn ptr_to_str<'a>(ptr: *const raw::RedisModuleString) -> Result<&'a str, Error> {
        let mut len = 0;
        let bytes = unsafe { raw::RedisModule_StringPtrLen.unwrap()(ptr, &mut len) };

        str::from_utf8(unsafe { slice::from_raw_parts(bytes as *const u8, len) })
            .map_err(|e| e.into())
    }
    pub fn append(&mut self, s: &str) -> Result<(), Error> {
        handle_status(
            unsafe {
                raw::RedisModule_StringAppendBuffer.unwrap()(
                    self.ctx,
                    self.redis_str.inner,
                    s.as_ptr() as *mut c_char,
                    s.len(),
                )
                .into()
            },
            "can not append buffer",
        )
    }
    pub fn len(&self) -> usize {
        let mut len = 0;
        unsafe { raw::RedisModule_StringPtrLen.unwrap()(self.inner, &mut len) };
        len
    }
}

impl Deref for RedisString {
    type Target = RedisStr;
    fn deref(&self) -> &Self::Target {
        &self.redis_str
    }
}

impl fmt::Display for RedisString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut len = 0;
        let bytes = unsafe { raw::RedisModule_StringPtrLen.unwrap()(self.inner, &mut len) };
        let value = str::from_utf8(unsafe { slice::from_raw_parts(bytes as *const u8, len) })
            .map_err(|_e| fmt::Error)?;
        write!(f, "{}", value)
    }
}

#[repr(C)]
pub struct RedisStr {
    inner: *mut raw::RedisModuleString,
}

impl Ptr for RedisStr {
    type PtrType = raw::RedisModuleString;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.inner
    }
}

impl RedisStr {
    pub fn from_ptr(inner: *mut raw::RedisModuleString) -> Self {
        RedisStr { inner }
    }

    pub fn get_ptr(&self) -> *mut raw::RedisModuleString {
        self.inner
    }

    pub fn get_longlong(&self) -> Result<i64, Error> {
        let mut ll: i64 = 0;
        handle_status(
            unsafe { raw::RedisModule_StringToLongLong.unwrap()(self.inner, &mut ll) },
            "can not get longlong",
        )?;
        Ok(ll)
    }
    pub fn get_positive_integer(&self) -> Result<u64, Error> {
        let ll = self.get_longlong()?;
        if ll < 1 {
            return Err(Error::generic("can not less than 1"));
        }
        Ok(ll as u64)
    }
    pub fn get_double(&self) -> Result<f64, Error> {
        let mut d: f64 = 0.0;
        handle_status(
            unsafe { raw::RedisModule_StringToDouble.unwrap()(self.inner, &mut d) },
            "can not get double",
        )?;
        Ok(d)
    }
    pub fn get_buffer(&self) -> &[u8] {
        let mut len = 0;
        let bytes = unsafe { raw::RedisModule_StringPtrLen.unwrap()(self.inner, &mut len) };
        unsafe { slice::from_raw_parts(bytes as *const u8, len) }
    }
    pub fn to_str(&self) -> Result<&str, Error> {
        let buffer = self.get_buffer();
        Ok(str::from_utf8(&buffer)?)
    }
}
