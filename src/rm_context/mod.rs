use crate::raw;
use crate::{
    handle_status, CallReply, Error, KeySpaceTypes, LogLevel, ReadKey, RString,
    RStr, Value, StatusCode, WriteKey, Ptr, ArgvFlags,
};
use std::ffi::CString;
use std::os::raw::{c_int, c_long, c_void, c_char};

pub mod block_client;
pub mod cluster;
pub mod timer;

#[repr(C)]
pub struct Context {
    inner: *mut raw::RedisModuleCtx,
}

impl Ptr for Context {
    type PtrType = raw::RedisModuleCtx;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.inner
    }
}

impl Context {
    pub fn from_ptr(inner: *mut raw::RedisModuleCtx) -> Self {
        Context { inner }
    }
    pub fn is_keys_position_request(&self) -> bool {
        // We want this to be available in tests where we don't have an actual Redis to call
        if cfg!(feature = "test") {
            return false;
        }

        let result = unsafe { raw::RedisModule_IsKeysPositionRequest.unwrap()(self.inner) };

        result != 0
    }
    pub fn key_at_pos(&self, pos: i32) {
        // TODO: This will crash redis if `pos` is out of range.
        // Think of a way to make this safe by checking the range.
        unsafe {
            raw::RedisModule_KeyAtPos.unwrap()(self.inner, pos as c_int);
        }
    }
    pub fn reply(&self, r: crate::RResult) -> StatusCode {
        match r {
            Ok(Value::Integer(v)) => unsafe {
                raw::RedisModule_ReplyWithLongLong.unwrap()(self.inner, v).into()
            },

            Ok(Value::Float(v)) => unsafe {
                raw::RedisModule_ReplyWithDouble.unwrap()(self.inner, v).into()
            },

            Ok(Value::SimpleString(s)) => unsafe {
                let msg = CString::new(s).unwrap();
                raw::RedisModule_ReplyWithSimpleString.unwrap()(self.inner, msg.as_ptr()).into()
            },

            Ok(Value::BulkString(s)) => unsafe {
                raw::RedisModule_ReplyWithString.unwrap()(
                    self.inner,
                    RString::from_str(self.inner, &s).get_ptr(),
                )
                .into()
            },

            Ok(Value::Buffer(b)) => unsafe {
                raw::RedisModule_ReplyWithStringBuffer.unwrap()(
                    self.inner,
                    b.as_ptr() as *const c_char,
                    b.len(),
                )
                .into()
            },

            Ok(Value::Array(array)) => {
                unsafe {
                    // According to the Redis source code this always succeeds,
                    // so there is no point in checking its return value.
                    raw::RedisModule_ReplyWithArray.unwrap()(self.inner, array.len() as c_long);
                }

                for elem in array {
                    self.reply(Ok(elem));
                }

                StatusCode::Ok
            }

            Ok(Value::Null) => unsafe {
                raw::RedisModule_ReplyWithNull.unwrap()(self.inner).into()
            },

            Ok(Value::NoReply) => StatusCode::Ok,

            Err(Error::WrongArity) => {
                if self.is_keys_position_request() {
                    StatusCode::Err
                } else {
                    unsafe { raw::RedisModule_WrongArity.unwrap()(self.inner).into() }
                }
            }
            Err(err) => unsafe {
                let msg = CString::new(err.to_string()).unwrap();
                raw::RedisModule_ReplyWithError.unwrap()(self.inner, msg.as_ptr()).into()
            },
        }
    }

