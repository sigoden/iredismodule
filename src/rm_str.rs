use std::ffi::CString;
use std::fmt;
use std::slice;
use std::str;
use crate::raw;
use crate::{Error, handle_status};

#[derive(Debug)]
pub struct Str {
    pub inner: *mut raw::RedisModuleString,
    ctx: *mut raw::RedisModuleCtx,
}

impl Str {
    pub fn create(ctx: *mut raw::RedisModuleCtx, s: &str) -> Str {
        let str = CString::new(s).unwrap();
        let inner = unsafe { raw::RedisModule_CreateString.unwrap()(ctx, str.as_ptr(), s.len()) };

        Str { ctx, inner }
    }
    pub fn append(&mut self, s: &str) -> Result<(), Error> {
        handle_status(
            unsafe {
                raw::RedisModule_StringAppendBuffer.unwrap()(self.ctx, self.inner, s.as_ptr() as *mut i8, s.len()).into()
            },
            "Could not append buffer"
        )
    }
}

impl Drop for Str {
    fn drop(&mut self) {
        unsafe {
            raw::RedisModule_FreeString.unwrap()(self.ctx, self.inner);
        }
    }
}

impl fmt::Display for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut len: libc::size_t = 0;
        let bytes = unsafe { raw::RedisModule_StringPtrLen.unwrap()(self.inner, &mut len) };
        let value = str::from_utf8(unsafe { slice::from_raw_parts(bytes as *const u8, len) }).map_err(|_| fmt::Error)?;
        write!(f, "{}", value)
    }
}
