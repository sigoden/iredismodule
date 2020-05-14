extern crate proc_macro;

use proc_macro::TokenStream;

mod rcmd;
mod rtypedef;
mod rwrap;

/// A wrapper of module command func.
///
/// It's can have five attrs.
/// ```
/// #[rcmd("hello.leftpad", "", 0, 0, 0)]
/// #[rcmd("hello.leftpad")]
/// #[rcmd("helloacl.authglobal", "no-auth")]
/// #[rcmd("hello.hcopy", "write deny-oom", 1, 1, 1)]
/// ```
///
/// The first attr command name, it is required.
///
/// The second attr is  'strflags' specify the behavior of the command and should
/// be passed as a C string composed of space separated words, like for
/// example "write deny-oom". The set of flags are:
/// * **"write"**:     The command may modify the data set (it may also read
///                    from it).
/// * **"readonly"**:  The command returns data from keys but never writes.
/// * **"admin"**:     The command is an administrative command (may change
///                    replication or perform similar tasks).
/// * **"deny-oom"**:  The command may use additional memory and should be
///                    denied during out of memory conditions.
/// * **"deny-script"**:   Don't allow this command in Lua scripts.
/// * **"allow-loading"**: Allow this command while the server is loading data.
///                        Only commands not interacting with the data set
///                        should be allowed to run in this mode. If not sure
///                        don't use this flag.
/// * **"pubsub"**:    The command publishes things on Pub/Sub channels.
/// * **"random"**:    The command may have different outputs even starting
///                    from the same input arguments and key values.
/// * **"allow-stale"**: The command is allowed to run on slaves that don't
///                      serve stale data. Don't use if you don't know what
///                      this means.
/// * **"no-monitor"**: Don't propagate the command on monitor. Use this if
///                     the command has sensible data among the arguments.
/// * **"no-slowlog"**: Don't log this command in the slowlog. Use this if
///                     the command has sensible data among the arguments.
/// * **"fast"**:      The command time complexity is not greater
///                    than O(log(N)) where N is the size of the collection or
///                    anything else representing the normal scalability
///                    issue with the command.
/// * **"getkeys-api"**: The command implements the interface to return
///                      the arguments that are keys. Used when start/stop/step
///                      is not enough because of the command syntax.
/// * **"no-cluster"**: The command should not register in Redis Cluster
///                     since is not designed to work with it because, for
///                     example, is unable to report the position of the
///                     keys, programmatically creates key names, or any
///                     other reason.
/// * **"no-auth"**:    This command can be run by an un-authenticated client.
///                     Normally this is used by a command that is used
///                     to authenticate a client.
///
/// The las three attrs means first_key, last_key and key_step.
///
/// ```rust,no_run
/// #[rcmd("hello.simple", "readonly", 0, 0, 0)]
/// fn hello_simple(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
///     let db = ctx.get_select_db();
///     Ok(db.into())
/// }
/// ```
///
/// This macro expands above code:
/// ```rust,no_run
/// fn hello_simple(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
///     let db = ctx.get_select_db();
///     Ok(db.into())
/// }
/// extern "C" fn hello_simple_c(
///     ctx: *mut iredismodule::raw::RedisModuleCtx,
///     argv: *mut *mut iredismodule::raw::RedisModuleString,
///     argc: std::os::raw::c_int,
/// ) -> std::os::raw::c_int {
///     use iredismodule::FromPtr;
///     let mut context = iredismodule::context::Context::from_ptr(ctx);
///     let response = hello_simple(&mut context, iredismodule::parse_args(argv, argc));
///     context.reply(response);
///     iredismodule::raw::REDISMODULE_OK as std::os::raw::c_int
/// }
/// fn hello_simple_cmd(
///     ctx: &mut iredismodule::context::Context,
/// ) -> Result<(), iredismodule::error::Error> {
///     ctx.create_cmd(
///         "hello.simple",
///         hello_simple_c,
///         "readonly",
///         0usize,
///         0usize,
///         0usize,
///     )
/// }
/// ```
/// The `hello_simple` fn is the origin fn.
///
/// The `hello_simple_c` fn is a c wrapper function which will be apply to ffi or callback.
///
/// The `hello_simple_cmd` fn is entrypoint to register command.
///
/// The `***_cmd` fn will be applyed to `define_macro`
/// ```rust,no_run
/// define_module! {
///     name: "simple",
///     version: 1,
///     data_types: [],
///     init_funcs: [],
///     commands: [
///         hello_simple_cmd, // <- used here
///     ]
/// }
#[proc_macro_attribute]
pub fn rcmd(attr: TokenStream, input: TokenStream) -> TokenStream {
    rcmd::rcmd(attr, input)
}

