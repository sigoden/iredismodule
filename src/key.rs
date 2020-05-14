//! A implementation of Redis key
use std::ops::Deref;
use std::os::raw::{c_int, c_void};
use std::time::Duration;

use crate::context::Context;
use crate::error::Error;
use crate::raw;
use crate::rtype::RType;
use crate::scan_cursor::ScanCursor;
use crate::string::{RStr, RString};
use crate::{handle_status, FromPtr, GetPtr};

/// Repersent a Redis key with read permision
///
/// create with [`ctx.open_read_key`](./context/struct.Context.html#method.open_read_key)
pub struct ReadKey {
    ptr: *mut raw::RedisModuleKey,
    ctx: *mut raw::RedisModuleCtx,
}

impl GetPtr for ReadKey {
    type PtrType = raw::RedisModuleKey;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.ptr
    }
}

impl Drop for ReadKey {
    fn drop(&mut self) {
        unsafe { raw::RedisModule_CloseKey.unwrap()(self.ptr) };
    }
}

impl ReadKey {
    pub fn new(ctx: *mut raw::RedisModuleCtx, keyname: &RStr) -> Self {
        let mode = raw::REDISMODULE_READ as c_int;
        let ptr = unsafe {
            raw::RedisModule_OpenKey.unwrap()(ctx, keyname.get_ptr(), mode)
                as *mut raw::RedisModuleKey
        };
        ReadKey { ptr, ctx }
    }

    /// Where the key pointer is NULL
    pub fn is_empty(&self) -> bool {
        let key_type = self.get_type();
        key_type == KeyType::Empty
    }

    /// Assuming `get_type` returned REDISMODULE_KEYTYPE_MODULE on
    /// the key, returns the module type low-level value stored at key, as
    /// it was set by the user via `set_value`.
    pub fn get_value<T>(&self, redis_type: &RType<T>) -> Result<Option<&mut T>, Error> {
        let exist = self.verify_module_type(redis_type)?;
        if !exist {
            return Ok(None);
        }
        let value = unsafe { raw::RedisModule_ModuleTypeGetValue.unwrap()(self.ptr) as *mut T };
        let value = unsafe { &mut *value };
        Ok(Some(value))
    }

    /// Check the key type.
    ///
    /// When `allow_null` is set, key have no value will pass the check.
    pub fn verify_type(&self, expect_type: KeyType, allow_null: bool) -> Result<(), Error> {
        let key_type = self.get_type();
        if key_type != expect_type {
            if !allow_null || key_type != KeyType::Empty {
                return Err(Error::WrongType);
            }
        }
        Ok(())
    }

    /// Check the type of key is `KeyType::Module` and the it's specifi module type is `redis_type`
    ///
    /// The bool indicate whether the value is empty
    pub fn verify_module_type<T>(&self, redis_type: &RType<T>) -> Result<bool, Error> {
        let key_type = self.get_type();
        if key_type == KeyType::Empty {
            return Ok(false);
        }
        if key_type != KeyType::Module {
            return Err(Error::WrongType);
        }
        let raw_type = unsafe { raw::RedisModule_ModuleTypeGetType.unwrap()(self.ptr) };

        if raw_type != *redis_type.raw_type.borrow() {
            return Err(Error::WrongType);
        }
        Ok(true)
    }

    // Get the string value of the eky
    pub fn string_get(&self) -> Result<RString, Error> {
        let value = unsafe {
            let mut len = 0;
            let data = raw::RedisModule_StringDMA.unwrap()(
                self.ptr,
                &mut len,
                raw::REDISMODULE_READ as c_int,
            ) as *mut u8;
            if data.is_null() {
                return Err(Error::new("fail to get string value"));
            }
            { std::str::from_utf8(std::slice::from_raw_parts(data, len as usize))? }
        };
        Ok(RString::from_str(value))
    }

