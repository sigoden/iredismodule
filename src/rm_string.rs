use crate::raw;
use crate::{handle_status, Error};
use std::ffi::CString;
use std::fmt;
use std::convert::TryInto;

use std::slice;
use std::str;

#[derive(Debug)]
pub struct RedisString {
    pub inner: *mut raw::RedisModuleString,
    ctx: *mut raw::RedisModuleCtx,
}

impl RedisString {
    pub fn create(ctx: *mut raw::RedisModuleCtx, value: &str) -> RedisString {
        let str = CString::new(value).unwrap();
        let inner = unsafe { raw::RedisModule_CreateString.unwrap()(ctx, str.as_ptr(), value.len()) };

        RedisString {
            ctx,
            inner,
        }
    }
    pub fn new(ctx: *mut raw::RedisModuleCtx, inner: *mut raw::RedisModuleString) -> RedisString {
        RedisString {
            ctx,
            inner,
        }
    }
    pub fn ptr_to_str<'a>(ptr: *const raw::RedisModuleString) -> Result<&'a str, Error> {
        let mut len = 0;
        let bytes = unsafe { raw::RedisModule_StringPtrLen.unwrap()(ptr, &mut len) };

        str::from_utf8(unsafe { slice::from_raw_parts(bytes as *const u8, len) }).map_err(|e| e.into())
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

    pub fn len(&self) -> usize {
        let mut len = 0;
        unsafe { raw::RedisModule_StringPtrLen.unwrap()(self.inner, &mut len) };
        len
    }
}

impl Clone for RedisString {
    fn clone(&self) -> Self {
        Self::new(self.ctx, self.inner)
    }
}

impl Drop for RedisString {
    fn drop(&mut self) {
        unsafe {
            raw::RedisModule_FreeString.unwrap()(self.ctx, self.inner);
        }
    }
}

impl AsRef<str> for RedisString {
    fn as_ref(&self) -> &str {
        RedisString::ptr_to_str(self.inner).unwrap()
    }
}

impl fmt::Display for RedisString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut len = 0;
        let bytes = unsafe { raw::RedisModule_StringPtrLen.unwrap()(self.inner, &mut len) };
        let value = str::from_utf8(unsafe { slice::from_raw_parts(bytes as *const u8, len) }).map_err(|e| fmt::Error)?;
        write!(f, "{}", value)
    }
}
