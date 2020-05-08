use std::cell::RefCell;
use std::ffi::CString;
use std::ptr;

use crate::raw;
use crate::{Ctx, LogLevel, Error};

pub struct RedisType {
    name: &'static str,
    version: i32,
    type_methods: raw::RedisModuleTypeMethods,
    pub raw_type: RefCell<*mut raw::RedisModuleType>,
}

// We want to be able to create static instances of this type,
// which means we need to implement Sync.
unsafe impl Sync for RedisType {}

impl RedisType {
    pub const fn new(
        name: &'static str,
        version: i32,
        type_methods: raw::RedisModuleTypeMethods,
    ) -> Self {
        RedisType {
            name,
            version,
            type_methods,
            raw_type: RefCell::new(ptr::null_mut()),
        }
    }

    pub fn create_data_type(&self, ctx: &Ctx) -> Result<(), Error> {
        if self.name.len() != 9 {
            let msg = "Redis requires the length of native type names to be exactly 9 characters";
            ctx.log(LogLevel::Warning, &format!("{}, name is: '{}'", msg, self.name));
            return Err(Error::generic(msg));
        }

        let type_name = CString::new(self.name).unwrap();

        let redis_type = unsafe {
            raw::RedisModule_CreateDataType.unwrap()(
                ctx.inner,
                type_name.as_ptr(),
                self.version, // Encoding version
                &mut self.type_methods.clone(),
            )
        };

        if redis_type.is_null() {
            let msg = "Created data type is null";
            ctx.log(LogLevel::Warning, msg);
            return Err(Error::generic(msg));
        }

        *self.raw_type.borrow_mut() = redis_type;

        ctx.log(LogLevel::Notice, &format!("Created new data type '{}'", self.name));

        Ok(())
    }
}
