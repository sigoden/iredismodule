use std::cell::RefCell;
use std::ffi::CString;
use std::ptr;

use crate::raw;
use crate::{Context, Error, LogLevel, Ptr, IO, RStr, Digest};

pub enum TypeFeature {
    Rdb,
    Aof,
    MemUsage,
    Digest,
    Aux,
}

pub trait TypeDef {
    fn name(&self) -> String;
    fn version(&self) -> i32; 
    fn type_method_version(&self) -> u64 {
        raw::REDISMODULE_TYPE_METHOD_VERSION as u64
    }
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
    fn free(value: Box<Self>) { }
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

pub struct RType {
    name: &'static str,
    version: i32,
    type_methods: raw::RedisModuleTypeMethods,
    pub raw_type: RefCell<*mut raw::RedisModuleType>,
}

// We want to be able to create static instances of this type,
// which means we need to implement Sync.
unsafe impl Sync for RType {}

impl RType {
    pub const fn new(
        name: &'static str,
        version: i32,
        type_methods: raw::RedisModuleTypeMethods,
    ) -> Self {
        RType {
            name,
            version,
            type_methods,
            raw_type: RefCell::new(ptr::null_mut()),
        }
    }

    pub fn create_data_type(&self, ctx: &Context) -> Result<(), Error> {
        if self.name.len() != 9 {
            let msg = "Redis requires the length of native type names to be exactly 9 characters";
            ctx.log(
                LogLevel::Warning,
                &format!("{}, name is: '{}'", msg, self.name),
            );
            return Err(Error::generic(msg));
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
            return Err(Error::generic(msg));
        }

        *self.raw_type.borrow_mut() = redis_type;

        ctx.log(
            LogLevel::Notice,
            &format!("Created new data type '{}'", self.name),
        );

        Ok(())
    }
}
