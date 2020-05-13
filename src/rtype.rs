//! Redis data type supports
//!
//! # Examples
//!
//! ```rust,no_run
//!
//! pub struct MyType {
//!     pub data: i64,
//! }
//!
//! #[rtypedef("mytype-00", 0)]
//! impl TypeMethod for MyType {
//!     fn rdb_load(io: &mut IO, encver: u32) -> Option<Box<Self>> {
//!         if encver != 0 {
//!             return None;
//!         }
//!         let data = io.load_signed();
//!         Some(Box::new(MyType { data }))
//!     }
//!     fn rdb_save(&self, io: &mut IO) {
//!         io.save_signed(self.data);
//!     }
//!     fn free(_: Box<Self>) {}
//!     fn aof_rewrite(&self, io: &mut IO, key: &RStr) {
//!         let keyname = key.to_str().unwrap();
//!         io.emit_aof(
//!             "HELLOTYPE.INSERT",
//!             ArgvFlags::new(),
//!             &[keyname, self.data.to_string().as_str() ],
//!         )
//!     }
//! }
//!
//! define_module! {
//!     ...
//!     data_types: [
//!         MYTYPE,
//!     ],
//!     ...
//! }
//!
//! ```

use std::cell::RefCell;
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr;

use crate::context::Context;
use crate::error::Error;
use crate::io::{Digest, IO};
use crate::raw;
use crate::string::{RStr, RString};
use crate::{LogLevel, Ptr};

/// A help trait for registing new data type
pub trait TypeMethod {
    /// Bit flags control when triggers the aux_load and aux_save callbacks
    const AUX_SAVE_TRIGGERS: AuxSaveTriggerFlag = AuxSaveTriggerFlag::AuxBeforeRdb;
    /// A callback function pointer that loads data from RDB files
    #[allow(unused_variables)]
    fn rdb_load(io: &mut IO, encver: u32) -> Option<Box<Self>> {
        unimplemented!()
    }
    /// A callback function pointer that saves data to RDB files.
    #[allow(unused_variables)]
    fn rdb_save(&self, io: &mut IO) {
        unimplemented!()
    }
    /// A callback function pointer that rewrites data as commands.
    #[allow(unused_variables)]
    fn aof_rewrite(&self, io: &mut IO, key: &RStr) {
        unimplemented!()
    }
    /// A callback function pointer that report memory usage
    ///
    /// It should currently be omitted since it is not yet implemented inside the Redis modules core.
    fn mem_usage(&self) -> usize {
        unimplemented!()
    }
    /// A callback function pointer that is used for `DEBUG DIGEST`.
    ///
    /// It should currently be omitted since it is not yet implemented inside the Redis modules core.
    #[allow(unused_variables)]
    fn digest(&self, digest: &mut Digest) {
        unimplemented!()
    }
    /// A callback function pointer that can free a type value.
    #[allow(unused_variables)]
    fn free(value: Box<Self>) {
        unimplemented!()
    }
    /// A callback function pointer that loads out of keyspace data from RDB files.
    #[allow(unused_variables)]
    fn aux_save(rdb: &mut IO, when: i32) {
        unimplemented!()
    }
    /// A callback function pointer that saves out of keyspace data to RDB files.
    #[allow(unused_variables)]
    fn aux_load(rdb: &mut IO, encver: u32, when: i32) {
        unimplemented!()
    }
}

/// Biflags for [`aux_save_triggers`](../raw/struct.RedisModuleTypeMethods.html#structfield.aux_save_triggers)
pub enum AuxSaveTriggerFlag {
    AuxBeforeRdb = raw::REDISMODULE_AUX_BEFORE_RDB as isize,
    AuxAfterRdb = raw::REDISMODULE_AUX_AFTER_RDB as isize,
}

impl Into<i32> for AuxSaveTriggerFlag {
    fn into(self) -> i32 {
        self as i32
    }
}

