//! Module context

use crate::call_reply::CallReply;
use crate::error::Error;
use crate::key::{ReadKey, WriteKey};
use crate::raw;
use crate::scan_cursor::ScanCursor;
use crate::string::{RStr, RString};
use crate::user::User;
use crate::value::Value;
use crate::{handle_status, FromPtr, GetPtr, LogLevel, RResult, CallFlags, ServerEvent};

use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_long, c_void};

mod block_client;
mod cluster;
mod mutex;
mod timer;
pub use mutex::MutexContext;

/// Wrap raw pointer `raw::RedisModuleCtx`
#[repr(C)]
pub struct Context {
    ptr: *mut raw::RedisModuleCtx,
}

impl GetPtr for Context {
    type PtrType = raw::RedisModuleCtx;
    fn get_ptr(&self) -> *mut Self::PtrType {
        self.ptr
    }
}

impl FromPtr for Context {
    type PtrType = raw::RedisModuleCtx;
    fn from_ptr(ptr: *mut raw::RedisModuleCtx) -> Context {
        Context { ptr }
    }
}

impl Context {
    /// Return true if a module command, that was declared with the
    /// flag "getkeys-api", is called in a special way to get the keys positions
    /// and not to get executed. Otherwise false is returned
    pub fn is_keys_position_request(&self) -> bool {
        let result = unsafe { raw::RedisModule_IsKeysPositionRequest.unwrap()(self.ptr) };
        result != 0
    }

    /// When a module command is called in order to obtain the position of
    /// keys, since it was flagged as "getkeys-api" during the registration,
    /// the command implementation checks for this special call using the
    /// `Context::is_keys_position_request` API and uses this function in
    /// order to report keys, like in the following example:
    ///
    /// ```rust,no_run
    ///     if (ctx.is_keys_position_request()) {
    ///         ctx.key_at_pos(1);
    ///         ctx.key_at_pos(2);
    ///     }
    /// ```
    ///
    ///  Note: in the example below the get keys API would not be needed since
    ///  keys are at fixed positions. This interface is only used for commands
    ///  with a more complex structure.
    pub fn key_at_pos(&self, pos: i32) {
        unsafe {
            raw::RedisModule_KeyAtPos.unwrap()(self.ptr, pos);
        }
    }
    /// Send reply to client
    ///
    /// It will choose the correct redis ffi function to reply depends on the `result::RResult` value.
    pub fn reply(&self, r: RResult) {
        match r {
            Ok(Value::Integer(v)) => unsafe {
                raw::RedisModule_ReplyWithLongLong.unwrap()(self.ptr, v);
            },
            Ok(Value::Float(v)) => unsafe {
                raw::RedisModule_ReplyWithDouble.unwrap()(self.ptr, v);
            },
            Ok(Value::String(v)) => unsafe {
                let msg = CString::new(v).unwrap();
                raw::RedisModule_ReplyWithSimpleString.unwrap()(self.ptr, msg.as_ptr());
            },
            Ok(Value::Buffer(v)) => unsafe {
                raw::RedisModule_ReplyWithStringBuffer.unwrap()(
                    self.ptr,
                    v.as_ptr() as *const c_char,
                    v.len(),
                );
            },
            Ok(Value::Array(v)) => {
                unsafe {
                    raw::RedisModule_ReplyWithArray.unwrap()(self.ptr, v.len() as c_long);
                }
                v.into_iter().for_each(|elem| self.reply(Ok(elem)));
            }
            Ok(Value::Null) => unsafe {
                raw::RedisModule_ReplyWithNull.unwrap()(self.ptr);
            },
            Ok(Value::NoReply) => {}
            Err(Error::WrongArity) => unsafe {
                raw::RedisModule_WrongArity.unwrap()(self.ptr);
            },
            Err(err) => unsafe {
                let msg = CString::new(err.to_string()).unwrap();
                raw::RedisModule_ReplyWithError.unwrap()(self.ptr, msg.as_ptr());
            },
        }
    }
    /// Exported API to call any Redis command from modules.
    pub fn call<T: AsRef<str>>(
        &self,
        command: T,
        flags: CallFlags,
        args: &[&RStr],
    ) -> Result<CallReply, Error> {
        let args: Vec<*mut raw::RedisModuleString> = args.iter().map(|s| s.get_ptr()).collect();

        let cmd = CString::new(command.as_ref()).unwrap();
        let flags: CString = flags.into();

        let reply: *mut raw::RedisModuleCallReply = unsafe {
            raw::RedisModule_Call.unwrap()(
                self.ptr,
                cmd.as_ptr(),
                flags.as_ptr(),
                args.as_ptr() as *mut c_char,
                args.len(),
            )
        };
        if reply.is_null() {
            Err(Error::new("fail to call command"))
        } else {
            Ok(CallReply::from_ptr(reply))
        }
    }
    /// Same as `Context::call`, but args is str-like
    pub fn call_str<T: AsRef<str>>(
        &self,
        command: T,
        flags: CallFlags,
        args: &[T],
    ) -> Result<CallReply, Error> {
        let str_args: Vec<RString> = args.iter().map(|v| RString::from_str(v.as_ref())).collect();
        let str_args: Vec<&RStr> = str_args.iter().map(|v| v.to_rstr()).collect();
        self.call(command, flags, &str_args)
    }

