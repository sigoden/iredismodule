use crate::raw;
use crate::{Error, RedisResult, RedisValue, Ptr};
use num_traits::FromPrimitive;
use std::os::raw::c_int;
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
    pub fn new(inner: *mut raw::RedisModuleCallReply) -> Self {
        CallReply { inner }
    }
    pub fn get_type(&self) -> ReplyType {
        unsafe { raw::RedisModule_CallReplyType.unwrap()(self.inner).into() }
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
        CallReply::new(unsafe { raw::RedisModule_CallReplyArrayElement.unwrap()(self.inner, idx) })
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

impl Into<RedisResult> for CallReply {
    fn into(self) -> RedisResult {
        let reply_type = self.get_type();
        match reply_type {
            ReplyType::Error => Err(Error::generic(&self.get_string())),
            ReplyType::Unknown => Err(Error::generic("Error on method call")),
            ReplyType::Array => {
                let length = self.get_length();
                let mut vec = Vec::with_capacity(length);
                for i in 0..length {
                    let value: RedisResult = self.get_array_element(i).into();
                    vec.push(value?)
                }
                Ok(RedisValue::Array(vec))
            }
            ReplyType::Integer => Ok(RedisValue::Integer(self.get_integer() as i64)),
            ReplyType::String => Ok(RedisValue::SimpleString(self.get_string())),
            ReplyType::Null => Ok(RedisValue::Null),
        }
    }
}

const REDISMODULE_REPLY_UNKNOWN_ISIZE: isize = raw::REDISMODULE_REPLY_UNKNOWN as isize;
const REDISMODULE_REPLY_STRING_ISIZE: isize = raw::REDISMODULE_REPLY_STRING as isize;
const REDISMODULE_REPLY_ERROR_ISIZE: isize = raw::REDISMODULE_REPLY_ERROR as isize;
const REDISMODULE_REPLY_INTEGER_ISIZE: isize = raw::REDISMODULE_REPLY_INTEGER as isize;
const REDISMODULE_REPLY_ARRAY_ISIZE: isize = raw::REDISMODULE_REPLY_ARRAY as isize;
const REDISMODULE_REPLY_NULL_ISIZE: isize = raw::REDISMODULE_REPLY_NULL as isize;

#[derive(Primitive, Debug, PartialEq)]
pub enum ReplyType {
    Unknown = REDISMODULE_REPLY_UNKNOWN_ISIZE,
    String = REDISMODULE_REPLY_STRING_ISIZE,
    Error = REDISMODULE_REPLY_ERROR_ISIZE,
    Integer = REDISMODULE_REPLY_INTEGER_ISIZE,
    Array = REDISMODULE_REPLY_ARRAY_ISIZE,
    Null = REDISMODULE_REPLY_NULL_ISIZE,
}

impl From<c_int> for ReplyType {
    fn from(v: c_int) -> Self {
        ReplyType::from_i32(v).unwrap()
    }
}
