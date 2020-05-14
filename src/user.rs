//! Acl user

use crate::error::Error;
use crate::raw;
use crate::{handle_status, FromPtr, GetPtr};
use std::ffi::CString;

/// Redis ACL user that the module can use to authenticate a client
#[repr(C)]
pub struct User {
    ptr: *mut raw::RedisModuleUser,
}

impl User {
    /// Creates a Redis ACL user that the module can use to authenticate a client.
    ///
    /// After obtaining the user, the module should set what such user can do
    /// using the `User::set_acl` function. Once configured, the user
    /// can be used in order to authenticate a connection, with the specified ACL rules.
    ///
    /// Note that:
    ///
    /// * Users created here are not listed by the ACL command.
    /// * Users created here are not checked for duplicated name, so it's up to
    ///   the module calling this function to take care of not creating users
    ///   with the same name.
    /// * The created user can be used to authenticate multiple Redis connections.
    ///
    /// If User is dropped, if there are still clients authenticated with this user,
    /// they are disconnected. The function to free the user should only be used
    /// when the caller really wants to invalidate the user to define a new one
    /// with different capabilities.
    pub fn new<T: AsRef<str>>(name: T) -> Self {
        let name = CString::new(name.as_ref()).unwrap();
        let ptr = unsafe { raw::RedisModule_CreateModuleUser.unwrap()(name.as_ptr()) };
        Self::from_ptr(ptr)
    }
    /// Sets the permissions on ACL user.
    pub fn set_acl<T: AsRef<str>>(&mut self, acl: T) -> Result<(), Error> {
        let acl = CString::new(acl.as_ref()).unwrap();
        handle_status(
            unsafe { raw::RedisModule_SetModuleUserACL.unwrap()(self.ptr, acl.as_ptr()) },
            "fail to set acl",
        )
    }
}

impl GetPtr for User {
    type PtrType = raw::RedisModuleUser;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.ptr
    }
}

impl FromPtr for User {
    type PtrType = raw::RedisModuleUser;
    fn from_ptr(ptr: *mut raw::RedisModuleUser) -> User {
        User { ptr }
    }
}

impl Drop for User {
    fn drop(&mut self) {
        unsafe { raw::RedisModule_FreeModuleUser.unwrap()(self.ptr) }
    }
}

unsafe impl Send for User {}
unsafe impl Sync for User {}
