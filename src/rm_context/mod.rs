use crate::raw;
use crate::{
    handle_status, CallReply, Error, KeySpaceTypes, LogLevel, ReadKey, RedisResult, RedisString,
    RedisStr, RedisValue, StatusCode, WriteKey, FMT,
};
use bitflags::bitflags;
use std::ffi::CString;
use std::os::raw::{c_int, c_long, c_void};

pub mod block_client;
pub mod cluster;
pub mod timer;

pub struct Context {
    pub inner: *mut raw::RedisModuleCtx,
}

impl Context {
    pub fn new(inner: *mut raw::RedisModuleCtx) -> Self {
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
    pub fn auto_memory(&self) {
        unsafe { raw::RedisModule_AutoMemory.unwrap()(self.inner) }
    }
    pub fn reply(&self, r: RedisResult) -> StatusCode {
        match r {
            Ok(RedisValue::Integer(v)) => unsafe {
                raw::RedisModule_ReplyWithLongLong.unwrap()(self.inner, v).into()
            },

            Ok(RedisValue::Float(v)) => unsafe {
                raw::RedisModule_ReplyWithDouble.unwrap()(self.inner, v).into()
            },

            Ok(RedisValue::SimpleString(s)) => unsafe {
                let msg = CString::new(s).unwrap();
                raw::RedisModule_ReplyWithSimpleString.unwrap()(self.inner, msg.as_ptr()).into()
            },

            Ok(RedisValue::BulkString(s)) => unsafe {
                raw::RedisModule_ReplyWithString.unwrap()(
                    self.inner,
                    RedisString::create(self.inner, &s).inner,
                )
                .into()
            },

            Ok(RedisValue::Buffer(b)) => unsafe {
                raw::RedisModule_ReplyWithStringBuffer.unwrap()(
                    self.inner,
                    b.as_ptr() as *const i8,
                    b.len(),
                )
                .into()
            },

            Ok(RedisValue::Array(array)) => {
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

            Ok(RedisValue::Null) => unsafe {
                raw::RedisModule_ReplyWithNull.unwrap()(self.inner).into()
            },

            Ok(RedisValue::NoReply) => StatusCode::Ok,

            Err(Error::WrongArity) => {
                if self.is_keys_position_request() {
                    StatusCode::Err
                } else {
                    unsafe { raw::RedisModule_WrongArity.unwrap()(self.inner).into() }
                }
            }
            Err(Error::Generic(s)) => unsafe {
                let msg = CString::new(s.to_string()).unwrap();
                raw::RedisModule_ReplyWithError.unwrap()(self.inner, msg.as_ptr()).into()
            },
        }
    }

    pub fn call(&self, command: &str, args: &[RedisStr]) -> RedisResult {
        let args: Vec<*mut raw::RedisModuleString> =
            args.iter().map(|s| s.inner).collect();

        let cmd = CString::new(command).unwrap();

        let reply_: *mut raw::RedisModuleCallReply = unsafe {
            let p_call = raw::RedisModule_Call.unwrap();
            p_call(
                self.inner,
                cmd.as_ptr(),
                FMT,
                args.as_ptr() as *mut i8,
                args.len(),
            )
        };
        CallReply::new(reply_).into()
    }

    pub fn replicate(&self, command: &str, args: &[RedisStr]) -> Result<(), Error> {
        let args: Vec<*mut raw::RedisModuleString> =
            args.iter().map(|s| s.inner).collect();

        let cmd = CString::new(command).unwrap();

        let result = unsafe {
            let p_call = raw::RedisModule_Replicate.unwrap();
            p_call(
                self.inner,
                cmd.as_ptr(),
                FMT,
                args.as_ptr() as *mut i8,
                args.len(),
            )
        };
        handle_status(result, "Cloud not replicate")
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
            "Cloud not select db",
        )
    }
    pub fn create_string(&self, value: &str) -> RedisString {
        RedisString::create(self.inner, value)
    }
    pub fn open_read_key(&self, keyname: &RedisStr) -> ReadKey {
        ReadKey::new(self.inner, keyname)
    }
    pub fn open_write_key(&self, keyname: &RedisStr) -> WriteKey {
        WriteKey::new(self.inner, keyname)
    }
    pub fn subscribe_to_keyspace_events<F>(&self, _types: KeySpaceTypes, _callback: F) {
        unimplemented!()
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

bitflags! {
    pub struct ClusterFlags: u64 {
        const NONE = raw:: REDISMODULE_CLUSTER_FLAG_NONE as u64;
        const NO_FAILOVER = raw::REDISMODULE_CLUSTER_FLAG_NO_FAILOVER as u64;
        const NO_REPLICATION = raw::REDISMODULE_CLUSTER_FLAG_NO_REDIRECTION as u64;
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