    pub fn call(&self, command: &str, flags: ArgvFlags, args: &[&RStr]) -> CallReply {
        let args: Vec<*mut raw::RedisModuleString> =
            args.iter().map(|s| s.get_ptr()).collect();

        let cmd = CString::new(command).unwrap();
        let flags: CString = flags.into();

        let reply_: *mut raw::RedisModuleCallReply = unsafe {
            let p_call = raw::RedisModule_Call.unwrap();
            p_call(
                self.inner,
                cmd.as_ptr(),
                flags.as_ptr(),
                args.as_ptr() as *mut c_char,
                args.len(),
            )
        };
        CallReply::from_ptr(reply_)
    }
    pub fn call_with_str_args(&self, command: &str, flags: ArgvFlags, args: &[&str]) -> CallReply {
        let str_args: Vec<RString> = args.iter().map(|v| self.create_string(v)).collect();
        let str_args: Vec<&RStr> = str_args.iter().map(|v| v.get_redis_str()).collect();
        self.call(command, flags, &str_args)
    }
    pub fn replicate(&self, command: &str, flags: ArgvFlags, args: &[&RStr]) -> Result<(), Error> {
        let args: Vec<*mut raw::RedisModuleString> =
            args.iter().map(|s| s.get_ptr()).collect();

        let cmd = CString::new(command).unwrap();
        let flags: CString = flags.into();

        let result = unsafe {
            let p_call = raw::RedisModule_Replicate.unwrap();
            p_call(
                self.inner,
                cmd.as_ptr(),
                flags.as_ptr(),
                args.as_ptr() as *mut c_char,
                args.len(),
            )
        };
        handle_status(result, "can not replicate")
    }
    pub fn replicate_with_str_args(&self, command: &str, flags: ArgvFlags, args: &[&str]) -> Result<(), Error> {
        let str_args: Vec<RString> = args.iter().map(|v| self.create_string(v)).collect();
        let str_args: Vec<&RStr> = str_args.iter().map(|v| v.get_redis_str()).collect();
        self.replicate(command, flags, &str_args)
    }
    pub fn replicate_verbatim(&self) {
        unsafe {
            raw::RedisModule_ReplicateVerbatim.unwrap()(self.inner);
        }
    }
    pub fn get_client_id(&self) -> u64 {
        unsafe { raw::RedisModule_GetClientId.unwrap()(self.inner) as u64 }
    }
    pub fn get_select_db(&self) -> i64 {
        unsafe { raw::RedisModule_GetSelectedDb.unwrap()(self.inner) as i64 }
    }
    pub fn get_context_flags(&self) -> u64 {
        unsafe { raw::RedisModule_GetContextFlags.unwrap()(self.inner) as u64 }
    }
    pub fn select_db(&self, newid: i32) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_SelectDb.unwrap()(self.inner, newid) },
            "can not select db",
        )
    }
    pub fn create_string(&self, value: &str) -> RString {
        RString::from_str(self.inner, value)
    }
    pub fn open_read_key(&self, keyname: &RStr) -> ReadKey {
        ReadKey::from_redis_str(self.inner, keyname)
    }
    pub fn open_write_key(&self, keyname: &RStr) -> WriteKey {
        WriteKey::from_redis_str(self.inner, keyname)
    }
    pub fn subscribe_to_keyspace_events<F>(&self, _types: KeySpaceTypes, _callback: F) {
        unimplemented!()
    }
    pub fn signal_key_as_ready(&self, key: &RStr) {
        unsafe { raw::RedisModule_SignalKeyAsReady.unwrap()(self.inner, key.get_ptr()) };
    }
    pub fn log(&self, level: LogLevel, message: &str) {
        let level: CString = level.into();
        let fmt = CString::new(message).unwrap();
        unsafe { raw::RedisModule_Log.unwrap()(self.inner, level.as_ptr(), fmt.as_ptr()) }
    }
    pub fn log_debug(&self, message: &str) {
        self.log(LogLevel::Notice, message);
    }
}

pub(crate) fn take_data<T>(data: *mut c_void) -> T {
    // Cast the *mut c_void supplied by the Redis API to a raw pointer of our custom type.
    let data = data as *mut T;

    // Take back ownership of the original boxed data, so we can unbox it safely.
    // If we don't do this, the data's memory will be leaked.
    let data = unsafe { Box::from_raw(data) };

    *data
}
