use crate::raw;
use crate::{Ptr, Error, handle_status};
use std::ffi::CString;

#[repr(C)]
pub struct User {
    inner: *mut raw::RedisModuleUser,
}

impl User {
    pub fn from_ptr(inner: *mut raw::RedisModuleUser) -> Self {
        User { inner }
    }
    pub fn new<T: AsRef<str>>(name: T) -> Self {
        let name = CString::new(name.as_ref()).unwrap();
        let inner = unsafe {
            raw::RedisModule_CreateModuleUser.unwrap()(name.as_ptr())
        };
        Self::from_ptr(inner)
    }
    pub fn set_acl<T: AsRef<str>>(&mut self, acl: T) -> Result<(), Error> {
        let acl = CString::new(acl.as_ref()).unwrap();
        handle_status(
            unsafe {
                raw::RedisModule_SetModuleUserACL.unwrap()(self.inner, acl.as_ptr())
            },
            "fail to set acl"
        )
    }
    pub fn free(&mut self) {
        unsafe { raw::RedisModule_FreeModuleUser.unwrap()(self.inner) }
    }
}

impl Ptr for User {
    type PtrType = raw::RedisModuleUser;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.inner
    }
}

unsafe impl Send for User {}
unsafe impl Sync for User {}