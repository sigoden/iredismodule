//! Module context

use crate::call_reply::CallReply;
use crate::error::Error;
use crate::key::{ReadKey, WriteKey};
use crate::raw;
use crate::scan_cursor::ScanCursor;
use crate::string::{RStr, RString};
use crate::user::User;
use crate::value::Value;
use crate::{handle_status, ArgvFlags, LogLevel, Ptr, RResult, StatusCode};

use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_long, c_void};

mod block_client;
mod cluster;
mod mutex;
mod subscribe;
mod timer;
pub use mutex::MutexContext;

#[repr(C)]
pub struct Context {
    ptr: *mut raw::RedisModuleCtx,
}

impl Ptr for Context {
    type PtrType = raw::RedisModuleCtx;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.ptr
    }
}

impl Context {
    pub fn from_ptr(ptr: *mut raw::RedisModuleCtx) -> Self {
        Context { ptr }
    }
    pub fn is_keys_position_request(&self) -> bool {
        // We want this to be available in tests where we don't have an actual Redis to call
        if cfg!(feature = "test") {
            return false;
        }

        let result = unsafe { raw::RedisModule_IsKeysPositionRequest.unwrap()(self.ptr) };

        result != 0
    }
    pub fn key_at_pos(&self, pos: i32) {
        // TODO: This will crash redis if `pos` is out of range.
        // Think of a way to make this safe by checking the range.
        unsafe {
            raw::RedisModule_KeyAtPos.unwrap()(self.ptr, pos as c_int);
        }
    }
    pub fn reply(&self, r: RResult) -> StatusCode {
        match r {
            Ok(Value::Integer(v)) => unsafe {
                raw::RedisModule_ReplyWithLongLong.unwrap()(self.ptr, v).into()
            },

            Ok(Value::Float(v)) => unsafe {
                raw::RedisModule_ReplyWithDouble.unwrap()(self.ptr, v).into()
            },

            Ok(Value::String(v)) => unsafe {
                let msg = CString::new(v).unwrap();
                raw::RedisModule_ReplyWithSimpleString.unwrap()(self.ptr, msg.as_ptr()).into()
            },

            Ok(Value::Buffer(v)) => unsafe {
                raw::RedisModule_ReplyWithStringBuffer.unwrap()(
                    self.ptr,
                    v.as_ptr() as *const c_char,
                    v.len(),
                )
                .into()
            },

            Ok(Value::Array(v)) => {
                unsafe {
                    raw::RedisModule_ReplyWithArray.unwrap()(self.ptr, v.len() as c_long);
                }

                for elem in v {
                    self.reply(Ok(elem));
                }

                StatusCode::Ok
            }

            Ok(Value::Null) => unsafe { raw::RedisModule_ReplyWithNull.unwrap()(self.ptr).into() },

            Ok(Value::NoReply) => StatusCode::Ok,

            Err(Error::WrongArity) => {
                if self.is_keys_position_request() {
                    StatusCode::Err
                } else {
                    unsafe { raw::RedisModule_WrongArity.unwrap()(self.ptr).into() }
                }
            }
            Err(err) => unsafe {
                let msg = CString::new(err.to_string()).unwrap();
                raw::RedisModule_ReplyWithError.unwrap()(self.ptr, msg.as_ptr()).into()
            },
        }
    }