/// A redis data type
///
/// Recommand creating rtype with `rdeftype` macro and `TypeMethods` trait.
pub struct RType<T> {
    /// A 9 characters data type name that MUST be unique in the Redis
    /// Modules ecosystem. Be creative... and there will be no collisions. Use
    /// the charset A-Z a-z 9-0, plus the two "-_" characters. A good
    /// idea is to use, for example `<typename>-<vendor>`. For example
    /// "tree-AntZ" may mean "Tree data structure by @antirez". To use both
    /// lower case and upper case letters helps in order to prevent collisions.
    name: &'static str,
    /// Encoding version, which is, the version of the serialization
    /// that a module used in order to persist data. As long as the "name"
    /// matches, the RDB loading will be dispatched to the type callbacks
    /// whatever 'encver' is used, however the module can understand if
    /// the encoding it must load are of an older version of the module.
    /// For example the module "tree-AntZ" initially used encver=0. Later
    /// after an upgrade, it started to serialize data in a different format
    /// and to register the type with encver=1. However this module may
    /// still load old data produced by an older version if the rdb_load
    /// callback is able to check the encver value and act accordingly.
    /// The encver must be a positive value between 0 and 1023.
    version: i32,
    /// A pointer to a RedisModuleTypeMethods structure that should be
    /// populated with the methods callbacks and structure version.
    type_methods: raw::RedisModuleTypeMethods,
    marker: std::marker::PhantomData<T>,
    pub raw_type: RefCell<*mut raw::RedisModuleType>,
}

// We want to be able to create static instances of this type,
// which means we need to implement Sync.
unsafe impl<T> Sync for RType<T> {}

impl<T> RType<T> {
    pub const fn new(
        name: &'static str,
        version: i32,
        type_methods: raw::RedisModuleTypeMethods,
    ) -> Self {
        RType {
            name,
            version,
            type_methods,
            marker: PhantomData,
            raw_type: RefCell::new(ptr::null_mut()),
        }
    }

    /// Decode a serialized representation of a module data type 'mt' from string
    /// 'str' and return a newly allocated value, or NULL if decoding failed.
    ///
    /// This call basically reuses the 'rdb_load' callback which module data types
    /// implement in order to allow a module to arbitrarily serialize/de-serialize
    /// keys, similar to how the Redis 'DUMP' and 'RESTORE' commands are implemented.
    pub fn load(&self, value: RStr) -> Box<T> {
        unsafe {
            let ptr = raw::RedisModule_LoadDataTypeFromString.unwrap()(
                value.get_ptr() as *const raw::RedisModuleString,
                *self.raw_type.borrow_mut(),
            );
            Box::from_raw(ptr as *mut T)
        }
    }

    /// Encode a module data type 'mt' value 'data' into serialized form, and return it
    /// as a newly allocated RedisModuleString.
    ///
    /// This call basically reuses the 'rdb_save' callback which module data types
    /// implement in order to allow a module to arbitrarily serialize/de-serialize
    /// keys, similar to how the Redis 'DUMP' and 'RESTORE' commands are implemented.
    pub fn save(&self, ctx: &Context, value: T) -> Option<(Box<T>, RString)> {
        let value = Box::into_raw(Box::new(value));
        unsafe {
            let ptr = raw::RedisModule_SaveDataTypeToString.unwrap()(
                ctx.get_ptr(),
                value as *mut c_void,
                *self.raw_type.borrow_mut(),
            );
            if ptr.is_null() {
                return None;
            }
            Some((Box::from_raw(value), RString::new(ctx.get_ptr(), ptr)))
        }
    }

    pub fn create(&self, ctx: &mut Context) -> Result<(), Error> {
        if self.name.len() != 9 {
            let msg = "Redis requires the length of native type names to be exactly 9 characters";
            ctx.log(
                LogLevel::Warning,
                &format!("{}, name is: '{}'", msg, self.name),
            );
            return Err(Error::new(msg));
        }

        let type_name = CString::new(self.name).unwrap();

        let redis_type = unsafe {
            raw::RedisModule_CreateDataType.unwrap()(
                ctx.get_ptr(),
                type_name.as_ptr(),
                self.version, // Encoding version
                &mut self.type_methods.clone(),
            )
        };

        if redis_type.is_null() {
            let msg = "Created data type is null";
            ctx.log(LogLevel::Warning, msg);
            return Err(Error::new(msg));
        }

        *self.raw_type.borrow_mut() = redis_type;

        ctx.log(
            LogLevel::Notice,
            &format!("Created new data type '{}'", self.name),
        );

        Ok(())
    }
}