    /// Replicate the specified command and arguments to slaves and AOF, as effect
    /// of execution of the calling command implementation.
    ///
    /// The replicated commands are always wrapped into the MULTI/EXEC that
    /// contains all the commands replicated in a given module command
    /// execution. However the commands replicated with `Context::call`
    /// are the first items, the ones replicated with `Context::replicate`
    /// will all follow before the EXEC.
    ///
    /// Modules should try to use one interface or the other.
    ///
    /// This command follows exactly the same interface of `Context::call`,
    /// so a set of format specifiers must be passed, followed by arguments
    /// matching the provided format specifiers.
    ///
    /// Please refer to `Context::call` for more information.
    ///
    /// Using the special "A" and "R" modifiers, the caller can exclude either
    /// the AOF or the replicas from the propagation of the specified command.
    /// Otherwise, by default, the command will be propagated in both channels.
    ///
    /// ## Note about calling this function from a thread safe context:
    ///
    /// Normally when you call this function from the callback implementing a
    /// module command, or any other callback provided by the Redis Module API,
    /// Redis will accumulate all the calls to this function in the context of
    /// the callback, and will propagate all the commands wrapped in a MULTI/EXEC
    /// transaction. However when calling this function from a threaded safe context
    /// that can live an undefined amount of time, and can be locked/unlocked in
    /// at will, the behavior is different: MULTI/EXEC wrapper is not emitted
    /// and the command specified is inserted in the AOF and replication stream
    /// immediately.
    ///
    /// ## Return value
    ///
    /// The command returns Err if the format specifiers are invalid
    /// or the command name does not belong to a known command.
    pub fn replicate<T: AsRef<str>>(
        &self,
        command: T,
        flags: CallFlags,
        args: &[&RStr],
    ) -> Result<(), Error> {
        let args: Vec<*mut raw::RedisModuleString> = args.iter().map(|s| s.get_ptr()).collect();

        let cmd = CString::new(command.as_ref()).unwrap();
        let flags: CString = flags.into();

        let result = unsafe {
            let p_call = raw::RedisModule_Replicate.unwrap();
            p_call(
                self.ptr,
                cmd.as_ptr(),
                flags.as_ptr(),
                args.as_ptr() as *mut c_char,
                args.len(),
            )
        };
        handle_status(result, "fail to replicate")
    }
    /// Same as `Context::replicate`, but args is str-like
    pub fn replicate_str<T: AsRef<str>>(
        &self,
        command: T,
        flags: CallFlags,
        args: &[T],
    ) -> Result<(), Error> {
        let str_args: Vec<RString> = args.iter().map(|v| RString::from_str(v.as_ref())).collect();
        let str_args: Vec<&RStr> = str_args.iter().map(|v| v.to_rstr()).collect();
        self.replicate(command, flags, &str_args)
    }
    /// This function will replicate the command exactly as it was invoked
    /// by the client. Note that this function will not wrap the command into
    /// a MULTI/EXEC stanza, so it should not be mixed with other replication
    /// commands.
    ///
    /// Basically this form of replication is useful when you want to propagate
    /// the command to the slaves and AOF file exactly as it was called, since
    /// the command can just be re-executed to deterministically re-create the
    /// new state starting from the old one.
    pub fn replicate_verbatim(&self) {
        unsafe {
            raw::RedisModule_ReplicateVerbatim.unwrap()(self.ptr);
        }
    }
    /// Return the ID of the current client calling the currently active module
    /// command. The returned ID has a few guarantees:
    ///`
    /// 1. The ID is different for each different client, so if the same client
    ///    executes a module command multiple times, it can be recognized as
    ///    having the same ID, otherwise the ID will be different.
    /// 2. The ID increases monotonically. Clients connecting to the server later
    ///    are guaranteed to get IDs greater than any past ID previously seen.
    ///
    /// Valid IDs are from 1 to 2^64-1. If 0 is returned it means there is no way
    /// to fetch the ID in the context the function was currently called.
    pub fn get_client_id(&self) -> u64 {
        unsafe { raw::RedisModule_GetClientId.unwrap()(self.ptr) as u64 }
    }
    /// Return the currently selected DB
    pub fn get_select_db(&self) -> i64 {
        unsafe { raw::RedisModule_GetSelectedDb.unwrap()(self.ptr) as i64 }
    }
    /// Return the current context's flags. The flags provide information on the
    /// current request context (whether the client is a Lua script or in a MULTI),
    /// and about the Redis instance in general, i.e replication and persistence.
    ///
    /// It is possible to call this function even with a NULL context, however
    /// in this case the following flags will not be reported:
    ///
    ///  * LUA, MULTI, REPLICATED, DIRTY (see below for more info).
    ///
    /// Available flags and their meaning:
    ///
    ///  * REDISMODULE_CTX_FLAGS_LUA: The command is running in a Lua script
    ///
    ///  * REDISMODULE_CTX_FLAGS_MULTI: The command is running inside a transaction
    ///
    ///  * REDISMODULE_CTX_FLAGS_REPLICATED: The command was sent over the replication
    ///    link by the MASTER
    ///
    ///  * REDISMODULE_CTX_FLAGS_MASTER: The Redis instance is a master
    ///
    ///  * REDISMODULE_CTX_FLAGS_SLAVE: The Redis instance is a slave
    ///
    ///  * REDISMODULE_CTX_FLAGS_READONLY: The Redis instance is read-only
    ///
    ///  * REDISMODULE_CTX_FLAGS_CLUSTER: The Redis instance is in cluster mode
    ///
    ///  * REDISMODULE_CTX_FLAGS_AOF: The Redis instance has AOF enabled
    ///
    ///  * REDISMODULE_CTX_FLAGS_RDB: The instance has RDB enabled
    ///
    ///  * REDISMODULE_CTX_FLAGS_MAXMEMORY:  The instance has Maxmemory set
    ///
    ///  * REDISMODULE_CTX_FLAGS_EVICT:  Maxmemory is set and has an eviction
    ///    policy that may delete keys
    ///
    ///  * REDISMODULE_CTX_FLAGS_OOM: Redis is out of memory according to the
    ///    maxmemory setting.
    ///
    ///  * REDISMODULE_CTX_FLAGS_OOM_WARNING: Less than 25% of memory remains before
    ///                                       reaching the maxmemory level.
    ///
    ///  * REDISMODULE_CTX_FLAGS_LOADING: Server is loading RDB/AOF
    ///
    ///  * REDISMODULE_CTX_FLAGS_REPLICA_IS_STALE: No active link with the master.
    ///
    ///  * REDISMODULE_CTX_FLAGS_REPLICA_IS_CONNECTING: The replica is trying to
    ///                                                 connect with the master.
    ///
    ///  * REDISMODULE_CTX_FLAGS_REPLICA_IS_TRANSFERRING: Master -> Replica RDB
    ///                                                   transfer is in progress.
    ///
    ///  * REDISMODULE_CTX_FLAGS_REPLICA_IS_ONLINE: The replica has an active link
    ///                                             with its master. This is the
    ///                                             contrary of STALE state.
    ///
    ///  * REDISMODULE_CTX_FLAGS_ACTIVE_CHILD: There is currently some background
    ///                                        process active (RDB, AUX or module).
    ////
    pub fn get_context_flags(&self) -> u64 {
        unsafe { raw::RedisModule_GetContextFlags.unwrap()(self.ptr) as u64 }
    }
    /// Change the currently selected DB. Returns an error if the id
    /// is out of range.
    ///
    /// Note that the client will retain the currently selected DB even after
    /// the Redis command implemented by the module calling this function
    /// returns.
    ///
    /// If the module command wishes to change something in a different DB and
    /// returns back to the original one, it should call `Context::get_selected_db`
    /// before in order to restore the old DB number before returning.
    pub fn select_db(&self, newid: i32) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_SelectDb.unwrap()(self.ptr, newid) },
            "fail to select db",
        )
    }
    /// Return an handle representing a Redis key with read permission only,
    /// so that it is possible, to call other APIs with the key handle
    /// as argument to perform operations on the key.
    ///
    /// Note The key may be not existed.
    pub fn open_read_key(&self, keyname: &RStr) -> ReadKey {
        ReadKey::new(self.ptr, keyname)
    }
    /// Return an handle representing a Redis key with write permission only,
    /// so that it is possible, to call other APIs with the key handle
    /// as argument to perform operations on the key.
    pub fn open_write_key(&self, keyname: &RStr) -> WriteKey {
        WriteKey::new(self.ptr, keyname)
    }
    /// This function is used in order to potentially unblock a client blocked
    /// on keys with `Context::block_client_on_keys`. When this function is called,
    /// all the clients blocked for this key will get their reply callback called,
    /// and if the callback returns REDISMODULE_OK the client will be unblocked.
    pub fn signal_key_as_ready(&self, key: &RStr) {
        unsafe { raw::RedisModule_SignalKeyAsReady.unwrap()(self.ptr, key.get_ptr()) };
    }
    /// Produces a log message to the standard Redis log
    ///
    /// There is a fixed limit to the length of the log line this function is able
    /// to emit, this limit is not specified but is guaranteed to be more than
    /// a few lines of text.
    pub fn log<T: AsRef<str>>(&self, level: LogLevel, message: T) {
        let level: CString = level.into();
        let fmt = CString::new(message.as_ref()).unwrap();
        unsafe { raw::RedisModule_Log.unwrap()(self.ptr, level.as_ptr(), fmt.as_ptr()) }
    }
    /// Log with notice loglevel
    pub fn notice<T: AsRef<str>>(&self, message: T) {
        self.log(LogLevel::Notice, message.as_ref());
    }
    /// Log with debug loglevel
    pub fn debug<T: AsRef<str>>(&self, message: T) {
        self.log(LogLevel::Debug, message.as_ref());
    }
    /// Log with verbose loglevel
    pub fn verbose<T: AsRef<str>>(&self, message: T) {
        self.log(LogLevel::Verbose, message.as_ref());
    }
    /// Log with warning loglevel
    pub fn warning<T: AsRef<str>>(&self, message: T) {
        self.log(LogLevel::Warning, message.as_ref());
    }
    /// Register a new command in the Redis server
    ///
    /// Note: prefer to use `define_module` macro to regiser command
    pub fn create_cmd(
        &self,
        name: &str,
        func: extern "C" fn(
            *mut raw::RedisModuleCtx,
            *mut *mut raw::RedisModuleString,
            c_int,
        ) -> c_int,
        flags: &str,
        first_key: usize,
        last_key: usize,
        key_step: usize,
    ) -> Result<(), Error> {
        let name = CString::new(name).unwrap();
        let flags = CString::new(flags).unwrap();
        handle_status(
            unsafe {
                raw::RedisModule_CreateCommand.unwrap()(
                    self.ptr,
                    name.as_ptr(),
                    Some(func),
                    flags.as_ptr(),
                    first_key as c_int,
                    last_key as c_int,
                    key_step as c_int,
                )
            },
            "fail to create command",
        )
    }
    /// Deauthenticate and close the client. The client resources will not be
    /// be immediately freed, but will be cleaned up in a background job. This is
    /// the recommended way to deauthenicate a client since most clients can't
    /// handle users becomming deauthenticated.
    ///
    /// The client ID is returned from the `Context:authenticate_client_with_user` and
    /// `Context::authenticate_client_with_acl_user` APIs, but can be obtained through
    /// the CLIENT api or through server events.
    ///
    /// This function is not thread safe, and must be executed within the context
    /// of a command or thread safe context.
    pub fn deauthenticate_and_close_client(&self, id: u64) {
        unsafe { raw::RedisModule_DeauthenticateAndCloseClient.unwrap()(self.ptr, id) }
    }
    /// Authenticate the current context's user with the name of provided redis acl user.
    /// Returns Err if the user is disabled or the user does not exist.
    ///
    /// See `Context::authenticate_client_with_user` for information about callback, client_id,
    /// and general usage for authentication.
    pub fn authenticate_client_with_acl_user<T>(
        &self,
        name: &str,
        callback: raw::RedisModuleUserChangedFunc,
        privdata: Option<T>,
    ) -> Result<u64, Error> {
        let name_ = CString::new(name).unwrap();
        let data = match privdata {
            Some(v) => Box::into_raw(Box::from(v)) as *mut c_void,
            None => 0 as *mut c_void,
        };
        let client_id = std::ptr::null_mut();
        handle_status(
            unsafe {
                raw::RedisModule_AuthenticateClientWithACLUser.unwrap()(
                    self.ptr,
                    name_.as_ptr(),
                    name.len(),
                    callback,
                    data,
                    client_id as *mut u64,
                )
            },
            "fail to authenticate client",
        )?;
        Ok(unsafe { *client_id })
    }
    /// Authenticate the current context's user with the provided redis acl user.
    /// Returns REDISMODULE_ERR if the user is disabled.
    ///
    /// This authentication can be tracked with the optional callback and private
    /// data fields. The callback will be called whenever the user of the client
    /// changes. This callback should be used to cleanup any state that is being
    /// kept in the module related to the client authentication. It will only be
    /// called once, even when the user hasn't changed, in order to allow for a
    /// new callback to be specified. If this authentication does not need to be
    /// tracked, pass in NULL for the callback and privdata.
    ///
    /// If returned client_id can be used with the `Context::deauthenticate_and_close_client`
    /// API in order to deauthenticate a previously authenticated client
    /// if the authentication is no longer valid.
    ///
    /// For expensive authentication operations, it is recommended to block the
    /// client and do the authentication in the background and then attach the user
    /// to the client in a threadsafe context.
    pub fn authenticate_client_with_user<T>(
        &self,
        user: &User,
        callback: raw::RedisModuleUserChangedFunc,
        privdata: Option<T>,
    ) -> Result<u64, Error> {
        let data = match privdata {
            Some(v) => Box::into_raw(Box::from(v)) as *mut c_void,
            None => 0 as *mut c_void,
        };
        let client_id = std::ptr::null_mut();
        handle_status(
            unsafe {
                raw::RedisModule_AuthenticateClientWithUser.unwrap()(
                    self.ptr,
                    user.get_ptr(),
                    callback,
                    data,
                    client_id as *mut u64,
                )
            },
            "fail to authenticate client",
        )?;
        Ok(unsafe { *client_id })
    }
    /// Returns the number of keys in the current db.
    pub fn db_size(&self) -> u64 {
        unsafe { raw::RedisModule_DbSize.unwrap()(self.ptr) }
    }
    /// Publish a message to subscribers (see PUBLISH command).
    pub fn publish_message(&self, channel: &RStr, msg: &RStr) -> Result<(), Error> {
        handle_status(
            unsafe {
                raw::RedisModule_PublishMessage.unwrap()(self.ptr, channel.get_ptr(), msg.get_ptr())
            },
            "fail to publish message",
        )
    }
    /// Signals that the key is modified from user's perspective (i.e. invalidate WATCH
    /// and client side caching).
        pub fn signal_modified_key(&self, key: &RStr) -> Result<(), Error> {
        handle_status(
            unsafe { raw::RedisModule_SignalModifiedKey.unwrap()(self.ptr, key.get_ptr()) },
            "fail to signal key modified",
        )
    }
    /// Set flags defining capabilities or behavior bit flags.
    ///
    /// REDISMODULE_OPTIONS_HANDLE_IO_ERRORS:
    /// Generally, modules don't need to bother with this, as the process will just
    /// terminate if a read error happens, however, setting this flag would allow
    /// repl-diskless-load to work if enabled.
    /// The module should use RedisModule_IsIOError after reads, before using the
    /// data that was read, and in case of error, propagate it upwards, and also be
    /// able to release the partially populated value and all it's allocations.
    pub fn set_module_options(&self, options: i32) {
        unsafe { raw::RedisModule_SetModuleOptions.unwrap()(self.ptr, options) }
    }
    /// Scan API that allows a module to scan all the keys and value in
    /// the selected db.
    ///
    /// Callback for scan implementation.
    /// void scan_callback(RedisModuleCtx *ctx, RedisModuleString *keyname,
    ///                    RedisModuleKey *key, void *privdata);
    /// ctx - the redis module context provided to for the scan.
    /// keyname - owned by the caller and need to be retained if used after this
    /// function.
    ///
    /// key - holds info on the key and value, it is provided as best effort, in
    /// some cases it might be NULL, in which case the user should (can) use
    /// RedisModule_OpenKey (and CloseKey too).
    /// when it is provided, it is owned by the caller and will be free when the
    /// callback returns.
    ///
    /// privdata - the user data provided to RedisModule_Scan.
    ///
    /// The way it should be used:
    /// ```rust,no_run
    /// let cursor = ScanCursor::new();
    /// loop {
    ///     if ctx.scan(cursor, callback, privdata).is_err() { break; }
    /// }
    /// ```
    /// It is also possible to use this API from another thread while the lock
    /// is acquired durring the actuall call to RM_Scan:
    /// ```rust,no_run
    /// let cursor = ScanCursor::new();
    /// ctx.lock();
    /// loop {
    ///     if ctx.scan(cursor, callback, privdata).is_err() { break; }
    ///     ctx.unlock();
    ///     // do some background job
    ///     ctx.lock()
    /// }
    /// ```
    ///
    /// The function will return 1 if there are more elements to scan and
    /// 0 otherwise, possibly setting errno if the call failed.
    ///
    /// It is also possible to restart and existing cursor using RM_CursorRestart.
    ///
    /// IMPORTANT: This API is very similar to the Redis SCAN command from the
    /// point of view of the guarantees it provides. This means that the API
    /// may report duplicated keys, but guarantees to report at least one time
    /// every key that was there from the start to the end of the scanning process.
    ///
    /// NOTE: If you do database changes within the callback, you should be aware
    /// that the internal state of the database may change. For instance it is safe
    /// to delete or modify the current key, but may not be safe to delete any
    /// other key.
    /// Moreover playing with the Redis keyspace while iterating may have the
    /// effect of returning more duplicates. A safe pattern is to store the keys
    /// names you want to modify elsewhere, and perform the actions on the keys
    /// later when the iteration is complete. Howerver this can cost a lot of
    /// memory, so it may make sense to just operate on the current key when
    /// possible during the iteration, given that this is safe.
    pub fn scan<T>(
        &self,
        cursor: &ScanCursor,
        callback: raw::RedisModuleScanCB,
        privdata: Option<T>,
    ) -> Result<(), Error> {
        let data = match privdata {
            Some(v) => Box::into_raw(Box::from(v)) as *mut c_void,
            None => 0 as *mut c_void,
        };
        handle_status(
            unsafe { raw::RedisModule_Scan.unwrap()(self.ptr, cursor.get_ptr(), callback, data) },
            "fail to scan",
        )
    }

    /// This function is called by a module in order to export some API with a
    /// given name. Other modules will be able to use this API by calling the
    /// symmetrical function `Context::get_shared_api` and casting the return value to
    /// the right function pointer.
    ///
    /// IMPORTANT: the apiname argument should be a string literal with static
    /// lifetime. The API relies on the fact that it will always be valid in
    /// the future.
    pub fn export_shared_api(&self, name: &str, fn_ptr: *mut c_void) -> Result<(), Error> {
        let name = CString::new(name).unwrap();
        handle_status(
            unsafe { raw::RedisModule_ExportSharedAPI.unwrap()(self.ptr, name.as_ptr(), fn_ptr) },
            "fail to export shared api",
        )
    }
    /// Request an exported API pointer. The return value is just a void pointer
    /// that the caller of this function will be required to cast to the right
    /// function pointer, so this is a private contract between modules.
    ///
    /// If the requested API is not available then None is returned. Because
    /// modules can be loaded at different times with different order, this
    /// function calls should be put inside some module generic API registering
    /// step, that is called every time a module attempts to execute a
    /// command that requires external APIs: if some API cannot be resolved, the
    /// command should return an error.
    pub fn get_shared_api(&self, name: &str) -> Option<*mut c_void> {
        let name = CString::new(name).unwrap();
        let ptr: *mut c_void =
            unsafe { raw::RedisModule_GetSharedAPI.unwrap()(self.ptr, name.as_ptr()) };
        if ptr.is_null() {
            return None;
        }
        Some(ptr)
    }
    /// keyspace-notifications API. A module can register callbacks to be notified
    /// when keyspce events occur.
    ///
    /// Notification events are filtered by their type (string events, set events,
    /// etc), and the subscriber callback receives only events that match a specific
    /// mask of event types.
    ///
    /// When subscribing to notifications with RedisModule_SubscribeToKeyspaceEvents
    /// the module must provide an event type-mask, denoting the events the subscriber
    /// is interested in. This can be an ORed mask of any of the following flags:
    ///
    ///  - REDISMODULE_NOTIFY_GENERIC: Generic commands like DEL, EXPIRE, RENAME
    ///  - REDISMODULE_NOTIFY_STRING: String events
    ///  - REDISMODULE_NOTIFY_LIST: List events
    ///  - REDISMODULE_NOTIFY_SET: Set events
    ///  - REDISMODULE_NOTIFY_HASH: Hash events
    ///  - REDISMODULE_NOTIFY_ZSET: Sorted Set events
    ///  - REDISMODULE_NOTIFY_EXPIRED: Expiration events
    ///  - REDISMODULE_NOTIFY_EVICTED: Eviction events
    ///  - REDISMODULE_NOTIFY_STREAM: Stream events
    ///  - REDISMODULE_NOTIFY_KEYMISS: Key-miss events
    ///  - REDISMODULE_NOTIFY_ALL: All events (Excluding REDISMODULE_NOTIFY_KEYMISS)
    ///
    /// We do not distinguish between key events and keyspace events, and it is up
    /// to the module to filter the actions taken based on the key.
    ///
    ///
    /// `type` is the event type bit, that must match the mask given at registration
    /// time. The event string is the actual command being executed, and key is the
    /// relevant Redis key.
    ///
    /// Notification callback gets executed with a redis context that can not be
    /// used to send anything to the client, and has the db number where the event
    /// occurred as its selected db number.
    ///
    /// Notice that it is not necessary to enable notifications in redis.conf for
    /// module notifications to work.
    ///
    /// Warning: the notification callbacks are performed in a synchronous manner,
    /// so notification callbacks must to be fast, or they would slow Redis down.
    /// If you need to take long actions, use threads to offload them.
    ///
    /// See https://redis.io/topics/notifications for more information.
    pub fn subscribe_to_keyspace_events(
        &self,
        types: i32,
        callback: raw::RedisModuleNotificationFunc,
    ) -> Result<(), Error> {
        handle_status(
            unsafe {
                raw::RedisModule_SubscribeToKeyspaceEvents.unwrap()(self.ptr, types, callback)
            },
            "fail to subscribe to keyspace events",
        )
    }

    /// Register to be notified, via a callback, when the specified server event
    /// happens. The callback is called with the event as argument, and an additional
    /// argument which is a void pointer and should be cased to a specific type
    /// that is event-specific (but many events will just use NULL since they do not
    /// have additional information to pass to the callback).
    ///
    /// If the callback is NULL and there was a previous subscription, the module
    /// will be unsubscribed. If there was a previous subscription and the callback
    /// is not null, the old callback will be replaced with the new one.
    ///
    ///
    /// The 'ctx' is a normal Redis module context that the callback can use in
    /// order to call other modules APIs. The 'eid' is the event itself, this
    /// is only useful in the case the module subscribed to multiple events: using
    /// the 'id' field of this structure it is possible to check if the event
    /// is one of the events we registered with this callback. The 'subevent' field
    /// depends on the event that fired.
    ///
    /// Finally the 'data' pointer may be populated, only for certain events, with
    /// more relevant data.
    ///
    /// Here is a list of events you can use as 'eid' and related sub events:
    ///
    ///      RedisModuleEvent_ReplicationRoleChanged
    ///
    ///          This event is called when the instance switches from master
    ///          to replica or the other way around, however the event is
    ///          also called when the replica remains a replica but starts to
    ///          replicate with a different master.
    ///
    ///          The following sub events are available:
    ///
    ///              REDISMODULE_SUBEVENT_REPLROLECHANGED_NOW_MASTER
    ///              REDISMODULE_SUBEVENT_REPLROLECHANGED_NOW_REPLICA
    ///
    ///          The 'data' field can be casted by the callback to a
    ///          RedisModuleReplicationInfo structure with the following fields:
    ///
    ///              int master; // true if master, false if replica
    ///              char *masterhost; // master instance hostname for NOW_REPLICA
    ///              int masterport; // master instance port for NOW_REPLICA
    ///              char *replid1; // Main replication ID
    ///              char *replid2; // Secondary replication ID
    ///              uint64_t repl1_offset; // Main replication offset
    ///              uint64_t repl2_offset; // Offset of replid2 validity
    ///
    ///      RedisModuleEvent_Persistence
    ///
    ///          This event is called when RDB saving or AOF rewriting starts
    ///          and ends. The following sub events are available:
    ///
    ///              REDISMODULE_SUBEVENT_PERSISTENCE_RDB_START
    ///              REDISMODULE_SUBEVENT_PERSISTENCE_AOF_START
    ///              REDISMODULE_SUBEVENT_PERSISTENCE_SYNC_RDB_START
    ///              REDISMODULE_SUBEVENT_PERSISTENCE_ENDED
    ///              REDISMODULE_SUBEVENT_PERSISTENCE_FAILED
    ///
    ///          The above events are triggered not just when the user calls the
    ///          relevant commands like BGSAVE, but also when a saving operation
    ///          or AOF rewriting occurs because of internal server triggers.
    ///          The SYNC_RDB_START sub events are happening in the forground due to
    ///          SAVE command, FLUSHALL, or server shutdown, and the other RDB and
    ///          AOF sub events are executed in a background fork child, so any
    ///          action the module takes can only affect the generated AOF or RDB,
    ///          but will not be reflected in the parent process and affect connected
    ///          clients and commands. Also note that the AOF_START sub event may end
    ///          up saving RDB content in case of an AOF with rdb-preamble.
    ///
    ///      RedisModuleEvent_FlushDB
    ///
    ///          The FLUSHALL, FLUSHDB or an internal flush (for instance
    ///          because of replication, after the replica synchronization)
    ///          happened. The following sub events are available:
    ///
    ///              REDISMODULE_SUBEVENT_FLUSHDB_START
    ///              REDISMODULE_SUBEVENT_FLUSHDB_END
    ///
    ///          The data pointer can be casted to a RedisModuleFlushInfo
    ///          structure with the following fields:
    ///
    ///              int32_t async;  // True if the flush is done in a thread.
    ///                                 See for instance FLUSHALL ASYNC.
    ///                                 In this case the END callback is invoked
    ///                                 immediately after the database is put
    ///                                 in the free list of the thread.
    ///              int32_t dbnum;  // Flushed database number, -1 for all the DBs
    ///                                 in the case of the FLUSHALL operation.
    ///
    ///          The start event is called *before* the operation is initated, thus
    ///          allowing the callback to call DBSIZE or other operation on the
    ///          yet-to-free keyspace.
    ///
    ///      RedisModuleEvent_Loading
    ///
    ///          Called on loading operations: at startup when the server is
    ///          started, but also after a first synchronization when the
    ///          replica is loading the RDB file from the master.
    ///          The following sub events are available:
    ///
    ///              REDISMODULE_SUBEVENT_LOADING_RDB_START
    ///              REDISMODULE_SUBEVENT_LOADING_AOF_START
    ///              REDISMODULE_SUBEVENT_LOADING_REPL_START
    ///              REDISMODULE_SUBEVENT_LOADING_ENDED
    ///              REDISMODULE_SUBEVENT_LOADING_FAILED
    ///
    ///          Note that AOF loading may start with an RDB data in case of
    ///          rdb-preamble, in which case you'll only recieve an AOF_START event.
    ///
    ///
    ///      RedisModuleEvent_ClientChange
    ///
    ///          Called when a client connects or disconnects.
    ///          The data pointer can be casted to a RedisModuleClientInfo
    ///          structure, documented in RedisModule_GetClientInfoById().
    ///          The following sub events are available:
    ///
    ///              REDISMODULE_SUBEVENT_CLIENT_CHANGE_CONNECTED
    ///              REDISMODULE_SUBEVENT_CLIENT_CHANGE_DISCONNECTED
    ///
    ///      RedisModuleEvent_Shutdown
    ///
    ///          The server is shutting down. No subevents are available.
    ///
    ///  RedisModuleEvent_ReplicaChange
    ///
    ///          This event is called when the instance (that can be both a
    ///          master or a replica) get a new online replica, or lose a
    ///          replica since it gets disconnected.
    ///          The following sub events are availble:
    ///
    ///              REDISMODULE_SUBEVENT_REPLICA_CHANGE_ONLINE
    ///              REDISMODULE_SUBEVENT_REPLICA_CHANGE_OFFLINE
    ///
    ///          No additional information is available so far: future versions
    ///          of Redis will have an API in order to enumerate the replicas
    ///          connected and their state.
    ///
    ///  RedisModuleEvent_CronLoop
    ///
    ///          This event is called every time Redis calls the serverCron()
    ///          function in order to do certain bookkeeping. Modules that are
    ///          required to do operations from time to time may use this callback.
    ///          Normally Redis calls this function 10 times per second, but
    ///          this changes depending on the "hz" configuration.
    ///          No sub events are available.
    ///
    ///          The data pointer can be casted to a RedisModuleCronLoop
    ///          structure with the following fields:
    ///
    ///              int32_t hz;  // Approximate number of events per second.
    ///
    ///  RedisModuleEvent_MasterLinkChange
    ///
    ///          This is called for replicas in order to notify when the
    ///          replication link becomes functional (up) with our master,
    ///          or when it goes down. Note that the link is not considered
    ///          up when we just connected to the master, but only if the
    ///          replication is happening correctly.
    ///          The following sub events are available:
    ///
    ///              REDISMODULE_SUBEVENT_MASTER_LINK_UP
    ///              REDISMODULE_SUBEVENT_MASTER_LINK_DOWN
    ///
    ///  RedisModuleEvent_ModuleChange
    ///
    ///          This event is called when a new module is loaded or one is unloaded.
    ///          The following sub events are availble:
    ///
    ///              REDISMODULE_SUBEVENT_MODULE_LOADED
    ///              REDISMODULE_SUBEVENT_MODULE_UNLOADED
    ///
    ///          The data pointer can be casted to a RedisModuleModuleChange
    ///          structure with the following fields:
    ///
    ///              const char* module_name;  // Name of module loaded or unloaded.
    ///              int32_t module_version;  // Module version.
    ///
    ///  RedisModuleEvent_LoadingProgress
    ///
    ///          This event is called repeatedly called while an RDB or AOF file
    ///          is being loaded.
    ///          The following sub events are availble:
    ///
    ///              REDISMODULE_SUBEVENT_LOADING_PROGRESS_RDB
    ///              REDISMODULE_SUBEVENT_LOADING_PROGRESS_AOF
    ///
    ///          The data pointer can be casted to a RedisModuleLoadingProgress
    ///          structure with the following fields:
    ///
    ///              int32_t hz;  // Approximate number of events per second.
    ///              int32_t progress;  // Approximate progress between 0 and 1024,
    ///                                    or -1 if unknown.
    ///
    /// The function returns REDISMODULE_OK if the module was successfully subscrived
    /// for the specified event. If the API is called from a wrong context then
    /// REDISMODULE_ERR is returned.
    pub fn subscribe_to_server_event(
        &self,
        events: ServerEvent,
        callback: raw::RedisModuleEventCallback,
    ) -> Result<(), Error> {
        handle_status(
            unsafe {
                raw::RedisModule_SubscribeToServerEvent.unwrap()(self.ptr, events.into(), callback)
            },
            "fail to subscribe to keyspace events",
        )
    }

    /// Notify keyspace event to redis core to broadcast
    pub fn notify_keyspace_event<T: AsRef<str>>(&self, type_: i32, event: T, key: &RStr) -> Result<(), Error> {
        let event = CString::new(event.as_ref()).unwrap();
        handle_status(
            unsafe {
                raw::RedisModule_NotifyKeyspaceEvent.unwrap()(
                    self.ptr,
                    type_,
                    event.as_ptr(),
                    key.get_ptr(),
                )
            },
            "fail to notify keyspace event",
        )
    }
}
