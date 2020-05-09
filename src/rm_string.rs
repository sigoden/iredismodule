use crate::raw;
use crate::{handle_status, Error, Ptr};

use std::ops::Deref;
use std::ffi::CString;
use std::fmt;

use std::slice;
use std::str;

pub struct RedisString {
    redis_str: RedisStr,
    ctx: Option<*mut raw::RedisModuleCtx>,
}

impl RedisString {
    pub fn new(ctx: *mut raw::RedisModuleCtx, inner: *mut raw::RedisModuleString) -> RedisString {
        let redis_str = RedisStr::from_ptr(inner);
        RedisString { ctx: Some(ctx), redis_str }
    }
    pub fn have_ctx(&self) -> bool {
        self.ctx.is_some()
    }
    pub fn set_ctx(&mut self, ctx: *mut raw::RedisModuleCtx) {
        self.ctx = Some(ctx);
    }
    pub fn from_str(ctx: *mut raw::RedisModuleCtx, value: &str) -> RedisString {
        let str = CString::new(value).unwrap();
        let inner =
            unsafe { raw::RedisModule_CreateString.unwrap()(ctx, str.as_ptr(), value.len()) };
        Self::new(ctx, inner)
    }
    pub fn ptr_to_str<'a>(ptr: *const raw::RedisModuleString) -> Result<&'a str, Error> {
        let mut len = 0;
        let bytes = unsafe { raw::RedisModule_StringPtrLen.unwrap()(ptr, &mut len) };

        str::from_utf8(unsafe { slice::from_raw_parts(bytes as *const u8, len) })
            .map_err(|e| e.into())
    }
    pub fn append(&mut self, s: &str) -> Result<(), Error> {
        let ctx = self.ctx.unwrap();
        handle_status(
            unsafe {
                raw::RedisModule_StringAppendBuffer.unwrap()(
                    ctx,
                    self.redis_str.inner,
                    s.as_ptr() as *mut i8,
                    s.len(),
                )
                .into()
            },
            "Could not append buffer",
        )
    }
    pub fn len(&self) -> usize {
        let mut len = 0;
        unsafe { raw::RedisModule_StringPtrLen.unwrap()(self.inner, &mut len) };
        len
    }
    fn from_redis_str(redis_str: RedisStr) -> Self {
        RedisString { redis_str, ctx: None }
    }
}

impl Clone for RedisString {
    fn clone(&self) -> Self {
        let ctx = self.ctx.unwrap();
        Self::new(ctx, self.inner)
    }
}

impl Drop for RedisString {
    fn drop(&mut self) {
        let ctx = self.ctx.unwrap();
        unsafe {
            raw::RedisModule_FreeString.unwrap()(ctx, self.inner);
        }
    }
}

impl Deref for RedisString {
    type Target = RedisStr;
    fn deref(&self) -> &Self::Target {
        &self.redis_str
    }
}

impl AsRef<str> for RedisString {
    fn as_ref(&self) -> &str {
        RedisString::ptr_to_str(self.inner).unwrap()
    }
}

impl From<RedisStr> for RedisString {
    fn from(v: RedisStr) -> Self {
        Self::from_redis_str(v)
    }
}

impl AsRef<RedisStr> for RedisString {
    fn as_ref(&self) -> &RedisStr {
        self
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
            "Cloud not get value",
        )?;
        Ok(ll)
    }
    pub fn get_double(&self) -> Result<f64, Error> {
        let mut d: f64 = 0.0;
        handle_status(
            unsafe { raw::RedisModule_StringToDouble.unwrap()(self.inner, &mut d) },
            "Cloud not get value",
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
