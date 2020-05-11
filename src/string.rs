use crate::raw;
use crate::{handle_status, Error, Ptr};

use std::ffi::CString;
use std::ops::Deref;
use std::os::raw::c_char;

use std::slice;
use std::str;

pub struct RString {
    redis_str: RStr,
    ctx: *mut raw::RedisModuleCtx,
}

impl RString {
    pub fn new(ctx: *mut raw::RedisModuleCtx, inner: *mut raw::RedisModuleString) -> RString {
        let redis_str = RStr::from_ptr(inner);
        RString { ctx, redis_str }
    }
    pub fn from_str(ctx: *mut raw::RedisModuleCtx, value: &str) -> RString {
        let str = CString::new(value).unwrap();
        let inner =
            unsafe { raw::RedisModule_CreateString.unwrap()(ctx, str.as_ptr(), value.len()) };
        Self::new(ctx, inner)
    }
    pub unsafe fn from_raw_parts(
        ctx: *mut raw::RedisModuleCtx,
        data: *mut u8,
        len: usize,
    ) -> RString {
        let value = std::slice::from_raw_parts(data, len);
        let str = CString::new(value).unwrap();
        let inner = raw::RedisModule_CreateString.unwrap()(ctx, str.as_ptr(), len);
        Self::new(ctx, inner)
    }
    pub fn get_redis_str(&self) -> &RStr {
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

impl Deref for RString {
    type Target = RStr;
    fn deref(&self) -> &Self::Target {
        &self.redis_str
    }
}

#[repr(C)]
pub struct RStr {
    inner: *mut raw::RedisModuleString,
}

impl Ptr for RStr {
    type PtrType = raw::RedisModuleString;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.inner
    }
}

impl RStr {
    pub fn from_ptr(inner: *mut raw::RedisModuleString) -> Self {
        RStr { inner }
    }

    pub fn get_ptr(&self) -> *mut raw::RedisModuleString {
        self.inner
    }

    pub fn get_integer(&self) -> Result<i64, Error> {
        let mut ll: i64 = 0;
        handle_status(
            unsafe { raw::RedisModule_StringToLongLong.unwrap()(self.inner, &mut ll) },
            "can not get integer",
        )?;
        Ok(ll)
    }
    pub fn get_integer_which(&self, which: fn(i64) -> bool) -> Result<u64, Error> {
        let ll = self.get_integer()?;
        if !which(ll) {
            return Err(Error::generic("can not get integer"));
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
