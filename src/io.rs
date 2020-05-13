use std::ffi::CString;
use std::os::raw::{c_char, c_uchar};

use crate::raw;
use crate::{ArgvFlags, Buffer, LogLevel, Ptr, RString, Context, RStr};

#[repr(C)]
pub struct IO {
    ptr: *mut raw::RedisModuleIO,
}

impl Ptr for IO {
    type PtrType = raw::RedisModuleIO;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.ptr
    }
}

impl IO {
    pub fn from_ptr(ptr: *mut raw::RedisModuleIO) -> Self {
        IO { ptr }
    }
    pub fn save_unsigned(&mut self, value: u64) {
        unsafe { raw::RedisModule_SaveUnsigned.unwrap()(self.ptr, value) };
    }
    pub fn load_unsigned(&mut self) -> u64 {
        unsafe { raw::RedisModule_LoadUnsigned.unwrap()(self.ptr) }
    }
    pub fn save_signed(&mut self, value: i64) {
        unsafe { raw::RedisModule_SaveSigned.unwrap()(self.ptr, value) };
    }
    pub fn load_signed(&mut self) -> i64 {
        unsafe { raw::RedisModule_LoadSigned.unwrap()(self.ptr) }
    }
    pub fn save_redis_string(&mut self, value: &RString) {
        unsafe { raw::RedisModule_SaveString.unwrap()(self.ptr, value.get_ptr()) }
    }
    pub fn load_redis_string(&mut self) -> RString {
        let ptr: *mut raw::RedisModuleString = unsafe { raw::RedisModule_LoadString.unwrap()(self.ptr) };
        let ctx = self.get_ctx();
        RString::new(ctx.get_ptr(), ptr)
    }
    pub fn save_string_buffer(&mut self, value: &[u8]) {
        unsafe {
            raw::RedisModule_SaveStringBuffer.unwrap()(
                self.ptr,
                value.as_ptr() as *const c_char,
                value.len(),
            )
        };
    }
    pub fn load_string_buffer(&mut self) -> Buffer {
        let mut len = 0;
        let bytes = unsafe { raw::RedisModule_LoadStringBuffer.unwrap()(self.ptr, &mut len) };
        Buffer::new(bytes, len)
    }
    pub fn save_double(&mut self, value: f64) {
        unsafe { raw::RedisModule_SaveDouble.unwrap()(self.ptr, value) };
    }
    pub fn load_double(&self) -> f64 {
        unsafe { raw::RedisModule_LoadDouble.unwrap()(self.ptr) }
    }
    pub fn save_float(&mut self, value: f32) {
        unsafe { raw::RedisModule_SaveFloat.unwrap()(self.ptr, value) };
    }
    pub fn load_float(&mut self) -> f32 {
        unsafe { raw::RedisModule_LoadFloat.unwrap()(self.ptr) }
    }
    pub fn emit_aof<T: AsRef<str>>(&mut self, command: &str, flags: ArgvFlags, args: &[T]) {
        let terminated_args: Vec<CString> = args
            .iter()
            .map(|s| CString::new(s.as_ref()).unwrap())
            .collect();

        let inner_args: Vec<_> = terminated_args.iter().map(|s| s.as_ptr()).collect();
        let flags: CString = flags.into();

        let cmd = CString::new(command).unwrap();

        unsafe {
            let p_call = raw::RedisModule_EmitAOF.unwrap();
            p_call(
                self.ptr,
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
        unsafe { raw::RedisModule_LogIOError.unwrap()(self.ptr, level.as_ptr(), fmt.as_ptr()) }
    }
    pub fn get_ctx(&self) -> Context {
       let ptr: *mut raw::RedisModuleCtx = unsafe { raw::RedisModule_GetContextFromIO.unwrap()(self.ptr) };
       Context::from_ptr(ptr)
    }
    pub fn have_error(&self) -> bool {
        unsafe { raw::RedisModule_IsIOError.unwrap()(self.ptr) != 0 }
    }
    pub fn get_keyname(&self) -> RStr {
        let ptr = unsafe { raw::RedisModule_GetKeyNameFromIO.unwrap()(self.ptr) };
        RStr::from_ptr({ ptr as *mut raw::RedisModuleString })
    }
}

#[repr(C)]
pub struct Digest {
    ptr: *mut raw::RedisModuleDigest,
}

impl Ptr for Digest {
    type PtrType = raw::RedisModuleDigest;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.ptr
    }
}

impl Digest {
    pub fn from_ptr(ptr: *mut raw::RedisModuleDigest) -> Self {
        Digest { ptr }
    }
    pub fn add_string_buffer(&mut self, ele: &str) {
        let s = CString::new(ele).unwrap();
        unsafe {
            raw::RedisModule_DigestAddStringBuffer.unwrap()(
                self.ptr,
                s.as_ptr() as *mut c_uchar,
                ele.len(),
            )
        }
    }
    pub fn add_long_long(&mut self, ll: i64) {
        unsafe { raw::RedisModule_DigestAddLongLong.unwrap()(self.ptr, ll) }
    }
    pub fn end_sequeue(&mut self) {
        unsafe { raw::RedisModule_DigestEndSequence.unwrap()(self.ptr) }
    }
}