    /// Get fields from an hash value.
    pub fn hash_get(&self, flag: HashGetFlag, field: &RStr) -> Result<Option<RString>, Error> {
        let value: *mut raw::RedisModuleString = std::ptr::null_mut();
        unsafe {
            handle_status(
                raw::RedisModule_HashGet.unwrap()(
                    self.ptr,
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
        Ok(Some(RString::from_ptr(value)))
    }
    /// Get range of zset (key, score) pairs order by score
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
                init(self.ptr, min, max, minex, maxex),
                "fail to execute zset_score_range",
            )?;
            while check_end(self.ptr) == 0 {
                let mut score = 0.0;
                let elem = get_elem(self.ptr, &mut score);
                result.push((RString::from_ptr(elem), score));
                next(self.ptr);
            }
            raw::RedisModule_ZsetRangeStop.unwrap()(self.ptr);
        }
        Ok(result)
    }
    /// Get range of zset (key, score) pairs order by lex
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
            let check_end = raw::RedisModule_ZsetRangeEndReached.unwrap();
            let get_elem = raw::RedisModule_ZsetRangeCurrentElement.unwrap();
            handle_status(
                init(self.ptr, min.get_ptr(), max.get_ptr()),
                "fail to execute zset_lex_range",
            )?;
            while check_end(self.ptr) == 0 {
                let mut score = 0.0;
                let elem = get_elem(self.ptr, &mut score);
                result.push((RString::from_ptr(elem), score));
                next(self.ptr);
            }
            raw::RedisModule_ZsetRangeStop.unwrap()(self.ptr)
        }
        Ok(result)
    }
    /// Return the length of the value associated with the key.
    ///
    /// For strings this is the length of the string. For all the other types
    /// is the number of elements (just counting keys for hashes).
    pub fn value_length(&self) -> usize {
        unsafe { raw::RedisModule_ValueLength.unwrap()(self.ptr) }
    }
    ///  Return the key expire value, as milliseconds of remaining TTL.
    ///
    /// If no TTL is associated with the key or if the key is empty, None is returned.
    pub fn get_expire(&self) -> Option<Duration> {
        let result: i64 = unsafe { raw::RedisModule_GetExpire.unwrap()(self.ptr) };
        if result == raw::REDISMODULE_NO_EXPIRE as i64 {
            None
        } else {
            Some(Duration::from_millis(result as u64))
        }
    }
    /// Return the type of the key.
    ///
    /// If the key pointer is NULL then `KeyType::EMPTY` is returned.
    pub fn get_type(&self) -> KeyType {
        let v = unsafe { raw::RedisModule_KeyType.unwrap()(self.ptr) as u32 };
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
    /// Scan api that allows a module to scan the elements in a hash, set or sorted set key
    pub fn scan<T>(
        &self,
        cursor: &ScanCursor,
        callback: raw::RedisModuleScanKeyCB,
        privdata: Option<T>,
    ) -> Result<(), Error> {
        let data = match privdata {
            Some(v) => Box::into_raw(Box::from(v)) as *mut c_void,
            None => 0 as *mut c_void,
        };
        handle_status(
            unsafe {
                raw::RedisModule_ScanKey.unwrap()(self.ptr, cursor.get_ptr(), callback, data)
            },
            "fail to scan",
        )
    }
    /// Returns the name of the key
    pub fn get_keyname(&self) -> RStr {
        let ptr = unsafe { raw::RedisModule_GetKeyNameFromModuleKey.unwrap()(self.ptr) };
        RStr::from_ptr({ ptr as *mut raw::RedisModuleString })
    }
    /// This function is used in order to potentially unblock a client blocked
    /// on keys with `Context::block_client_on_keys`. When this function is called,
    /// all the clients blocked for this key will get their reply callback called,
    /// and if the callback returns REDISMODULE_OK the client will be unblocked.
    pub fn signal_ready(&self) {
        unsafe {
            raw::RedisModule_SignalKeyAsReady.unwrap()(self.ctx, self.get_keyname().get_ptr())
        }
    }
    /// Gets the key access frequency or -1 if the server's eviction policy is not
    /// LFU based.
    pub fn get_lfu(&self) -> Result<u64, Error> {
        let mut freq = 0;
        handle_status(
            unsafe { raw::RedisModule_GetLFU.unwrap()(self.ptr, &mut freq) },
            "fail to get lfu",
        )?;
        Ok(freq as u64)
    }
    /// Gets the key last access time.
    ///
    /// Value is idletime in milliseconds or -1 if the server's eviction policy is
    /// LFU based.
    pub fn get_lru(&self) -> Result<Duration, Error> {
        let mut time_ms = 0;
        handle_status(
            unsafe { raw::RedisModule_GetLRU.unwrap()(self.ptr, &mut time_ms) },
            "fail to get lru",
        )?;
        Ok(Duration::from_millis(time_ms as u64))
    }
    fn get_context(&self) -> Context {
        Context::from_ptr(self.ctx)
    }
}

