use std::ops::Deref;
use std::os::raw::{c_int, c_void};
use std::time::Duration;

use crate::raw;
use crate::{handle_status, Context, Error, Ptr, RStr, RString, RType};

pub struct ReadKey {
    inner: *mut raw::RedisModuleKey,
    ctx: *mut raw::RedisModuleCtx,
}

impl Ptr for ReadKey {
    type PtrType = raw::RedisModuleKey;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.inner
    }
}

impl Drop for ReadKey {
    fn drop(&mut self) {
        unsafe { raw::RedisModule_CloseKey.unwrap()(self.inner) };
    }
}

impl ReadKey {
    pub fn from_redis_str(ctx: *mut raw::RedisModuleCtx, keyname: &RStr) -> Self {
        let mode = raw::REDISMODULE_READ as c_int;
        let inner = unsafe {
            raw::RedisModule_OpenKey.unwrap()(ctx, keyname.get_ptr(), mode)
                as *mut raw::RedisModuleKey
        };
        ReadKey { inner, ctx }
    }

    pub fn is_empty(&self) -> bool {
        let key_type = self.get_type();
        key_type == KeyType::Empty
    }

    pub fn get_value<T>(&self, redis_type: &RType<T>) -> Result<Option<&mut T>, Error> {
        let exist = self.verify_module_type(redis_type)?;
        if !exist {
            return Ok(None);
        }
        let value = unsafe { raw::RedisModule_ModuleTypeGetValue.unwrap()(self.inner) as *mut T };
        let value = unsafe { &mut *value };
        Ok(Some(value))
    }

    pub fn verify_type(&self, expect_type: KeyType, allow_null: bool) -> Result<(), Error> {
        let key_type = self.get_type();
        if key_type != expect_type {
            if !allow_null || key_type != KeyType::Empty {
                return Err(Error::WrongType);
            }
        }
        Ok(())
    }

    pub fn verify_module_type<T>(&self, redis_type: &RType<T>) -> Result<bool, Error> {
        let key_type = self.get_type();
        if key_type == KeyType::Empty {
            return Ok(false);
        }
        if key_type != KeyType::Module {
            return Err(Error::WrongType);
        }
        let raw_type = unsafe { raw::RedisModule_ModuleTypeGetType.unwrap()(self.inner) };

        if raw_type != *redis_type.raw_type.borrow() {
            return Err(Error::WrongType);
        }
        Ok(true)
    }

    pub fn string_get(&self) -> Result<RString, Error> {
        unsafe {
            let mut len = 0;
            let data = raw::RedisModule_StringDMA.unwrap()(
                self.inner,
                &mut len,
                raw::REDISMODULE_READ as c_int,
            ) as *mut u8;
            if data.is_null() {
                return Err(Error::generic("fail to execute string_dma"));
            }
            Ok(RString::from_raw_parts(self.ctx, data, len as usize))
        }
    }