    pub fn call(&self, command: &str, flags: ArgvFlags, args: &[&RStr]) -> CallReply {
        let args: Vec<*mut raw::RedisModuleString> = args.iter().map(|s| s.get_ptr()).collect();

        let cmd = CString::new(command).unwrap();
        let flags: CString = flags.into();

        let reply_: *mut raw::RedisModuleCallReply = unsafe {
            let p_call = raw::RedisModule_Call.unwrap();
            p_call(
                self.ptr,
                cmd.as_ptr(),
                flags.as_ptr(),
                args.as_ptr() as *mut c_char,
                args.len(),
            )
        };
        CallReply::from_ptr(reply_)
    }
    pub fn call_str<T: AsRef<str>>(
        &self,
        command: &str,
        flags: ArgvFlags,
        args: &[T],
    ) -> CallReply {
        let str_args: Vec<RString> = args
            .iter()
            .map(|v| self.create_string(v.as_ref()))
            .collect();
        let str_args: Vec<&RStr> = str_args.iter().map(|v| v.get_redis_str()).collect();
        self.call(command, flags, &str_args)
    }
    pub fn replicate(&self, command: &str, flags: ArgvFlags, args: &[&RStr]) -> Result<(), Error> {
        let args: Vec<*mut raw::RedisModuleString> = args.iter().map(|s| s.get_ptr()).collect();

        let cmd = CString::new(command).unwrap();
        let flags: CString = flags.into();

        let result = unsafe {
            let p_call = raw::RedisModule_Replicate.unwrap();
            p_call(
                self.ptr,
                cmd.as_ptr(),
                flags.as_ptr(),
                args.as_ptr() as *mut c_char,
                args.len(),
            )
        };
        handle_status(result, "fail to replicate")
    }
    pub fn replicate_str<T: AsRef<str>>(
        &self,
        command: &str,
        flags: ArgvFlags,
        args: &[T],
    ) -> Result<(), Error> {
        let str_args: Vec<RString> = args
            .iter()
            .map(|v| self.create_string(v.as_ref()))
            .collect();
        let str_args: Vec<&RStr> = str_args.iter().map(|v| v.get_redis_str()).collect();
        self.replicate(command, flags, &str_args)
    }
    pub fn replicate_verbatim(&self) {
        unsafe {
            raw::RedisModule_ReplicateVerbatim.unwrap()(self.ptr);
        }
    }
    pub fn get_client_id(&self) -> u64 {
        unsafe { raw::RedisModule_GetClientId.unwrap()(self.ptr) as u64 }
    }
    pub fn get_select_db(&self) -> i64 {
        unsafe { raw::RedisModule_GetSelectedDb.unwrap()(self.ptr) as i64 }
    }
    pub fn get_context_flags(&self) -> u64 {
        unsafe { raw::RedisModule_GetContextFlags.unwrap()(self.ptr) as u64 }
    }
    pub fn select_db(&self, newid: i32) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_SelectDb.unwrap()(self.ptr, newid) },
            "fail to select db",
        )
    }
    pub fn create_string(&self, value: &str) -> RString {
        RString::from_str(self.ptr, value)
    }
    pub fn open_read_key(&self, keyname: &RStr) -> ReadKey {
        ReadKey::from_redis_str(self.ptr, keyname)
    }
    pub fn open_write_key(&self, keyname: &RStr) -> WriteKey {
        WriteKey::from_redis_str(self.ptr, keyname)
    }
    pub fn signal_key_as_ready(&self, key: &RStr) {
        unsafe { raw::RedisModule_SignalKeyAsReady.unwrap()(self.ptr, key.get_ptr()) };
    }
    pub fn log<T: AsRef<str>>(&self, level: LogLevel, message: T) {
        let level: CString = level.into();
        let fmt = CString::new(message.as_ref()).unwrap();
        unsafe { raw::RedisModule_Log.unwrap()(self.ptr, level.as_ptr(), fmt.as_ptr()) }
    }
    pub fn notice<T: AsRef<str>>(&self, message: T) {
        self.log(LogLevel::Notice, message.as_ref());
    }
    pub fn debug<T: AsRef<str>>(&self, message: T) {
        self.log(LogLevel::Debug, message.as_ref());
    }
    pub fn verbose<T: AsRef<str>>(&self, message: T) {
        self.log(LogLevel::Verbose, message.as_ref());
    }
    pub fn warning<T: AsRef<str>>(&self, message: T) {
        self.log(LogLevel::Warning, message.as_ref());
    }
    pub fn create_cmd(
        &self,
        name: &str,
        func: extern "C" fn(
            *mut raw::RedisModuleCtx,
            *mut *mut raw::RedisModuleString,
            c_int,
        ) -> c_int,
        flags: &str,
        first_key: usize,
        last_key: usize,
        key_step: usize,
    ) -> Result<(), Error> {
        let name = CString::new(name).unwrap();
        let flags = CString::new(flags).unwrap();
        handle_status(
            unsafe {
                raw::RedisModule_CreateCommand.unwrap()(
                    self.ptr,
                    name.as_ptr(),
                    Some(func),
                    flags.as_ptr(),
                    first_key as c_int,
                    last_key as c_int,
                    key_step as c_int,
                )
            },
            "fail to create command",
        )
    }

    pub fn deauthenticate_and_close_client(&self, id: u64) {
        unsafe { raw::RedisModule_DeauthenticateAndCloseClient.unwrap()(self.ptr, id) }
    }
    pub fn authenticate_client_with_acl_user<T>(
        &self,
        name: &str,
        callback: raw::RedisModuleUserChangedFunc,
        privdata: Option<T>,
        client_id: u64,
    ) -> Result<u64, Error> {
        let c_name = CString::new(name).unwrap();
        let data = match privdata {
            Some(v) => Box::into_raw(Box::from(v)) as *mut c_void,
            None => 0 as *mut c_void,
        };
        let mut client_id = client_id;
        handle_status(
            unsafe {
                raw::RedisModule_AuthenticateClientWithACLUser.unwrap()(
                    self.ptr,
                    c_name.as_ptr(),
                    name.len(),
                    callback,
                    data,
                    &mut client_id,
                )
            },
            "fail to authenticate client",
        )?;
        Ok(client_id)
    }
    pub fn authenticate_client_with_user<T>(
        &self,
        user: &User,
        callback: raw::RedisModuleUserChangedFunc,
        privdata: Option<T>,
        client_id: u64,
    ) -> Result<u64, Error> {
        let data = match privdata {
            Some(v) => Box::into_raw(Box::from(v)) as *mut c_void,
            None => 0 as *mut c_void,
        };
        let mut client_id = client_id;
        handle_status(
            unsafe {
                raw::RedisModule_AuthenticateClientWithUser.unwrap()(
                    self.ptr,
                    user.get_ptr(),
                    callback,
                    data,
                    &mut client_id,
                )
            },
            "fail to authenticate client",
        )?;
        Ok(client_id)
    }
    pub fn db_size(&self) -> u64 {
        unsafe { raw::RedisModule_DbSize.unwrap()(self.ptr) }
    }
    pub fn publish_message(&self, channel: &RStr, msg: &RStr) -> Result<(), Error> {
        handle_status(
            unsafe {
                raw::RedisModule_PublishMessage.unwrap()(self.ptr, channel.get_ptr(), msg.get_ptr())
            },
            "fail to publish message",
        )
    }
    pub fn signal_modified_key(&self, key: &RStr) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_SignalModifiedKey.unwrap()(self.ptr, key.get_ptr()) },
            "fail to signal key modified",
        )
    }
    pub fn set_module_options(&self, options: i32) {
        unsafe { raw::RedisModule_SetModuleOptions.unwrap()(self.ptr, options) }
    }
    pub fn scan<T>(
        &self,
        cursor: &ScanCursor,
        callback: raw::RedisModuleScanCB,
        privdata: Option<T>,
    ) -> Result<(), Error> {
        let data = match privdata {
            Some(v) => Box::into_raw(Box::from(v)) as *mut c_void,
            None => 0 as *mut c_void,
        };
        handle_status(
            unsafe { raw::RedisModule_Scan.unwrap()(self.ptr, cursor.get_ptr(), callback, data) },
            "fail to scan",
        )
    }
    pub fn export_shared_api(&self, name: &str, fn_ptr: *mut c_void) -> Result<(), Error> {
        let name = CString::new(name).unwrap();
        handle_status(
            unsafe { raw::RedisModule_ExportSharedAPI.unwrap()(self.ptr, name.as_ptr(), fn_ptr) },
            "fail to export shared api",
        )
    }
    pub fn get_shared_api(&self, name: &str) -> Option<*mut c_void> {
        let name = CString::new(name).unwrap();
        let ptr: *mut c_void =
            unsafe { raw::RedisModule_GetSharedAPI.unwrap()(self.ptr, name.as_ptr()) };
        if ptr.is_null() {
            return None;
        }
        Some(ptr)
    }
}