/// Repersent a Redis key with read and write permision
///
/// create with [`ctx.open_write_key`](./context/struct.Context.html#method.open_write_key)
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
    pub fn new(ctx: *mut raw::RedisModuleCtx, keyname: &RStr) -> Self {
        let mode = (raw::REDISMODULE_READ | raw::REDISMODULE_WRITE) as c_int;
        let ptr = unsafe {
            raw::RedisModule_OpenKey.unwrap()(ctx, keyname.get_ptr(), mode)
                as *mut raw::RedisModuleKey
        };
        WriteKey {
            read_key: ReadKey { ptr, ctx },
        }
    }
    /// Set the specified module type object as the value of the key, deleting the old value if any.
    pub fn set_value<T>(&self, redis_type: &RType<T>, value: T) -> Result<&mut T, Error> {
        let value = Box::into_raw(Box::new(value)) as *mut c_void;
        handle_status(
            unsafe {
                raw::RedisModule_ModuleTypeSetValue.unwrap()(
                    self.ptr,
                    *redis_type.raw_type.borrow(),
                    value,
                )
            },
            "fail to set value",
        )?;
        Ok(unsafe { &mut *(value as *mut T) })
    }
    ///  Replace the value assigned to a module type.
    ///
    ///  The key must be open for writing, have an existing value, and have a moduleType
    ///  that matches the one specified by the caller.
    ///
    ///  Unlike `WriteKey::set_value` which will free the old value, this function
    ///  simply swaps the old value with the new value.
    ///
    ///  The function returns Ok on success, Err on errors
    ///  such as:
    ///
    ///  1. Key is not opened for writing.
    ///  2. Key is not a module data type key.
    ///  3. Key is a module datatype other than 'mt'.
    ///
    ///  If old_value is non-NULL, the old value is returned by reference.
    pub fn replace_value<T>(
        &self,
        redis_type: &RType<T>,
        value: T,
    ) -> Result<(&mut T, Box<T>), Error> {
        let value = Box::into_raw(Box::new(value)) as *mut c_void;
        let mut old_value: *mut c_void = std::ptr::null_mut();
        handle_status(
            unsafe {
                raw::RedisModule_ModuleTypeReplaceValue.unwrap()(
                    self.ptr,
                    *redis_type.raw_type.borrow(),
                    value,
                    &mut old_value,
                )
            },
            "fail to replace value",
        )?;
        Ok(unsafe { (&mut *(value as *mut T), Box::from_raw(old_value as *mut T)) })
    }
    /// Remove the key, and setup the key to accept new writes as an empty
    /// key (that will be created on demand).
    pub fn delete(&self) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_DeleteKey.unwrap()(self.ptr) },
            "fail to execute delete",
        )
    }
    /// Unlink the key (that is delete it in a non-blocking way, not reclaiming
    /// memory immediately) and setup the key to  accept new writes as
    /// an empty key (that will be created on demand).
    pub fn unlink(&self) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_UnlinkKey.unwrap()(self.ptr) },
            "fail to execute unlink",
        )
    }
    /// Set new expire for the key.
    /// If the special expire REDISMODULE_NO_EXPIRE is set, the expire is
    /// cancelled if there was one (the same as the PERSIST command).
    /// Note that the expire must be provided as a positive integer representing
    /// the number of milliseconds of TTL the key should have.
    pub fn set_expire(&self, expire_ms: Duration) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_SetExpire.unwrap()(self.ptr, expire_ms.as_millis() as i64) },
            "fail to execute set_expire",
        )
    }
    /// Set the specified string 'str' as the value of the key, deleting the old value if any.
    pub fn string_set(&self, value: &RStr) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_StringSet.unwrap()(self.ptr, value.get_ptr()) },
            "fail to execute string_set",
        )
    }
    /// Push an element into a list
    pub fn list_push(&self, position: ListPosition, value: &RStr) -> Result<(), Error> {
        handle_status(
            unsafe {
                raw::RedisModule_ListPush.unwrap()(self.ptr, position as i32, value.get_ptr())
            },
            "fail to execute list_push",
        )
    }
    /// Pop an element from the list, and returns it.
    pub fn list_pop(&self, pos: ListPosition) -> Result<RString, Error> {
        let p = unsafe { raw::RedisModule_ListPop.unwrap()(self.ptr, pos as i32) };
        if p.is_null() {
            return Err(Error::new("fail to pop list"));
        }
        Ok(RString::from_ptr(p))
    }
    /// Set the field of the specified hash field to the specified value.
    ///
    /// If value is none, it will clear the field.
    pub fn hash_set(
        &self,
        flag: HashSetFlag,
        field: &RStr,
        value: Option<&RStr>,
    ) -> Result<(), Error> {
        let value_ = match value {
            Some(v) => v.get_ptr(),
            None => 0 as *mut raw::RedisModuleString,
        };
        unsafe {
            handle_status(
                raw::RedisModule_HashSet.unwrap()(
                    self.ptr,
                    flag.into(),
                    field.get_ptr(),
                    value_,
                    0,
                ),
                "fail to execute hash_set",
            )?;
        }
        Ok(())
    }
    /// Add a new element into a sorted set, with the specified 'score'.
    /// If the element already exists, the score is updated.
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
                raw::RedisModule_ZsetAdd.unwrap()(self.ptr, score, ele.get_ptr(), &mut flag),
                "fail to execute zset_add",
            )?;
            out_flag = flag.into();
        }
        Ok(out_flag)
    }
    /// This function works exactly like `WriteKey::zset_add`, but instead of setting
    /// a new score, the score of the existing element is incremented, or if the
    /// element does not already exist, it is added assuming the old score was
    /// zero.
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
                    self.ptr,
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
    /// Remove the specified element from the sorted set.
    ///
    /// The bool indicate Whether the element was removed
    pub fn zset_rem(&self, ele: &RStr) -> Result<bool, Error> {
        let mut flag = 0;
        unsafe {
            handle_status(
                raw::RedisModule_ZsetRem.unwrap()(self.ptr, ele.get_ptr(), &mut flag),
                "fail to execute zset_rem",
            )?;
        }
        let result = if flag == 0 { false } else { true };
        Ok(result)
    }
    /// On success retrieve the double score associated at the sorted set element 'ele'.
    pub fn zset_score(&self, ele: &RStr) -> Result<f64, Error> {
        unsafe {
            let mut score = 0.0;
            handle_status(
                raw::RedisModule_ZsetScore.unwrap()(self.ptr, ele.get_ptr(), &mut score),
                "fail to execute zset_score",
            )?;
            Ok(score)
        }
    }
    /// Set the key access frequency. only relevant if the server's maxmemory policy
    /// is LFU based.
    pub fn set_lfu(&self, freq: u64) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_SetLFU.unwrap()(self.ptr, freq as i64) },
            "fail to set lfu",
        )
    }
    /// Set the key last access time for LRU based eviction. not relevent if the
    /// servers's maxmemory policy is LFU based. Value is idle time in milliseconds.
    pub fn set_lru(&self, time_ms: Duration) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_SetLRU.unwrap()(self.ptr, time_ms.as_millis() as i64) },
            "fail to set lru",
        )
    }
}

