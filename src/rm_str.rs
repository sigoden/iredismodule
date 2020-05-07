use crate::raw;
use crate::{handle_status, Error};
use std::ffi::CString;
use std::fmt;
use std::ptr;
use std::slice;
use std::str;

#[derive(Debug)]
pub struct Str {
    pub inner: *mut raw::RedisModuleString,
    ctx: Option<*mut raw::RedisModuleCtx>,
}

impl Str {
    pub fn new(inner: *mut raw::RedisModuleString) -> Str {
        Str { ctx: None, inner }
    }
    pub fn create(ctx: *mut raw::RedisModuleCtx, s: &str) -> Str {
        let str = CString::new(s).unwrap();
        let inner = unsafe { raw::RedisModule_CreateString.unwrap()(ctx, str.as_ptr(), s.len()) };

        Str {
            ctx: Some(ctx),
            inner,
        }
    }
    pub fn append(&mut self, s: &str) -> Result<(), Error> {
        self.ensure_ctx()?;
        handle_status(
            unsafe {
                raw::RedisModule_StringAppendBuffer.unwrap()(
                    self.ctx.unwrap(),
                    self.inner,
                    s.as_ptr() as *mut i8,
                    s.len(),
                )
                .into()
            },
            "Could not append buffer",
        )
    }
    pub fn ensure_ctx(&self) -> Result<(), Error> {
        match self.ctx {
            Some(_) => Ok(()),
            None => Err(Error::generic("Nacked str")),
        }
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
}

impl Drop for Str {
    fn drop(&mut self) {
        if self.ensure_ctx().is_ok() {
            unsafe {
                raw::RedisModule_FreeString.unwrap()(self.ctx.unwrap(), self.inner);
            }
        }
    }
}

impl fmt::Display for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let buffer = self.get_buffer();
        let value = str::from_utf8(&buffer).map_err(|_| fmt::Error)?;
        write!(f, "{}", value)
    }
}
