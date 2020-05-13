//! Acl user

use crate::raw;
use crate::{Ptr, handle_status};
use std::ffi::CString;
use crate::error::Error;

/// Redis ACL user that the module can use to authenticate a client
#[repr(C)]
pub struct User {
    ptr: *mut raw::RedisModuleUser,
}

impl User {
    pub fn from_ptr(ptr: *mut raw::RedisModuleUser) -> Self {
        User { ptr }
    }
    pub fn new<T: AsRef<str>>(name: T) -> Self {
        let name = CString::new(name.as_ref()).unwrap();
        let ptr = unsafe {
            raw::RedisModule_CreateModuleUser.unwrap()(name.as_ptr())
        };
        Self::from_ptr(ptr)
    }
    /// Sets the permissions on ACL user. 
    pub fn set_acl<T: AsRef<str>>(&mut self, acl: T) -> Result<(), Error> {
        let acl = CString::new(acl.as_ref()).unwrap();
        handle_status(
            unsafe {
                raw::RedisModule_SetModuleUserACL.unwrap()(self.ptr, acl.as_ptr())
            },
            "fail to set acl"
        )
    }
}

impl Ptr for User {
    type PtrType = raw::RedisModuleUser;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.ptr
    }
}

impl Drop for User {
    fn drop(&mut self) {
        unsafe { raw::RedisModule_FreeModuleUser.unwrap()(self.ptr) }
    }
}

unsafe impl Send for User {}
unsafe impl Sync for User {}