    pub fn hash_get(&self, flag: HashGetFlag, field: &RStr) -> Result<Option<RString>, Error> {
        let value: *mut raw::RedisModuleString = std::ptr::null_mut();
        unsafe {
            handle_status(
                raw::RedisModule_HashGet.unwrap()(
                    self.inner,
                    flag.into(),
                    field.get_ptr(),
                    &value,
                    0,
                ),
                "fail to execute hash_get",
            )?;
        }
        if value.is_null() {
            return Ok(None);
        }
        Ok(Some(RString::new(self.ctx, value)))
    }
    pub fn zset_score_range(
        &self,
        dir: ZsetRangeDirection,
        min: f64,
        max: f64,
        min_exclude: bool,
        max_exclude: bool,
    ) -> Result<Vec<(RString, f64)>, Error> {
        let minex = min_exclude as i32;
        let maxex = max_exclude as i32;
        let mut result = vec![];
        unsafe {
            let (init, next) = {
                match dir {
                    ZsetRangeDirection::FristIn => (
                        raw::RedisModule_ZsetFirstInScoreRange.unwrap(),
                        raw::RedisModule_ZsetRangeNext.unwrap(),
                    ),
                    ZsetRangeDirection::LastIn => (
                        raw::RedisModule_ZsetLastInScoreRange.unwrap(),
                        raw::RedisModule_ZsetRangePrev.unwrap(),
                    ),
                }
            };
            let check_end = raw::RedisModule_ZsetRangeEndReached.unwrap();
            let get_elem = raw::RedisModule_ZsetRangeCurrentElement.unwrap();
            handle_status(
                init(self.inner, min, max, minex, maxex),
                "fail to execute zset_score_range",
            )?;
            while check_end(self.inner) == 0 {
                let mut score = 0.0;
                let elem = get_elem(self.inner, &mut score);
                result.push((RString::new(self.ctx, elem), score));
                next(self.inner);
            }
            raw::RedisModule_ZsetRangeStop.unwrap()(self.inner);
        }
        Ok(result)
    }
    pub fn zset_lex_range(
        &self,
        dir: ZsetRangeDirection,
        min: &RStr,
        max: &RStr,
    ) -> Result<Vec<(RString, f64)>, Error> {
        let mut result = vec![];
        unsafe {
            let (init, next) = {
                match dir {
                    ZsetRangeDirection::FristIn => (
                        raw::RedisModule_ZsetFirstInLexRange.unwrap(),
                        raw::RedisModule_ZsetRangeNext.unwrap(),
                    ),
                    ZsetRangeDirection::LastIn => (
                        raw::RedisModule_ZsetLastInLexRange.unwrap(),
                        raw::RedisModule_ZsetRangePrev.unwrap(),
                    ),
                }
            };
            let ctx = self.get_context();
            ctx.log_debug(&format!(
                "dir = {:?} min={} max={}",
                dir,
                min.to_str().unwrap(),
                max.to_str().unwrap()
            ));
            let check_end = raw::RedisModule_ZsetRangeEndReached.unwrap();
            let get_elem = raw::RedisModule_ZsetRangeCurrentElement.unwrap();
            handle_status(
                init(self.inner, min.get_ptr(), max.get_ptr()),
                "fail to execute zset_lex_range",
            )?;
            ctx.log_debug(&format!("range start"));
            while check_end(self.inner) == 0 {
                let mut score = 0.0;
                ctx.log_debug(&format!("range step"));
                let elem = get_elem(self.inner, &mut score);
                result.push((RString::new(self.ctx, elem), score));
                next(self.inner);
            }
            ctx.log_debug(&format!("range stop"));
            raw::RedisModule_ZsetRangeStop.unwrap()(self.inner)
        }
        Ok(result)
    }
    pub fn value_length(&self) -> usize {
        unsafe { raw::RedisModule_ValueLength.unwrap()(self.inner) }
    }
    pub fn get_expire(&self) -> Option<Duration> {
        let result: i64 = unsafe { raw::RedisModule_GetExpire.unwrap()(self.inner) };
        if result == raw::REDISMODULE_NO_EXPIRE as i64 {
            None
        } else {
            Some(Duration::from_millis(result as u64))
        }
    }
    pub fn get_type(&self) -> KeyType {
        let v = unsafe { raw::RedisModule_KeyType.unwrap()(self.inner) as u32 };
        match v {
            raw::REDISMODULE_KEYTYPE_EMPTY => KeyType::Empty,
            raw::REDISMODULE_KEYTYPE_STRING => KeyType::String,
            raw::REDISMODULE_KEYTYPE_LIST => KeyType::List,
            raw::REDISMODULE_KEYTYPE_HASH => KeyType::Hash,
            raw::REDISMODULE_KEYTYPE_SET => KeyType::Set,
            raw::REDISMODULE_KEYTYPE_ZSET => KeyType::ZSet,
            raw::REDISMODULE_KEYTYPE_MODULE => KeyType::Module,
            _ => panic!("unknown key type"),
        }
    }
    fn get_context(&self) -> Context {
        Context::from_ptr(self.ctx)
    }
}

pub struct WriteKey {
    read_key: ReadKey,
}

impl AsRef<ReadKey> for WriteKey {
    fn as_ref(&self) -> &ReadKey {
        &self.read_key
    }
}

