use crate::raw;
use crate::{handle_status, Error};
use std::ffi::CString;
use std::fmt;
use std::ops;

use std::slice;
use std::str;

#[derive(Debug)]
pub struct RedisString {
    pub inner: *mut raw::RedisModuleString,
    ctx: *mut raw::RedisModuleCtx,
}

impl RedisString {
    pub fn new(ctx: *mut raw::RedisModuleCtx, s: &str) -> RedisString {
        let str = CString::new(s).unwrap();
        let inner = unsafe { raw::RedisModule_CreateString.unwrap()(ctx, str.as_ptr(), s.len()) };

        RedisString {
            ctx,
            inner,
        }
    }
    pub fn append(&mut self, s: &str) -> Result<(), Error> {
        handle_status(
            unsafe {
                raw::RedisModule_StringAppendBuffer.unwrap()(
                    self.ctx,
                    self.inner,
                    s.as_ptr() as *mut i8,
                    s.len(),
                )
                .into()
            },
            "Could not append buffer",
        )
    }
}

impl Drop for RedisString {
    fn drop(&mut self) {
        unsafe {
            raw::RedisModule_FreeString.unwrap()(self.ctx, self.inner);
        }
    }
}

impl fmt::Display for RedisString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.to_str().map_err(|_| fmt::Error)?;
        write!(f, "{}", value)
    }
}

impl ops::Deref for RedisString {
    type Target = RedisStr;
    #[inline]
    fn deref(&self) -> &RedisStr {
        unsafe { RedisStr::from_ptr(self.inner) }
    }
}

impl AsRef<RedisStr> for RedisString {
    #[inline]
    fn as_ref(&self) -> &RedisStr {
        self
    }
}

pub struct RedisStr {
    pub inner: *const raw::RedisModuleString,
}

impl RedisStr {
    pub unsafe fn from_ptr<'a>(inner: *const raw::RedisModuleString) -> &'a Self {
        &*(inner as *const RedisStr)
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