/// This macro will be used to define a module type.
///
/// It must be used in `impl TypeMethod for T`.
///
/// It have two attr value.
///
/// * **name**: A 9 characters data type name that MUST be unique in the Redis
///   Modules ecosystem. Be creative... and there will be no collisions. Use
///   the charset A-Z a-z 9-0, plus the two "-_" characters. A good
///   idea is to use, for example `<typename>-<vendor>`. For example
///   "tree-AntZ" may mean "Tree data structure by @antirez". To use both
///   lower case and upper case letters helps in order to prevent collisions.
/// * **encver**: Encoding version, which is, the version of the serialization
///   that a module used in order to persist data. As long as the "name"
///   matches, the RDB loading will be dispatched to the type callbacks
///   whatever 'encver' is used, however the module can understand if
///   the encoding it must load are of an older version of the module.
///   For example the module "tree-AntZ" initially used encver=0. Later
///   after an upgrade, it started to serialize data in a different format
///   and to register the type with encver=1. However this module may
///   still load old data produced by an older version if the rdb_load
///   callback is able to check the encver value and act accordingly.
///   The encver must be a positive value between 0 and 1023.
///
/// ```rust,no_run
/// #[rtypedef("hellotype", 0)]
/// impl TypeMethod for HelloTypeNode {
///     fn rdb_load(io: &mut IO, encver: u32) -> Option<Box<Self>> {
///         if encver != 0 {
///             return None;
///         }
///         let elements = io.load_unsigned();
///         let mut hto = Self::new();
///         for _ in 0..elements {
///             let ele = io.load_signed();
///             hto.push(ele);
///         }
///         Some(Box::new(hto))
///     }
///     fn rdb_save(&self, io: &mut IO) {
///         let eles: Vec<&i64> = self.iter().collect();
///         io.save_unsigned(eles.len() as u64);
///         eles.iter().for_each(|v| io.save_signed(**v));
///     }
///     fn free(_: Box<Self>) {}
///     fn aof_rewrite<T: AsRef<str>>(&self, io: &mut IO, key: T) {
///         let eles: Vec<&i64> = self.iter().collect();
///         let keyname = key.as_ref();
///         eles.iter().for_each(|v| {
///             io.emit_aof(
///                 "HELLOTYPE.INSERT",
///                 &[keyname, &v.to_string()],
///             )
///         })
///     }
///     fn mem_usage(&self) -> usize {
///         std::mem::size_of::<Self>() * self.len()
///     }
///     fn digest(&self, digest: &mut Digest) {
///         let eles: Vec<&i64> = self.iter().collect();
///         eles.iter().for_each(|v| digest.add_long_long(**v));
///         digest.end_sequeue();
///     }
/// }
/// ```
///
/// The macro will generate static variable which repersent the data type. The variabe name
/// is generated by switching to uppercase and replace "-" with "_".
///
/// The methods of trait will be expand to extern "C" fn and will be used to set the
/// value of RedisModuleTypeMethods fields.
///
/// For example. The macro will generate `hellotype_rdb_save` based on method `rdb_save`.
/// ```rust,no_run
/// unsafe extern "C" fn hellotype_rdb_save(
///     rdb: *mut iredismodule::raw::RedisModuleIO,
///     value: *mut std::os::raw::c_void,
/// ) {
///     use iredismodule::FromPtr;
///     let mut io = iredismodule::io::IO::from_ptr(rdb);
///     let hto = &*(value as *mut HelloTypeNode);
///     hto.rdb_save(&mut io)
/// }
/// ```
/// If the method is ommited, the value will be set none in construct `RedisModuleTypeMethods`.
///
/// ```rust,no_run
/// pub static HELLOTYPE: iredismodule::rtype::RType<HelloTypeNode> = iredismodule::rtype::RType::new(
///     "hellotype",
///     0i32,
///     iredismodule::raw::RedisModuleTypeMethods {
///         version: iredismodule::raw::REDISMODULE_TYPE_METHOD_VERSION as u64,
///         rdb_load: Some(hellotype_rdb_load),
///         rdb_save: Some(hellotype_rdb_save),
///         aof_rewrite: Some(hellotype_aof_rewrite),
///         mem_usage: Some(hellotype_mem_usage),
///         free: Some(hellotype_free),
///         digest: Some(hellotype_digest),
///         aux_load: None,
///         aux_save: None,
///         aux_save_triggers: HelloTypeNode::AUX_SAVE_TRIGGERS as i32,
///     },
/// );
///
/// Finally, use `define_macro` to register that data type.
/// ```rust,no_run
/// define_module! {
///     name: "hellotype",
///     version: 1,
///     data_types: [
///         HELLOTYPE,
///     ],
///     init_funcs: [],
///     commands: [
///         ...
///     ],
/// }
/// ```
#[proc_macro_attribute]
pub fn rtypedef(attr: TokenStream, input: TokenStream) -> TokenStream {
    rtypedef::rtypedef(attr, input)
}