impl Deref for WriteKey {
    type Target = ReadKey;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl WriteKey {
    pub fn from_redis_str(ctx: *mut raw::RedisModuleCtx, keyname: &RStr) -> Self {
        let mode = (raw::REDISMODULE_READ | raw::REDISMODULE_WRITE) as c_int;
        let inner = unsafe {
            raw::RedisModule_OpenKey.unwrap()(ctx, keyname.get_ptr(), mode)
                as *mut raw::RedisModuleKey
        };
        WriteKey {
            read_key: ReadKey { inner, ctx },
        }
    }

    pub fn set_value<T>(&mut self, redis_type: &RType<T>, value: T) -> Result<&mut T, Error> {
        let value = Box::into_raw(Box::new(value)) as *mut c_void;
        handle_status(
            unsafe {
                raw::RedisModule_ModuleTypeSetValue.unwrap()(
                    self.inner,
                    *redis_type.raw_type.borrow(),
                    value,
                )
            },
            "fail to execute set_value",
        )?;
        Ok(unsafe { &mut *(value as *mut T) })
    }

    pub fn delete(&mut self) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_DeleteKey.unwrap()(self.inner) },
            "fail to execute delete",
        )
    }
    pub fn unlink(&mut self) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_UnlinkKey.unwrap()(self.inner) },
            "fail to execute unlink",
        )
    }
    pub fn set_expire(&mut self, expire: Duration) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_SetExpire.unwrap()(self.inner, expire.as_millis() as i64) },
            "fail to execute set_expire",
        )
    }
    pub fn string_set(&mut self, value: &RStr) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_StringSet.unwrap()(self.inner, value.get_ptr()) },
            "fail to execute string_set",
        )
    }
    pub fn list_push(&mut self, position: ListPosition, value: &RStr) -> Result<(), Error> {
        handle_status(
            unsafe {
                raw::RedisModule_ListPush.unwrap()(self.inner, position as i32, value.get_ptr())
            },
            "fail to execute list_push",
        )
    }
    pub fn list_pop(&mut self, pos: ListPosition) -> Result<RString, Error> {
        let p = unsafe { raw::RedisModule_ListPop.unwrap()(self.inner, pos as i32) };
        if p.is_null() {
            return Err(Error::generic("fail to pop list"));
        }
        Ok(RString::new(self.ctx, p))
    }
    pub fn hash_set(&self, flag: HashSetFlag, field: &RStr, value: &RStr) -> Result<(), Error> {
        unsafe {
            handle_status(
                raw::RedisModule_HashSet.unwrap()(
                    self.inner,
                    flag.into(),
                    field.get_ptr(),
                    value.get_ptr(),
                    0,
                ),
                "fail to execute hash_set",
            )?;
        }
        Ok(())
    }
    pub fn zset_add(
        &self,
        score: f64,
        ele: &RStr,
        flag: ZaddInputFlag,
    ) -> Result<ZaddOuputFlag, Error> {
        let out_flag;
        unsafe {
            let mut flag = flag as c_int;
            handle_status(
                raw::RedisModule_ZsetAdd.unwrap()(self.inner, score, ele.get_ptr(), &mut flag),
                "fail to execute zset_add",
            )?;
            out_flag = flag.into();
        }
        Ok(out_flag)
    }
    pub fn zset_incrby(
        &self,
        ele: &RStr,
        score: f64,
        flag: ZaddInputFlag,
    ) -> Result<(ZaddOuputFlag, f64), Error> {
        let out_flag;
        let mut new_score = 0.0;
        unsafe {
            let mut flag = flag as c_int;
            handle_status(
                raw::RedisModule_ZsetIncrby.unwrap()(
                    self.inner,
                    score,
                    ele.get_ptr(),
                    &mut flag,
                    &mut new_score,
                ),
                "fail to execute zset_incrby",
            )?;
            out_flag = flag.into();
        }
        Ok((out_flag, new_score))
    }
    pub fn zset_rem(&self, ele: &RStr) -> Result<bool, Error> {
        let mut flag = 0;
        unsafe {
            handle_status(
                raw::RedisModule_ZsetRem.unwrap()(self.inner, ele.get_ptr(), &mut flag),
                "fail to execute zset_rem",
            )?;
        }
        let result = if flag == 0 { false } else { true };
        Ok(result)
    }
    pub fn zset_score(&self, ele: &RStr) -> Result<f64, Error> {
        unsafe {
            let mut score = 0.0;
            handle_status(
                raw::RedisModule_ZsetScore.unwrap()(self.inner, ele.get_ptr(), &mut score),
                "fail to execute zset_score",
            )?;
            Ok(score)
        }
    }
}

pub enum ListPosition {
    Head = raw::REDISMODULE_LIST_HEAD as isize,
    Tail = raw::REDISMODULE_LIST_TAIL as isize,
}

#[derive(Debug, PartialEq)]
pub enum KeyType {
    Empty = raw::REDISMODULE_KEYTYPE_EMPTY as isize,
    String = raw::REDISMODULE_KEYTYPE_STRING as isize,
    List = raw::REDISMODULE_KEYTYPE_LIST as isize,
    Hash = raw::REDISMODULE_KEYTYPE_HASH as isize,
    Set = raw::REDISMODULE_KEYTYPE_SET as isize,
    ZSet = raw::REDISMODULE_KEYTYPE_ZSET as isize,
    Module = raw::REDISMODULE_KEYTYPE_MODULE as isize,
}

#[derive(Debug, PartialEq)]
pub enum HashSetFlag {
    Normal,
    NX,
    XX,
}

impl Into<c_int> for HashSetFlag {
    fn into(self) -> c_int {
        match self {
            HashSetFlag::Normal => raw::REDISMODULE_HASH_NONE as c_int,
            HashSetFlag::NX => raw::REDISMODULE_HASH_NX as c_int,
            HashSetFlag::XX => raw::REDISMODULE_HASH_XX as c_int,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum HashGetFlag {
    Normal,
    Exists,
}

impl Into<c_int> for HashGetFlag {
    fn into(self) -> c_int {
        match self {
            HashGetFlag::Normal => raw::REDISMODULE_HASH_NONE as c_int,
            HashGetFlag::Exists => raw::REDISMODULE_HASH_EXISTS as c_int,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ZsetRangeDirection {
    FristIn,
    LastIn,
}

#[derive(Debug, PartialEq)]
pub enum ZaddInputFlag {
    XX = raw::REDISMODULE_ZADD_XX as isize,
    NX = raw::REDISMODULE_ZADD_NX as isize,
}

#[derive(Debug, PartialEq)]
pub enum ZaddOuputFlag {
    Added,
    Updated,
    Nop,
}

impl From<c_int> for ZaddOuputFlag {
    fn from(flag: c_int) -> Self {
        match flag as u32 {
            raw::REDISMODULE_ZADD_ADDED => ZaddOuputFlag::Added,
            raw::REDISMODULE_ZADD_UPDATED => ZaddOuputFlag::Updated,
            raw::REDISMODULE_ZADD_NOP => ZaddOuputFlag::Nop,
            _ => panic!("invalid zadd flag"),
        }
    }
}
