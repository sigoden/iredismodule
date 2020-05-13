//! Helper macros

/// Assert the length of args, throw `Error::WrongArity` when failed
///
/// # Examples
/// ```rust,no_run
/// #[rcmd("hello.repl2", "write", 1, 1, 1)]
/// fn hello_repl2(ctx: &mut Context, args: Vec<RStr>) -> RResult {
///     assert_len!(args, 2);
///     ...
/// }
///
/// ```
#[macro_export]
macro_rules! assert_len {
    ($args:expr, $n:expr) => {
        if $args.len() != $n {
            return Err(Error::WrongArity);
        }
    };
}

/// Create a redis module will be so easy
#[macro_export]
macro_rules! define_module {
    (
        name: $module_name:expr,
        version: $module_version:expr,
        data_types: [
            $($data_type:ident),* $(,)*
        ],
        init_funcs: [
            $($init_func:ident),* $(,)*
        ],
        commands: [
            $($command:ident),* $(,)*
        ]
        $(,)*
    ) => {
        #[no_mangle]
        #[allow(non_snake_case)]
        pub extern "C" fn RedisModule_OnLoad(
            ctx: *mut $crate::raw::RedisModuleCtx,
            argv: *mut *mut $crate::raw::RedisModuleString,
            argc: std::os::raw::c_int,
        ) -> std::os::raw::c_int {
            let mut name_buffer = [0; 64];
            unsafe {
                std::ptr::copy(
                    $module_name.as_ptr(),
                    name_buffer.as_mut_ptr(),
                    $module_name.len(),
                );
            }
            let c_err = $crate::raw::REDISMODULE_ERR as std::os::raw::c_int;
            let module_version = $module_version as std::os::raw::c_int;
            if unsafe {
                $crate::raw::Export_RedisModule_Init(
                    ctx,
                    name_buffer.as_ptr() as *const std::os::raw::c_char,
                    module_version,
                    $crate::raw::REDISMODULE_APIVER_1 as std::os::raw::c_int,
                )
            } == c_err {
                return c_err;
            }
            let mut context = $crate::context::Context::from_ptr(ctx);

            $(
                if $init_func(ctx, argv, argc) == c_err {
                    return c_err;
                }
            )*

            $(
                if (&$data_type).create(&mut context).is_err() {
                    return c_err;
                }
            )*

            $(
                if $command(&mut context).is_err() {
                    return c_err;
                }
            )*
            $crate::raw::REDISMODULE_OK as std::os::raw::c_int
        }

    }
}