/// Wrap of all kind of ffi fn and callback.
///
/// The first attr value is a kind, it's point out what kind of function to be wrapped.
///
/// ## **free**  - wrap a free callback
///
/// ```rust,no_run
/// #[rwrap("free")]
/// fn helloblock_free(ctx: &mut Context, data: Box<String>) { }
/// ```
/// The code above will be expanded below
/// ```rust,no_run
/// extern "C" fn helloblock_free_c(
///     ctx: *mut iredismodule::raw::RedisModuleCtx,
///     data: *mut std::os::raw::c_void,
/// ) {
///     use iredismodule::FromPtr;
///     let mut context = iredismodule::context::Context::from_ptr(ctx);
///     let data = data as *mut String;
///     let data = unsafe { Box::from_raw(data) };
///     helloblock_free(&mut context, data);
/// }
/// fn helloblock_free(ctx: &mut Context, data: Box<String>) {}
///
/// ```
/// ## **cmd** - wrap a call callback
///
/// ```rust,no_run
/// #[rwrap("call")]
/// fn helloblock_reply(ctx: &mut Context, _: Vec<RStr>) -> RResult {}
/// ```
/// The code above will be expanded below
/// ```rust,no_run
/// extern "C" fn helloblock_reply_c(
///     ctx: *mut iredismodule::raw::RedisModuleCtx,
///     argv: *mut *mut iredismodule::raw::RedisModuleString,
///     argc: std::os::raw::c_int,
/// ) -> std::os::raw::c_int {
///     let args = iredismodule::parse_args(argv, argc);
///     let mut context = iredismodule::context::Context::from_ptr(ctx);
///     let result = helloblock_reply(&mut context, args);
///     if result.is_err() {
///         return iredismodule::raw::REDISMODULE_ERR as std::os::raw::c_int;
///     }
///     context.reply(result);
///     return iredismodule::raw::REDISMODULE_OK as std::os::raw::c_int;
/// }
/// fn helloblock_reply(ctx: &mut Context, _: Vec<RStr>) -> RResult {
/// ```
#[proc_macro_attribute]
pub fn rwrap(attr: TokenStream, input: TokenStream) -> TokenStream {
    rwrap::rwrap(attr, input)
}