/// The position of WriteKey::ListPop / WriteKey::ListPush operation
pub enum ListPosition {
    Head = raw::REDISMODULE_LIST_HEAD as isize,
    Tail = raw::REDISMODULE_LIST_TAIL as isize,
}

/// The type of key
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

/// Control the behaiver of WriteKey::hash_set
#[derive(Debug, PartialEq)]
pub enum HashSetFlag {
    /// Set the value
    None,
    /// The operation is performed only if the field was not already existing in the hash.
    NX,
    /// The operation is performed only if the field was already existing,
    /// so that a new value could be associated to an existing filed,
    /// but no new fields are created.
    XX,
}

impl Into<c_int> for HashSetFlag {
    fn into(self) -> c_int {
        match self {
            HashSetFlag::None => raw::REDISMODULE_HASH_NONE as c_int,
            HashSetFlag::NX => raw::REDISMODULE_HASH_NX as c_int,
            HashSetFlag::XX => raw::REDISMODULE_HASH_XX as c_int,
        }
    }
}

/// Control the behavior of `ReadKey::hash_get`
#[derive(Debug, PartialEq)]
pub enum HashGetFlag {
    None,
    Exists,
}

impl Into<c_int> for HashGetFlag {
    fn into(self) -> c_int {
        match self {
            HashGetFlag::None => raw::REDISMODULE_HASH_NONE as c_int,
            HashGetFlag::Exists => raw::REDISMODULE_HASH_EXISTS as c_int,
        }
    }
}

/// Control the order of zset_range
#[derive(Debug, PartialEq)]
pub enum ZsetRangeDirection {
    FristIn,
    LastIn,
}

/// Control the behavier of zadd
#[derive(Debug, PartialEq)]
pub enum ZaddInputFlag {
    /// Element must already exist. Do nothing otherwise.
    XX = raw::REDISMODULE_ZADD_XX as isize,
    /// Element must not exist. Do nothing otherwise.
    NX = raw::REDISMODULE_ZADD_NX as isize,
}

/// Describe the state of zadd operation
#[derive(Debug, PartialEq)]
pub enum ZaddOuputFlag {
    /// The new element was added to the sorted set.
    Added,
    /// The score of the element was updated.
    Updated,
    /// No operation was performed because XX or NX flags.
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
