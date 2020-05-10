use std::ffi::CString;
use std::os::raw::{c_char, c_uchar};

use crate::raw;
use crate::{Error, LogLevel, Buffer, RString, RStr, ArgvFlags, Ptr};

#[repr(C)]
pub struct IO {
    inner: *mut raw::RedisModuleIO,
}

impl Ptr for IO {
    type PtrType = raw::RedisModuleIO;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.inner
    }
}

impl IO {
    pub fn from_ptr(inner: *mut raw::RedisModuleIO) -> Self {
        IO { inner }
    }
    pub fn save_unsigned(&mut self, value: u64) {
        unsafe { raw::RedisModule_SaveUnsigned.unwrap()(self.inner, value) };
    }
    pub fn load_unsigned(&mut self) -> u64 {
        unsafe { raw::RedisModule_LoadUnsigned.unwrap()(self.inner) }
    }
    pub fn save_signed(&mut self, value: i64) {
        unsafe { raw::RedisModule_SaveSigned.unwrap()(self.inner, value) };
    }
    pub fn load_signed(&mut self) -> i64 {
        unsafe { raw::RedisModule_LoadSigned.unwrap()(self.inner) }
    }
    pub fn save_redis_string(&mut self, value: &RString) {
        unsafe { raw::RedisModule_SaveString.unwrap()(self.inner, value.get_ptr()) }
    }
    pub fn save_string_buffer(&mut self, value: &[u8]) {
        unsafe {
            raw::RedisModule_SaveStringBuffer.unwrap()(
                self.inner,
                value.as_ptr() as *const c_char,
                value.len(),
            )
        };
    }
    pub fn save_string(&mut self, value: &str) {
        self.save_string_buffer(value.as_bytes())
    }
    pub fn load_string_buffer(&mut self) -> Buffer {
        let mut len = 0;
        let bytes = unsafe { raw::RedisModule_LoadStringBuffer.unwrap()(self.inner, &mut len) };
        Buffer::new(bytes, len)
    }
    pub fn load_string(&mut self) -> Result<String, Error> {
        let buffer = self.load_string_buffer();
        buffer.to_string().map_err(|e| e.into())
    }
    pub fn save_double(&mut self, value: f64) {
        unsafe { raw::RedisModule_SaveDouble.unwrap()(self.inner, value) };
    }
    pub fn load_double(&self) -> f64 {
        unsafe { raw::RedisModule_LoadDouble.unwrap()(self.inner) }
    }
    pub fn save_float(&mut self, value: f32) {
        unsafe { raw::RedisModule_SaveFloat.unwrap()(self.inner, value) };
    }
    pub fn load_float(&mut self) -> f32 {
        unsafe { raw::RedisModule_LoadFloat.unwrap()(self.inner) }
    }
    pub fn emit_aof(&mut self, command: &str, flags: ArgvFlags, args: &[&str]) {
        let terminated_args: Vec<CString> = args
            .iter()
            .map(|s| CString::new(*s).unwrap())
            .collect();

        let inner_args: Vec<_> = terminated_args.iter().map(|s| s.as_ptr()).collect();
        let flags: CString = flags.into();

        let cmd = CString::new(command).unwrap();

        unsafe {
            let p_call = raw::RedisModule_EmitAOF.unwrap();
            p_call(
                self.inner,
                cmd.as_ptr(),
                flags.as_ptr(),
                inner_args.as_ptr() as *mut c_char,
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
    inner: *mut raw::RedisModuleDigest,
}

impl Ptr for Digest {
    type PtrType = raw::RedisModuleDigest;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.inner
    }
}

impl Digest {
    pub fn from_ptr(inner: *mut raw::RedisModuleDigest) -> Self {
        Digest { inner }
    }
    pub fn add_string_buffer(&mut self, ele: &str) {
        let s = CString::new(ele).unwrap();
        unsafe { raw::RedisModule_DigestAddStringBuffer.unwrap()(self.inner, s.as_ptr() as *mut c_uchar, ele.len()) }
    }
    pub fn add_long_long(&mut self, ll: i64) {
        unsafe { raw::RedisModule_DigestAddLongLong.unwrap()(self.inner, ll) }
    }
    pub fn end_sequeue(&mut self) {
        unsafe { raw::RedisModule_DigestEndSequence.unwrap()(self.inner) }
    }
}
