//! Handle redis call reply

use crate::error::Error;
use crate::raw;
use crate::value::Value;
use crate::{FromPtr, GetPtr, RResult};

/// Wrap the pointer of a RedisModuleCallReply
#[repr(C)]
pub struct CallReply {
    ptr: *mut raw::RedisModuleCallReply,
}

impl GetPtr for CallReply {
    type PtrType = raw::RedisModuleCallReply;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.ptr
    }
}

impl FromPtr for CallReply {
    type PtrType = raw::RedisModuleCallReply;
    fn from_ptr(ptr: *mut raw::RedisModuleCallReply) -> CallReply {
        CallReply { ptr }
    }
}

impl CallReply {
    /// Return the reply type
    pub fn get_type(&self) -> ReplyType {
        let x = unsafe { raw::RedisModule_CallReplyType.unwrap()(self.ptr) as u32 };
        match x {
            raw::REDISMODULE_REPLY_STRING => ReplyType::String,
            raw::REDISMODULE_REPLY_ERROR => ReplyType::Error,
            raw::REDISMODULE_REPLY_INTEGER => ReplyType::Integer,
            raw::REDISMODULE_REPLY_ARRAY => ReplyType::Array,
            raw::REDISMODULE_REPLY_NULL => ReplyType::Null,
            _ => ReplyType::Unknown,
        }
    }
    /// Get the string value from a string type reply
    pub fn get_string(&self) -> String {
        if self.get_type() != ReplyType::String {
            panic!("Reply type is not string")
        }
        let buf = self.get_proto();
        if buf[0] == 36 {
            String::from_utf8(proto_to_buff_string(buf)).unwrap()
        } else {
            proto_to_string(buf)
        }
    }
    /// Return the bulk string buffer
    pub fn get_bulk_string(&self) -> Vec<u8> {
        if self.get_type() != ReplyType::String {
            panic!("Reply type is not bulk string")
        }
        proto_to_buff_string(self.get_proto())
    }
    /// Return the double value
    pub fn get_double(&self) -> f64 {
        if self.get_type() != ReplyType::String {
            panic!("Reply type is not bulk string")
        }
        let value = proto_to_buff_string(self.get_proto());
        let value_str = std::str::from_utf8(&value).unwrap();
        value_str.parse::<f64>().unwrap()
    }
    /// Get the integer value from a integer type reply
    pub fn get_integer(&self) -> i64 {
        if self.get_type() != ReplyType::Integer {
            panic!("Reply type is not integer")
        }
        unsafe { raw::RedisModule_CallReplyInteger.unwrap()(self.ptr) }
    }
    /// Return the 'idx'-th nested call reply element of an array reply, or None
    /// if the reply type is wrong or the index is out of range
    pub fn get_array_element(&self, idx: usize) -> Option<CallReply> {
        let ptr = unsafe { raw::RedisModule_CallReplyArrayElement.unwrap()(self.ptr, idx) };
        if ptr.is_null() {
            None
        } else {
            Some(CallReply::from_ptr(ptr))
        }
    }
    // Return raw proto buffer
    pub fn get_proto(&self) -> Vec<u8> {
        unsafe { 
            let mut len = 0;
            let ptr = raw::RedisModule_CallReplyProto.unwrap()(self.ptr, &mut len);
            let data: &[u8] = std::slice::from_raw_parts(ptr as *const u8, len);
            data.to_vec()
        }
    }
    /// Return the length of array reply type.
    ///
    /// If the reply is not array type, 0 is returned
    pub fn get_length(&self) -> usize {
        unsafe { raw::RedisModule_CallReplyLength.unwrap()(self.ptr) }
    }
}

impl Drop for CallReply {
    fn drop(&mut self) {
        unsafe { raw::RedisModule_FreeCallReply.unwrap()(self.ptr) }
    }
}

impl Into<RResult> for CallReply {
    fn into(self) -> RResult {
        let reply_type = self.get_type();
        match reply_type {
            ReplyType::Error => Err(Error::new(&self.get_string())),
            ReplyType::Unknown => Err(Error::new("Unkown reply type")),
            ReplyType::Array => {
                let length = self.get_length();
                let mut vec = Vec::with_capacity(length);
                for i in 0..length {
                    let value: RResult = self.get_array_element(i).unwrap().into();
                    vec.push(value?)
                }
                Ok(Value::Array(vec))
            }
            ReplyType::Integer => Ok(Value::Integer(self.get_integer() as i64)),
            ReplyType::String => {
                let buf = self.get_proto();
                if buf[0] == 36 { // bulk string
                    Ok(Value::BulkString(proto_to_buff_string(buf)))
                } else if buf[0] == 43 { // simple string
                    Ok(Value::String(proto_to_string(buf)))
                } else {
                    Err(Error::new("Invalid string reply"))
                }
            },
            ReplyType::Null => Ok(Value::Null),
        }
    }
}

fn proto_to_buff_string(buf: Vec<u8>) -> Vec<u8> {
    if buf[1] == 45 { // empty bulk string
        return vec![];
    }
    let len_buf = buf.iter().skip(1).take_while(|v| **v != 13).cloned().collect::<Vec<u8>>();
    let len: usize = String::from_utf8(len_buf).unwrap().parse().unwrap();
    buf
        .into_iter()
        .skip_while(|v| *v != 10)
        .skip(1)
        .take(len)
        .collect()
}

fn proto_to_string(buf: Vec<u8>) -> String {
    let value: Vec<u8> = buf.into_iter().skip(1).take_while(|v| *v != 13).collect();
    std::str::from_utf8(&value).unwrap().to_owned()
}

#[derive(Debug, PartialEq)]
/// Kind of reply type
pub enum ReplyType {
    Unknown = raw::REDISMODULE_REPLY_UNKNOWN as isize,
    String = raw::REDISMODULE_REPLY_STRING as isize,
    Error = raw::REDISMODULE_REPLY_ERROR as isize,
    Integer = raw::REDISMODULE_REPLY_INTEGER as isize,
    Array = raw::REDISMODULE_REPLY_ARRAY as isize,
    Null = raw::REDISMODULE_REPLY_NULL as isize,
}
