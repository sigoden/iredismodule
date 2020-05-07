use crate::num_traits::FromPrimitive;
use crate::raw;
use crate::{handle_status, Error, Str};

use std::os::raw::c_int;
use std::time::Duration;

pub struct ReadKey {
    pub inner: *mut raw::RedisModuleKey,
    ctx: *mut raw::RedisModuleCtx,
    keyname: Str,
}

impl Drop for ReadKey {
    fn drop(&mut self) {
        unsafe { raw::RedisModule_CloseKey.unwrap()(self.inner) }
    }
}

impl ReadKey {
    pub fn create(ctx: *mut raw::RedisModuleCtx, keyname: &str) -> Self {
        let keyname = Str::create(ctx, keyname);
        let mode = raw::REDISMODULE_READ as c_int;
        let inner = unsafe {
            raw::RedisModule_OpenKey.unwrap()(ctx, keyname.inner, mode) as *mut raw::RedisModuleKey
        };
        ReadKey {
            inner,
            ctx,
            keyname,
        }
    }
    pub fn value_length(&self) -> usize {
        unsafe { raw::RedisModule_ValueLength.unwrap()(self.inner) }
    }
    pub fn get_expire(&self) -> Option<Duration> {
        let result: i64 = unsafe { raw::RedisModule_GetExpire.unwrap()(self.inner) };
        if result != 0 {
            None
        } else {
            Some(Duration::from_millis(result as u64))
        }
    }
}

pub struct WriteKey {
    pub inner: *mut raw::RedisModuleKey,
    ctx: *mut raw::RedisModuleCtx,
    keyname: Str,
}

impl Drop for WriteKey {
    fn drop(&mut self) {
        unsafe { raw::RedisModule_CloseKey.unwrap()(self.inner) }
    }
}

impl WriteKey {
    pub fn create(ctx: *mut raw::RedisModuleCtx, keyname: &str) -> Self {
        let keyname = Str::create(ctx, keyname);
        let mode = (raw::REDISMODULE_READ | raw::REDISMODULE_WRITE) as c_int;
        let inner = unsafe {
            raw::RedisModule_OpenKey.unwrap()(ctx, keyname.inner, mode) as *mut raw::RedisModuleKey
        };
        WriteKey {
            inner,
            ctx,
            keyname,
        }
    }
    pub fn delete(&mut self) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_DeleteKey.unwrap()(self.inner) },
            "Could not delete key",
        )
    }
    pub fn unlink(&mut self) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_UnlinkKey.unwrap()(self.inner) },
            "Could not unlink key",
        )
    }
    pub fn set_expire(&mut self, expire: Duration) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_SetExpire.unwrap()(self.inner, expire.as_millis() as i64) },
            "Could not set expire",
        )
    }
    pub fn string_set(&mut self, _str: &str) -> Result<(), Error> {
        unimplemented!()
    }
    pub fn string_dma(&mut self) {
        unimplemented!()
    }
    pub fn string_truncate(&mut self, _newlen: u32) -> Result<(), Error> {
        unimplemented!()
    }
    pub fn list_push(&mut self, _pos: ListWhere, _str: &str) -> Result<(), Error> {
        unimplemented!()
    }
    pub fn list_pop(&mut self, _pos: ListWhere) -> Result<Str, Error> {
        unimplemented!()
    }
    pub fn zset_add(
        &mut self,
        _score: f64,
        _str: &Str,
        _flag: ZaddInputFlag,
    ) -> Result<ZaddOutputFlag, Error> {
        unimplemented!()
    }
}

pub fn key_type(key: *mut raw::RedisModuleKey) -> Option<KeyType> {
    if key.is_null() {
        None
    } else {
        Some(unsafe { raw::RedisModule_KeyType.unwrap()(key) }.into())
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

pub enum ZaddInputFlag {}

pub enum ZaddOutputFlag {}
