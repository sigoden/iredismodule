extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn, AttributeArgs, Ident};
use proc_macro2::Span;
use darling::FromMeta;
use quote::{quote};

#[derive(Debug, FromMeta)]
struct CmdAttributeOpts {
    name: String,
    flags: String,
    first_key: usize,
    last_key: usize,
    key_step: usize,
}

#[proc_macro_attribute]
pub fn cmd(attr: TokenStream, input: TokenStream) -> TokenStream {
    let cmd_fn = syn::parse::<ItemFn>(input).expect("fail to parse fn");
    let attr_args =  parse_macro_input!(attr as AttributeArgs);
    let opts: CmdAttributeOpts = CmdAttributeOpts::from_list(&attr_args).expect("fail to parse fn attrs");
    let fn_name = cmd_fn.sig.ident.clone();
    let vis = cmd_fn.vis.clone();

    let raw_fn_name = Ident::new(&format!("{}_raw", &fn_name), Span::call_site());
    let create_fn_name = Ident::new(&format!("create_{}", &fn_name), Span::call_site());
    let name = opts.name.clone();
    let flags = opts.flags.clone();
    let first_key = opts.first_key;
    let last_key = opts.last_key;
    let key_step = opts.key_step;
    let raw_fn = quote! {
        #vis extern "C" fn #raw_fn_name(
            ctx: *mut redismodule::raw::RedisModuleCtx,
            argv: *mut *mut redismodule::raw::RedisModuleString,
            argc: std::os::raw::c_int
        ) -> std::os::raw::c_int {
            let context = redismodule::Context::from_ptr(ctx);
            let response = #fn_name(&context, redismodule::parse_args(argv, argc));
            context.reply(response) as std::os::raw::c_int
        }
    };
    let create_fn = quote! {
        #vis fn #create_fn_name(ctx: &mut redismodule::Context) -> Result<(), redismodule::Error> {
            ctx.create_command(#name, #raw_fn_name, #flags, #first_key, #last_key, #key_step)
        }
    };
    let output = quote! {
        #cmd_fn
        #raw_fn
        #create_fn
    };
    TokenStream::from(output)
}


#[proc_macro_attribute]
pub fn init(attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}
