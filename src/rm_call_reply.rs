use crate::raw;
use crate::{Error, Result, Value, Ptr};
use std::slice;

#[repr(C)]
pub struct CallReply {
    inner: *mut raw::RedisModuleCallReply,
}

impl Ptr for CallReply {
    type PtrType = raw::RedisModuleCallReply;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.inner
    }
}

impl CallReply {
    pub fn from_ptr(inner: *mut raw::RedisModuleCallReply) -> Self {
        CallReply { inner }
    }
    pub fn get_type(&self) -> ReplyType {
        let x = unsafe { raw::RedisModule_CallReplyType.unwrap()(self.inner) as u32 };
        match x {
            raw::REDISMODULE_REPLY_STRING => ReplyType::String,
            raw::REDISMODULE_REPLY_ERROR => ReplyType::Error,
            raw::REDISMODULE_REPLY_INTEGER => ReplyType::Integer,
            raw::REDISMODULE_REPLY_ARRAY => ReplyType::Array,
            raw::REDISMODULE_REPLY_NULL => ReplyType::Null,
            _ => ReplyType::Unknown,
        }
    }
    pub fn get_string(&self) -> String {
        unsafe {
            let mut len = 0;
            let reply_string: *mut u8 =
                raw::RedisModule_CallReplyStringPtr.unwrap()(self.inner, &mut len) as *mut u8;
            String::from_utf8(
                slice::from_raw_parts(reply_string, len)
                    .into_iter()
                    .map(|v| *v)
                    .collect(),
            )
            .unwrap()
        }
    }
    pub fn get_integer(&self) -> i64 {
        unsafe { raw::RedisModule_CallReplyInteger.unwrap()(self.inner) }
    }
    pub fn get_array_element(&self, idx: usize) -> CallReply {
        CallReply::from_ptr(unsafe { raw::RedisModule_CallReplyArrayElement.unwrap()(self.inner, idx) })
    }
    pub fn get_length(&self) -> usize {
        unsafe { raw::RedisModule_CallReplyLength.unwrap()(self.inner) }
    }
}

impl Drop for CallReply {
    fn drop(&mut self) {
        unsafe { raw::RedisModule_FreeCallReply.unwrap()(self.inner) }
    }
}

impl Into<Result> for CallReply {
    fn into(self) -> Result {
        let reply_type = self.get_type();
        match reply_type {
            ReplyType::Error => Err(Error::generic(&self.get_string())),
            ReplyType::Unknown => Err(Error::generic("Error on method call")),
            ReplyType::Array => {
                let length = self.get_length();
                let mut vec = Vec::with_capacity(length);
                for i in 0..length {
                    let value: Result = self.get_array_element(i).into();
                    vec.push(value?)
                }
                Ok(Value::Array(vec))
            }
            ReplyType::Integer => Ok(Value::Integer(self.get_integer() as i64)),
            ReplyType::String => Ok(Value::SimpleString(self.get_string())),
            ReplyType::Null => Ok(Value::Null),
        }
    }
}

pub enum ReplyType {
    Unknown = raw::REDISMODULE_REPLY_UNKNOWN as isize,
    String = raw::REDISMODULE_REPLY_STRING as isize,
    Error = raw::REDISMODULE_REPLY_ERROR as isize,
    Integer = raw::REDISMODULE_REPLY_INTEGER as isize,
    Array = raw::REDISMODULE_REPLY_ARRAY as isize,
    Null = raw::REDISMODULE_REPLY_NULL as isize,
}