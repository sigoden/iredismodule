use bitflags::bitflags;
use num_traits::FromPrimitive;
use std::ops::Deref;
use std::os::raw::{c_int, c_void};
use std::time::Duration;

use crate::raw;
use crate::{handle_status, Ctx, Error, RedisString, RedisType};

pub struct ReadKey {
    pub inner: *mut raw::RedisModuleKey,
    ctx: *mut raw::RedisModuleCtx,
}

impl Drop for ReadKey {
    fn drop(&mut self) {
        unsafe { raw::RedisModule_CloseKey.unwrap()(self.inner) };
    }
}

impl ReadKey {
    pub fn new(ctx: *mut raw::RedisModuleCtx, keyname: &RedisString) -> Self {
        let mode = raw::REDISMODULE_READ as c_int;
        let inner = unsafe {
            raw::RedisModule_OpenKey.unwrap()(ctx, keyname.inner, mode) as *mut raw::RedisModuleKey
        };
        ReadKey { inner, ctx }
    }

    pub fn is_empty(&self) -> bool {
        let key_type: KeyType = unsafe { raw::RedisModule_KeyType.unwrap()(self.inner) }.into();
        key_type == KeyType::Empty
    }

    pub fn get_value<T>(&self, redis_type: &RedisType) -> Result<Option<&mut T>, Error> {
        self.verify_type(redis_type)?;
        let value = unsafe { raw::RedisModule_ModuleTypeGetValue.unwrap()(self.inner) as *mut T };

        if value.is_null() {
            return Ok(None);
        }

        let value = unsafe { &mut *value };
        Ok(Some(value))
    }

    pub fn verify_type(&self, redis_type: &RedisType) -> Result<(), Error> {
        let key_type: KeyType = unsafe { raw::RedisModule_KeyType.unwrap()(self.inner) }.into();
        let err = Error::generic("Existing key has wrong Redis type");
        if key_type != KeyType::Module {
            return Err(err);
        }
        let raw_type = unsafe { raw::RedisModule_ModuleTypeGetType.unwrap()(self.inner) };

        if raw_type != *redis_type.raw_type.borrow() {
            return Err(err);
        }
        Ok(())
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
    pub fn key_type(&self) -> Option<i32> {
        if self.inner.is_null() {
            None
        } else {
            Some(unsafe { raw::RedisModule_KeyType.unwrap()(self.inner) })
        }
    }
}

pub struct WriteKey {
    readkey: ReadKey,
}

impl AsRef<ReadKey> for WriteKey {
    fn as_ref(&self) -> &ReadKey {
        &self.readkey
    }
}

impl Deref for WriteKey {
    type Target = ReadKey;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl WriteKey {
    pub fn new(ctx: *mut raw::RedisModuleCtx, keyname: &RedisString) -> Self {
        let mode = (raw::REDISMODULE_READ | raw::REDISMODULE_WRITE) as c_int;
        let inner = unsafe {
            raw::RedisModule_OpenKey.unwrap()(ctx, keyname.inner, mode) as *mut raw::RedisModuleKey
        };
        WriteKey {
            readkey: ReadKey { inner, ctx },
        }
    }

    pub fn set_value<T>(&self, redis_type: &RedisType, value: T) -> Result<(), Error> {
        self.verify_type(redis_type)?;
        let value = Box::into_raw(Box::new(value)) as *mut c_void;
        handle_status(
            unsafe {
                raw::RedisModule_ModuleTypeSetValue.unwrap()(
                    self.inner,
                    *redis_type.raw_type.borrow(),
                    value,
                )
            },
            "Cloud not set value",
        )
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
    pub fn string_set(&mut self, value: &RedisString) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_StringSet.unwrap()(self.inner, value.inner) },
            "Cloud not set key string",
        )
    }
    pub fn string_dma<'a>(&mut self, _mode: i32) {
        unimplemented!()
    }
    pub fn string_truncate(&mut self, _newlen: u32) -> Result<(), Error> {
        unimplemented!()
    }
    pub fn list_push(&mut self, position: ListPosition, value: &str) -> Result<(), Error> {
        let value = Ctx::new(self.ctx).create_string(value);
        handle_status(
            unsafe { raw::RedisModule_ListPush.unwrap()(self.inner, position as i32, value.inner) },
            "Cloud not push list",
        )
    }
    pub fn list_push_rs(
        &mut self,
        position: ListPosition,
        value: &RedisString,
    ) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_ListPush.unwrap()(self.inner, position as i32, value.inner) },
            "Cloud not push list",
        )
    }
    pub fn list_pop(&mut self, pos: ListPosition) -> Result<RedisString, Error> {
        let p = unsafe { raw::RedisModule_ListPop.unwrap()(self.inner, pos as i32) };
        if p.is_null() {
            return Err(Error::generic("Cloud not pop list"));
        }
        Ok(RedisString::new(self.ctx, p))
    }
    pub fn zset_add(
        &mut self,
        _score: f64,
        _str: &RedisString,
        _flag: ZaddInputFlag,
    ) -> Result<ZaddOutputFlag, Error> {
        unimplemented!()
    }
}

pub enum ListPosition {
    Head = raw::REDISMODULE_LIST_HEAD as isize,
    Tail = raw::REDISMODULE_LIST_TAIL as isize,
}

bitflags! {
    pub struct AccessMode: i32 {
        const READ = raw::REDISMODULE_READ as i32;
        const WRITE = raw::REDISMODULE_WRITE as i32;
    }
}

pub enum ZaddInputFlag {}

pub enum ZaddOutputFlag {}

#[derive(Primitive, Debug, PartialEq)]
pub enum KeyType {
    Empty = raw::REDISMODULE_KEYTYPE_EMPTY as isize,
    String = raw::REDISMODULE_KEYTYPE_STRING as isize,
    List = raw::REDISMODULE_KEYTYPE_LIST as isize,
    Hash = raw::REDISMODULE_KEYTYPE_HASH as isize,
    Set = raw::REDISMODULE_KEYTYPE_SET as isize,
    ZSet = raw::REDISMODULE_KEYTYPE_ZSET as isize,
    Module = raw::REDISMODULE_KEYTYPE_MODULE as isize,
}

impl From<c_int> for KeyType {
    fn from(v: c_int) -> Self {
        KeyType::from_i32(v).unwrap()
    }
}
