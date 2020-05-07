use crate::num_traits::FromPrimitive;
use crate::raw;
use crate::{Ctx, handle_status, Error, Str};
use std::time::Duration;
use std::os::raw::{c_int};

pub struct ReadKey {
    pub inner: *mut raw::RedisModuleKey,
    ctx: *mut raw::RedisModuleCtx,
}

impl Drop for ReadKey {
    fn drop(&mut self) {
        unsafe { raw::RedisModule_CloseKey.unwrap()(self.inner) }
    }
}

impl ReadKey {
    pub fn value_length(&self) -> u32 {
        // RedisModule_ValueLength
        unimplemented!()
    }
    pub fn get_expire(&self) -> Option<Duration> {
        // RedisModule_GetExpire
        unimplemented!()
    }
    pub fn set_expire(&mut self, expire: Duration) -> Result<(), Error> {
        // RedisModule_SetExpire
        unimplemented!()
    }
    pub fn string_set(&mut self, str: &str) -> Result<(), Error> {
        unimplemented!()
    }
    pub fn string_dma(&mut self) {
        unimplemented!()
    }
    pub fn string_truncate(&mut self, newlen: u32) -> Result<(), Error> {
        unimplemented!()
    }
    pub fn list_push(&mut self, pos: ListWhere, str: &str) -> Result<(), Error> {
        unimplemented!()
    }
    pub fn list_pop(&mut self, pos: ListWhere) -> Result<Str, Error> {
        unimplemented!()
    }
    pub fn zset_add(&mut self, score: f64, str: &Str, flag: ZaddInputFlag) -> Result<ZaddOutputFlag, Error> {
        unimplemented!()
    }
}

pub struct WriteKey {
    pub inner: *mut raw::RedisModuleKey,
    ctx: *mut raw::RedisModuleCtx,
}

impl Drop for WriteKey {
    fn drop(&mut self) {
        unsafe { raw::RedisModule_CloseKey.unwrap()(self.inner) }
    }
}

impl WriteKey {
    pub fn delete(&mut self) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_DeleteKey.unwrap()(self.inner) },
            "Could not delete key"
        )
    }
    pub fn unlink(&mut self) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_UnlinkKey.unwrap()(self.inner) },
            "Could not unlink key"
        )
    }
}

pub fn key_type(key: *mut raw::RedisModuleKey) -> Option<KeyType> {
    if key.is_null() {
        None
    } else {
        Some(unsafe {
            raw::RedisModule_KeyType.unwrap()(key)
        }.into())
    }
}

const REDISMODULE_READ_ISIZE: isize = raw::REDISMODULE_READ as isize;
const REDISMODULE_WRITE_ISIZE: isize = raw::REDISMODULE_WRITE as isize;

#[derive(Primitive, Debug, PartialEq)]
pub enum KeyType {
    Read = REDISMODULE_READ_ISIZE,
    Write = REDISMODULE_WRITE_ISIZE,
}

impl From<c_int> for KeyType {
    fn from(v: c_int) -> Self {
        KeyType::from_i32(v).unwrap()
    }
}


const REDISMODULE_LIST_HEAD_SIZE: isize = raw::REDISMODULE_LIST_HEAD as isize;
const REDISMODULE_LIST_TAIL_SIZE: isize = raw::REDISMODULE_LIST_TAIL as isize;

#[derive(Primitive, Debug, PartialEq)]
pub enum ListWhere {
    Head = REDISMODULE_LIST_HEAD_SIZE,
    Tail = REDISMODULE_LIST_TAIL_SIZE,
}

impl From<c_int> for ListWhere {
    fn from(v: c_int) -> Self {
        ListWhere::from_i32(v).unwrap()
    }
}

pub enum ZaddInputFlag {

}

pub enum ZaddOutputFlag {

}