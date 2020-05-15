//! Deal with rdb and digest

use std::ffi::CString;
use std::os::raw::{c_char, c_uchar};

use crate::context::Context;
use crate::raw;
use crate::string::{RStr, RString};
use crate::{FromPtr, GetPtr, LogLevel};

/// Wrap the pointer of a RedisModuleIO
#[repr(C)]
pub struct IO {
    ptr: *mut raw::RedisModuleIO,
}

impl GetPtr for IO {
    type PtrType = raw::RedisModuleIO;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.ptr
    }
}

impl FromPtr for IO {
    type PtrType = raw::RedisModuleIO;
    fn from_ptr(ptr: *mut raw::RedisModuleIO) -> IO {
        IO { ptr }
    }
}

impl IO {
    /// Save an unsigned 64 bit value into the RDB file. This function should only
    /// be called in the context of the rdb_save method of modules implementing new
    /// data types.
    pub fn save_unsigned(&mut self, value: u64) {
        unsafe { raw::RedisModule_SaveUnsigned.unwrap()(self.ptr, value) };
    }
    /// Load an unsigned 64 bit value from the RDB file. This function should only
    /// be called in the context of the rdb_load method of modules implementing
    /// new data types.
    pub fn load_unsigned(&mut self) -> u64 {
        unsafe { raw::RedisModule_LoadUnsigned.unwrap()(self.ptr) }
    }
    /// Like `IO::save_unsigned` but for signed 64 bit values.
    pub fn save_signed(&mut self, value: i64) {
        unsafe { raw::RedisModule_SaveSigned.unwrap()(self.ptr, value) };
    }
    /// Like `IO::load_unsigned` but for signed 64 bit values.
    pub fn load_signed(&mut self) -> i64 {
        unsafe { raw::RedisModule_LoadSigned.unwrap()(self.ptr) }
    }
    /// In the context of the rdb_save method of a module type, saves a
    /// redis string into the RDB file.
    pub fn save_rstring(&mut self, value: RString) {
        unsafe { raw::RedisModule_SaveString.unwrap()(self.ptr, value.get_ptr()) }
    }
    /// In the context of the rdb_load method of a module data type, loads a redis string
    /// from the RDB file, that was previously saved with `IO::save_rstring`.
    pub fn load_rstring(&mut self) -> RString {
        let ptr: *mut raw::RedisModuleString =
            unsafe { raw::RedisModule_LoadString.unwrap()(self.ptr) };
        RString::from_ptr(ptr)
    }
    /// In the context of the rdb_save method of a module type, saves a
    /// string into the RDB file.
    pub fn save_string(&mut self, value: &str) {
        let value_ = CString::new(value).unwrap();
        unsafe {
            raw::RedisModule_SaveStringBuffer.unwrap()(self.ptr, value_.as_ptr(), value.len())
        }
    }
    /// In the context of the rdb_load method of a module data type, loads a string
    /// from the RDB file, that was previously saved with `IO::save_string`.
    pub fn load_string(&mut self) -> String {
        let data: &[u8] = unsafe {
            let mut len = 0;
            let ptr = raw::RedisModule_LoadStringBuffer.unwrap()(self.ptr, &mut len);
            std::slice::from_raw_parts(ptr as *const u8, len)
        };
        String::from_utf8(data.to_vec()).unwrap()
    }
    /// In the context of the rdb_save method of a module type, saves a
    /// double into the RDB file.
    pub fn save_double(&mut self, value: f64) {
        unsafe { raw::RedisModule_SaveDouble.unwrap()(self.ptr, value) };
    }
    /// In the context of the rdb_load method of a module data type, loads a double
    /// from the RDB file, that was previously saved with `IO::save_double`.
    pub fn load_double(&self) -> f64 {
        unsafe { raw::RedisModule_LoadDouble.unwrap()(self.ptr) }
    }
    /// In the context of the rdb_save method of a module type, saves a
    /// float into the RDB file.
    pub fn save_float(&mut self, value: f32) {
        unsafe { raw::RedisModule_SaveFloat.unwrap()(self.ptr, value) };
    }
    /// In the context of the rdb_load method of a module data type, loads a float
    /// from the RDB file, that was previously saved with `IO::save_float`.
    pub fn load_float(&mut self) -> f32 {
        unsafe { raw::RedisModule_LoadFloat.unwrap()(self.ptr) }
    }
    /// Emits a command into the AOF during the AOF rewriting process. This function
    /// is only called in the context of the aof_rewrite method of data types exported
    /// by a module. The command works exactly like `Context::Call` in the way
    /// the parameters are passed, but it does not return anything as the error
    /// handling is performed by Redis itself.
    pub fn emit_aof<T: AsRef<str>>(&mut self, command: T, args: &[T]) {
        let terminated_args: Vec<CString> = args
            .iter()
            .map(|s| CString::new(s.as_ref()).unwrap())
            .collect();

        let inner_args: Vec<_> = terminated_args.iter().map(|s| s.as_ptr()).collect();
        let flags: CString = CString::new("v").unwrap();

        let cmd = CString::new(command.as_ref()).unwrap();

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
    /// Log errors from RDB / AOF serialization callbacks.
    ///
    /// This function should be used when a callback is returning a critical
    /// error to the caller since cannot load or save the data for some
    /// critical reason.
    pub fn log_io_error(&self, level: LogLevel, message: &str) {
        let level: CString = level.into();
        let fmt = CString::new(message).unwrap();
        unsafe { raw::RedisModule_LogIOError.unwrap()(self.ptr, level.as_ptr(), fmt.as_ptr()) }
    }
    pub fn get_ctx(&self) -> Context {
        let ptr: *mut raw::RedisModuleCtx =
            unsafe { raw::RedisModule_GetContextFromIO.unwrap()(self.ptr) };
        Context::from_ptr(ptr)
    }
    /// Returns true if any previous IO API failed.
    ///
    /// for Load* APIs the REDISMODULE_OPTIONS_HANDLE_IO_ERRORS flag must be set with
    /// `Context::set_module_options` first.
    pub fn have_error(&self) -> bool {
        unsafe { raw::RedisModule_IsIOError.unwrap()(self.ptr) != 0 }
    }
    /// Returns a RStr with the name of the key currently saving or
    /// loading, when an IO data type callback is called.
    pub fn get_keyname(&self) -> Option<RStr> {
        let ptr = unsafe { raw::RedisModule_GetKeyNameFromIO.unwrap()(self.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(RStr::from_ptr({ ptr as *mut raw::RedisModuleString }))
        }
    }
}

/// Wrap the pointer of a RedisModuleDigest
#[repr(C)]
pub struct Digest {
    ptr: *mut raw::RedisModuleDigest,
}

impl GetPtr for Digest {
    type PtrType = raw::RedisModuleDigest;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.ptr
    }
}

impl FromPtr for Digest {
    type PtrType = raw::RedisModuleDigest;
    fn from_ptr(ptr: *mut raw::RedisModuleDigest) -> Digest {
        Digest { ptr }
    }
}

impl Digest {
    /// Add a new element to the digest. This function can be called multiple times
    /// one element after the other, for all the elements that constitute a given
    /// data structure. The function call must be followed by the call to
    /// `RedisModule_DigestEndSequence` eventually, when all the elements that are
    /// always in a given order are added. See the Redis Modules data types
    /// documentation for more info. However this is a quick example that uses Redis
    /// data types as an example.
    ///
    /// To add a sequence of unordered elements (for example in the case of a Redis
    /// Set), the pattern to use is:
    ///
    ///     foreach element {
    ///         AddElement(element);
    ///         EndSequence();
    ///     }
    ///
    /// Because Sets are not ordered, so every element added has a position that
    /// does not depend from the other. However if instead our elements are
    /// ordered in pairs, like field-value pairs of an Hash, then one should
    /// use:
    ///
    ///     foreach key,value {
    ///         AddElement(key);
    ///         AddElement(value);
    ///         EndSquence();
    ///     }
    ///
    /// Because the key and value will be always in the above order, while instead
    /// the single key-value pairs, can appear in any position into a Redis hash.
    ///
    /// A list of ordered elements would be implemented with:
    ///
    ///     foreach element {
    ///         AddElement(element);
    ///     }
    ///     EndSequence();
    ///
    pub fn add_string<T: AsRef<str>>(&mut self, s: T) {
        let s_ = CString::new(s.as_ref()).unwrap();
        unsafe {
            raw::RedisModule_DigestAddStringBuffer.unwrap()(
                self.ptr,
                s_.as_ptr() as *mut c_uchar,
                s.as_ref().len(),
            )
        }
    }
    /// Like `Digest::digest_add_string_buffer` but takes a long long as input
    /// that gets converted into a string before adding it to the digest.
    pub fn add_integer(&mut self, i: i64) {
        unsafe { raw::RedisModule_DigestAddLongLong.unwrap()(self.ptr, i) }
    }
    /// See `Context:add_string_buffer`
    pub fn end_sequeue(&mut self) {
        unsafe { raw::RedisModule_DigestEndSequence.unwrap()(self.ptr) }
    }
}
