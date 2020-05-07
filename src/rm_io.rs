use std::ffi::CString;
use std::os::raw::c_char;
use std::slice;

use crate::raw;
use crate::{CmdFmtFlags, LogLevel, RedisBuffer, Str};
pub struct IO {
    pub inner: *mut raw::RedisModuleIO,
}

impl IO {
    pub fn create(inner: *mut raw::RedisModuleIO) -> Self {
        IO { inner }
    }
    pub fn save_unsigned(&self, value: u64) {
        unsafe { raw::RedisModule_SaveUnsigned.unwrap()(self.inner, value) };
    }
    pub fn load_unsigned(&self) -> u64 {
        unsafe { raw::RedisModule_LoadUnsigned.unwrap()(self.inner) }
    }
    pub fn save_signed(&self, value: i64) {
        unsafe { raw::RedisModule_SaveSigned.unwrap()(self.inner, value) };
    }
    pub fn load_signed(&self) -> i64 {
        unsafe { raw::RedisModule_LoadSigned.unwrap()(self.inner) }
    }
    pub fn save_string(&self, value: &Str) {
        unsafe { raw::RedisModule_SaveString.unwrap()(self.inner, value.inner) }
    }
    pub fn load_string(&self) -> *mut raw::RedisModuleString {
        unsafe { raw::RedisModule_LoadString.unwrap()(self.inner) }
    }
    pub fn save_string_buffer(&self, value: &[u8]) {
        unsafe {
            raw::RedisModule_SaveStringBuffer.unwrap()(
                self.inner,
                value.as_ptr() as *const c_char,
                value.len(),
            )
        };
    }
    pub fn load_string_buffer(&self) -> RedisBuffer {
        let mut len = 0;
        let bytes = unsafe { raw::RedisModule_LoadStringBuffer.unwrap()(self.inner, &mut len) };
        RedisBuffer::new(bytes, len)
    }
    pub fn save_double(&self, value: f64) {
        unsafe { raw::RedisModule_SaveDouble.unwrap()(self.inner, value) };
    }
    pub fn load_double(&self) -> f64 {
        unsafe { raw::RedisModule_LoadDouble.unwrap()(self.inner) }
    }
    pub fn save_float(&self, value: f32) {
        unsafe { raw::RedisModule_SaveFloat.unwrap()(self.inner, value) };
    }
    pub fn load_float(&self) -> f32 {
        unsafe { raw::RedisModule_LoadFloat.unwrap()(self.inner) }
    }
    pub fn emit_aof(&self, command: &str, args: &[&str], flags: &[CmdFmtFlags]) {
        let terminated_args: Vec<CString> =
            args.iter().map(|s| CString::new(*s).unwrap()).collect();

        let inner_args: Vec<_> = terminated_args.iter().map(|s| s.as_ptr()).collect();

        let cmd = CString::new(command).unwrap();
        let fmt = CString::new(CmdFmtFlags::multi(flags)).unwrap();

        unsafe {
            let p_call = raw::RedisModule_EmitAOF.unwrap();
            p_call(
                self.inner,
                cmd.as_ptr(),
                fmt.as_ptr(),
                inner_args.as_ptr() as *mut i8,
                terminated_args.len(),
            )
        };
    }
    pub fn log_io_error(&self, level: LogLevel, message: &str) {
        let level: CString = level.into();
        let fmt = CString::new(message).unwrap();
        unsafe { raw::RedisModule_LogIOError.unwrap()(self.inner, level.as_ptr(), fmt.as_ptr()) }
    }
}

pub struct Digest {
    pub inner: raw::RedisModuleDigest,
}

impl Digest {
    pub fn add_string_buffer(&mut self, ele: &str) {
        unimplemented!()
    }
    pub fn add_long_long(&mut self, ll: i128) {
        unimplemented!()
    }
    pub fn end_sequeue(&mut self) {
        unimplemented!()
    }
}
