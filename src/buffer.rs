//! Owned u8 slice

use std::os::raw::{c_char, c_void};
use std::slice;
use std::string::FromUtf8Error;

use crate::raw;

#[derive(Debug)]
pub struct Buffer {
    buffer: *const c_char,
    len: usize,
}

impl Buffer {
    pub fn new(buffer: *const c_char, len: usize) -> Buffer {
        Buffer { buffer, len }
    }

    pub fn to_string(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.as_ref().to_vec())
    }
}

impl AsRef<[u8]> for Buffer {
    fn as_ref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.buffer as *const u8, self.len) }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            raw::RedisModule_Free.unwrap()(self.buffer as *mut c_void);
        }
    }
}
