use crate::raw;
use crate::rm;
use std::ffi::CString;
use std::time::Duration;
use std::os::raw::{c_long};
use crate::{
    Str,
    RedisValue,
    RedisResult,
    CallReply,
    Error,
    CtxFlags,
    ReadKey,
    WriteKey,
    TimerID,
    ClusterNodeList,
    ClusterNode,
    MsgType,
    KeySpaceTypes,
    BlockClient,
    CmdFmtFlags,
};

pub struct Ctx {
    pub inner: *mut raw::RedisModuleCtx,
}

impl Ctx {
    pub fn create(inner: *mut raw::RedisModuleCtx) -> Self {
        Ctx { inner }
    }
    pub fn is_keys_position_request(&self) -> Result<(), Error> {
        unimplemented!()
    }
    pub fn Key_at_pos(&self) {
        unimplemented!()
    }
    pub fn is_module_busy(&self) -> Result<(), Error> {
        unimplemented!()
    }
    pub fn auto_memory(&self) {
        unimplemented!()
    }
    pub fn reply(&self, r: RedisResult) -> Result<(), Error> {
        let status = match r {
            Ok(RedisValue::Integer(v)) => unsafe {
                raw::RedisModule_ReplyWithLongLong.unwrap()(self.inner, v)
            },

            Ok(RedisValue::Float(v)) => unsafe {
                raw::RedisModule_ReplyWithDouble.unwrap()(self.inner, v)
            },

            Ok(RedisValue::SimpleStringStatic(s)) => unsafe {
                let msg = CString::new(s).unwrap();
                raw::RedisModule_ReplyWithSimpleString.unwrap()(self.inner, msg.as_ptr())
            },

            Ok(RedisValue::SimpleString(s)) => unsafe {
                let msg = CString::new(s).unwrap();
                raw::RedisModule_ReplyWithSimpleString.unwrap()(self.inner, msg.as_ptr())
            },

            Ok(RedisValue::BulkString(s)) => unsafe {
                raw::RedisModule_ReplyWithString.unwrap()(
                    self.inner,
                    Str::create(self.inner, s.as_ref()).inner,
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

                rm::OK
            }

            Ok(RedisValue::Null) => unsafe {
                raw::RedisModule_ReplyWithNull.unwrap()(self.inner)
            },

            Ok(RedisValue::NoReply) => rm::OK,

            Err(Error::WrongArity) => {
                match self.is_keys_position_request() {
                    Ok(_) => rm::ERR,
                    Err(_) => unsafe {
                        raw::RedisModule_WrongArity.unwrap()(self.inner)
                    }
                }
            },
            Err(Error::Generic(s)) => unsafe {
                let msg = CString::new(s.to_string()).unwrap();
                raw::RedisModule_ReplyWithError.unwrap()(self.inner, msg.as_ptr())
            },
            Err(Error::FromUtf8(s)) => unsafe {
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
        rm::handle_status(status, "Could not reply")
    }


    pub fn call(&self, command: &str, args: &[&str], flags: &[CmdFmtFlags]) -> RedisResult {
        let terminated_args: Vec<Str> = args
            .iter()
            .map(|s| Str::create(self.inner, s))
            .collect();

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
        CallReply::create(reply_).into()
    }

    pub fn replicate(&self) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn replicate_verbatim(&self) {
        unimplemented!()
    }
    pub fn get_client_id(&self) -> u128 {
        unimplemented!()
    }
    pub fn get_select_db(&self) -> u16 {
        unimplemented!()
    }
    pub fn select_db(&self, newid: u16) -> Result<(), Error> {
        unimplemented!()
    }
    pub fn get_context_flags(&self) -> CtxFlags {
        unsafe { CtxFlags::new(raw::RedisModule_GetContextFlags.unwrap()(self.inner) as u32) }
    }
    pub fn open_read_key(&self, keyname: &str) -> ReadKey {
        unimplemented!()
    }
    pub fn open_write_key(&self, keyname: &str) -> WriteKey {
        unimplemented!()
    }
    pub fn set_cluster_flags() {
        unimplemented!()
    }
    pub fn block_client<F, G>(&self, reply_callbck: F, timeout_callback: F, free_privdata: G, timeout: Duration) -> BlockClient {
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
    pub fn subscribe_to_keyspace_events<F>(&self, types: KeySpaceTypes, callback: F) {
        unimplemented!()
    }
    pub fn register_cluster_message_receiver<F>(&self, msg_type: MsgType, callback: F) {
        unimplemented!()
    }
    pub fn send_cluster_message(&self, target_id: Option<ClusterNode>, msg_type: MsgType, msg: &str) -> Result<(), Error> {
        unimplemented!()
    }
    pub fn get_cluster_nodes_list() -> Option<ClusterNodeList> {
        unimplemented!()
    }
    pub fn create_timer<F, T>(&self, period: Duration, callback: F, data: T) -> TimerID {
        unimplemented!()
    }
    pub fn stop_timer<T>(&self, id: TimerID) ->  Result<T, Error> {
        unimplemented!()
    }
    pub fn get_timer_info<T>(&self, id: TimerID) -> Result<(Duration, T), Error> {
        unimplemented!()
    }
}