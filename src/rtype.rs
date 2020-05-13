use std::cell::RefCell;
use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr;
use std::os::raw::c_void;

use crate::raw;
use crate::{Context, Digest, Error, LogLevel, Ptr, RStr, IO, RString};

pub trait TypeMethod {
    #[allow(unused_variables)]
    fn rdb_load(io: &mut IO, encver: u32) -> Option<Box<Self>> {
        unimplemented!()
    }
    #[allow(unused_variables)]
    fn rdb_save(&self, io: &mut IO) {
        unimplemented!()
    }
    #[allow(unused_variables)]
    fn aof_rewrite(&self, io: &mut IO, key: &RStr) {
        unimplemented!()
    }
    fn mem_usage(&self) -> usize {
        unimplemented!()
    }
    #[allow(unused_variables)]
    fn digest(&self, digest: &mut Digest) {
        unimplemented!()
    }
    #[allow(unused_variables)]
    fn free(value: Box<Self>) {
        unimplemented!()
    }
    #[allow(unused_variables)]
    fn aux_load(rdb: &mut IO, encver: u32, when: i32) {
        unimplemented!()
    }
    #[allow(unused_variables)]
    fn aux_save(rdb: &mut IO, when: i32) {
        unimplemented!()
    }
    fn aux_save_triggers(&self) -> i32 {
        unimplemented!()
    }
}

pub struct RType<T> {
    name: &'static str,
    version: i32,
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

    pub fn load(&self, value: RStr) -> Box<T> {
        unsafe {
            let ptr = raw::RedisModule_LoadDataTypeFromString.unwrap()(
                value.get_ptr() as *const raw::RedisModuleString, 
                *self.raw_type.borrow_mut(),
            );
            Box::from_raw(ptr as *mut T)
        }
    }

    pub fn save(&self, ctx: &Context, value: T) -> Option<(Box<T>, RString)> {
        let value = Box::into_raw(Box::new(value));
        unsafe {
           let ptr = raw::RedisModule_SaveDataTypeToString.unwrap()(
                ctx.get_ptr(),
                value as *mut c_void,
                *self.raw_type.borrow_mut()
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
