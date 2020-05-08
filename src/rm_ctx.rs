use crate::raw;
use crate::rm::{CODE_ERR, CODE_OK};
use crate::{
    handle_status, BlockClient, CallReply, ClusterNode, ClusterNodeList, CmdFmtFlags,
    Error, KeySpaceTypes, LogLevel, MsgType, ReadKey, RedisResult, RedisValue, RedisString, TimerID,
    WriteKey,
};
use bitflags::bitflags;
use std::convert::TryInto;
use std::ffi::CString;
use std::os::raw::{c_int, c_long, c_void};
use std::time::Duration;

pub struct Ctx {
    pub inner: *mut raw::RedisModuleCtx,
}

impl Ctx {
    pub fn new(inner: *mut raw::RedisModuleCtx) -> Self {
        Ctx { inner }
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
    pub fn reply(&self, r: RedisResult) -> Result<(), Error> {
        let status = match r {
            Ok(RedisValue::Integer(v)) => unsafe {
                raw::RedisModule_ReplyWithLongLong.unwrap()(self.inner, v)
            },

            Ok(RedisValue::Float(v)) => unsafe {
                raw::RedisModule_ReplyWithDouble.unwrap()(self.inner, v)
            },

            Ok(RedisValue::SimpleString(s)) => unsafe {
                let msg = CString::new(s).unwrap();
                raw::RedisModule_ReplyWithSimpleString.unwrap()(self.inner, msg.as_ptr())
            },

            Ok(RedisValue::BulkString(s)) => unsafe {
                raw::RedisModule_ReplyWithString.unwrap()(
                    self.inner,
                    RedisString::new(self.inner, &s).inner,
                )
            },

            Ok(RedisValue::Buffer(b)) => unsafe {
                raw::RedisModule_ReplyWithStringBuffer.unwrap()(
                    self.inner,
                    b.as_ptr() as *const i8,
                    b.len(),
                )
            },

            Ok(RedisValue::Array(array)) => {
                unsafe {
                    // According to the Redis source code this always succeeds,
                    // so there is no point in checking its return value.
                    raw::RedisModule_ReplyWithArray.unwrap()(self.inner, array.len() as c_long);
                }

                for elem in array {
                    self.reply(Ok(elem))?;
                }

                CODE_OK
            }

            Ok(RedisValue::Null) => unsafe { raw::RedisModule_ReplyWithNull.unwrap()(self.inner) },

            Ok(RedisValue::NoReply) => CODE_OK,

            Err(Error::WrongArity) => {
                if self.is_keys_position_request() {
                    CODE_ERR
                } else {
                    unsafe { raw::RedisModule_WrongArity.unwrap()(self.inner) }
                }
            }
            Err(Error::Generic(s)) => unsafe {
                let msg = CString::new(s.to_string()).unwrap();
                raw::RedisModule_ReplyWithError.unwrap()(self.inner, msg.as_ptr())
            },
            Err(Error::ParseInt(s)) => unsafe {
                let msg = CString::new(s.to_string()).unwrap();
                raw::RedisModule_ReplyWithError.unwrap()(self.inner, msg.as_ptr())
            },
            Err(Error::ParseFloat(s)) => unsafe {
                let msg = CString::new(s.to_string()).unwrap();
                raw::RedisModule_ReplyWithError.unwrap()(self.inner, msg.as_ptr())
            },
        };
        handle_status(status, "Could not reply")
    }

    pub fn call(&self, command: &str, args: &[&str], flags: &[CmdFmtFlags]) -> RedisResult {
        let terminated_args: Vec<RedisString> = args.iter().map(|s| RedisString::new(self.inner, s)).collect();

        let inner_args: Vec<*mut raw::RedisModuleString> =
            terminated_args.iter().map(|s| s.inner).collect();

        let cmd = CString::new(command).unwrap();
        let fmt = CString::new(CmdFmtFlags::multi(flags)).unwrap();

        let reply_: *mut raw::RedisModuleCallReply = unsafe {
            let p_call = raw::RedisModule_Call.unwrap();
            p_call(
                self.inner,
                cmd.as_ptr(),
                fmt.as_ptr(),
                inner_args.as_ptr() as *mut i8,
                terminated_args.len(),
            )
        };
        CallReply::new(reply_).into()
    }

    pub fn replicate(
        &self,
        command: &str,
        args: &[&str],
        flags: &[CmdFmtFlags],
    ) -> Result<(), Error> {
        let terminated_args: Vec<RedisString> = args.iter().map(|s| RedisString::new(self.inner, s)).collect();

        let inner_args: Vec<*mut raw::RedisModuleString> =
            terminated_args.iter().map(|s| s.inner).collect();

        let cmd = CString::new(command).unwrap();
        let fmt = CString::new(CmdFmtFlags::multi(flags)).unwrap();

        let result = unsafe {
            let p_call = raw::RedisModule_Replicate.unwrap();
            p_call(
                self.inner,
                cmd.as_ptr(),
                fmt.as_ptr(),
                inner_args.as_ptr() as *mut i8,
                terminated_args.len(),
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
    pub fn get_select_db(&self) -> i32 {
        unsafe { raw::RedisModule_GetSelectedDb.unwrap()(self.inner) as i32 }
    }
    pub fn get_context_flags(&self) -> u32 {
        unsafe { raw::RedisModule_GetContextFlags.unwrap()(self.inner) as u32 }
    }
    pub fn select_db(&self, newid: i32) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_SelectDb.unwrap()(self.inner, newid) },
            "Cloud not select db",
        )
    }
    pub fn open_read_key(&self, keyname: &str) -> ReadKey {
        ReadKey::create(self.inner, keyname)
    }
    pub fn open_write_key(&self, keyname: &str) -> WriteKey {
        WriteKey::create(self.inner, keyname)
    }
    pub fn set_cluster_flags(&self, flags: u64) {
        unsafe { raw::RedisModule_SetClusterFlags.unwrap()(self.inner, flags) }
    }
    pub fn block_client<F, G>(
        &self,
        _reply_callbck: F,
        _timeout_callback: F,
        _free_privdata: G,
        _timeout: Duration,
    ) -> BlockClient {
        unimplemented!()
    }
    pub fn is_blocked_reply_request(&self) -> bool {
        unimplemented!()
    }
    pub fn is_blocked_timeout_request(&self) -> bool {
        unimplemented!()
    }
    pub fn get_block_client_handle(&self) -> BlockClient {
        unimplemented!()
    }
    pub fn subscribe_to_keyspace_events<F>(&self, _types: KeySpaceTypes, _callback: F) {
        unimplemented!()
    }
    pub fn register_cluster_message_receiver<F>(&self, _msg_type: MsgType, _callback: F) {
        unimplemented!()
    }
    pub fn send_cluster_message(
        &self,
        _target_id: Option<ClusterNode>,
        _msg_type: MsgType,
        _msg: &str,
    ) -> Result<(), Error> {
        unimplemented!()
    }
    pub fn get_cluster_nodes_list() -> Option<ClusterNodeList> {
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

    pub fn create_timer<F, T>(&self, period: Duration, callback: F, data: T) -> TimerID
    where
        F: FnOnce(&Ctx, T),
    {
        let cb_data = CtxDataCallback { data, callback };

        // Store the user-provided data on the heap before passing ownership of it to Redis,
        // so that it will outlive the current scope.
        let data = Box::from(cb_data);

        // Take ownership of the data inside the box and obtain a raw pointer to pass to Redis.
        let data = Box::into_raw(data);

        let timer_id = unsafe {
            raw::RedisModule_CreateTimer.unwrap()(
                self.inner,
                period
                    .as_millis()
                    .try_into()
                    .expect("Value must fit in 64 bits"),
                Some(ctx_data_callback::<F, T>),
                data as *mut c_void,
            )
        };

        timer_id as TimerID
    }
    pub fn stop_timer<T>(&self, id: TimerID) -> Result<T, Error> {
        let mut data: *mut c_void = std::ptr::null_mut();

        handle_status(
            unsafe { raw::RedisModule_StopTimer.unwrap()(self.inner, id, &mut data) },
            "Cloud not stop timer",
        )?;

        let data: T = take_data(data);
        return Ok(data);
    }

    pub fn get_timer_info<T>(&self, id: TimerID) -> Result<(Duration, &T), Error> {
        let mut remaining: u64 = 0;
        let mut data: *mut c_void = std::ptr::null_mut();

        handle_status(
            unsafe {
                raw::RedisModule_GetTimerInfo.unwrap()(self.inner, id, &mut remaining, &mut data)
            },
            "Cloud not get timer info",
        )?;

        // Cast the *mut c_void supplied by the Redis API to a raw pointer of our custom type.
        let data = data as *mut T;

        // Dereference the raw pointer (we know this is safe, since Redis should return our
        // original pointer which we know to be good) and turn it into a safe reference
        let data = unsafe { &*data };

        Ok((Duration::from_millis(remaining), data))
    }
}

bitflags! {
    pub struct ClusterFlags: u64 {
        const NONE = raw:: REDISMODULE_CLUSTER_FLAG_NONE as u64;
        const NO_FAILOVER = raw::REDISMODULE_CLUSTER_FLAG_NO_FAILOVER as u64;
        const NO_REPLICATION = raw::REDISMODULE_CLUSTER_FLAG_NO_REDIRECTION as u64;
    }
}

extern "C" fn ctx_data_callback<F, T>(ctx: *mut raw::RedisModuleCtx, data: *mut c_void)
where
    F: FnOnce(&Ctx, T),
{
    let ctx = &Ctx::new(ctx);

    if data.is_null() {
        ctx.log_debug("[callback] Data must not null!");
        return;
    }

    let cb_data: CtxDataCallback<F, T> = take_data(data);
    (cb_data.callback)(ctx, cb_data.data);
}

#[repr(C)]
pub(crate) struct CtxDataCallback<F: FnOnce(&Ctx, T), T> {
    data: T,
    callback: F,
}

fn take_data<T>(data: *mut c_void) -> T {
    // Cast the *mut c_void supplied by the Redis API to a raw pointer of our custom type.
    let data = data as *mut T;

    // Take back ownership of the original boxed data, so we can unbox it safely.
    // If we don't do this, the data's memory will be leaked.
    let data = unsafe { Box::from_raw(data) };

    *data
}
