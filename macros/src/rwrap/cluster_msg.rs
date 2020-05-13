use proc_macro::{TokenStream};
use proc_macro2::Span;
use quote::quote;
use syn::Ident;

pub fn cluster_msg(item_fn: syn::ItemFn) -> TokenStream {
    let fn_name = item_fn.sig.ident.clone();
    let c_fn_name = Ident::new(&format!("{}_c", &fn_name), Span::call_site());
    let vis = item_fn.vis.clone();

    let output = quote! {
        #vis extern "C" fn #c_fn_name(
            ctx: *mut iredismodule::raw::RedisModuleCtx,
            sender_id: *const std::os::raw::c_char,
            type_: u8,
            payload: *const std::os::raw::c_uchar,
            len: u32,
        ) {
            let mut context = iredismodule::context::Context::from_ptr(ctx);
            let sender_id = std::str::from_utf8(unsafe {
                std::slice::from_raw_parts(
                    sender_id as *const std::os::raw::c_uchar,
                    iredismodule::raw::REDISMODULE_NODE_ID_LEN as usize,
                )
            })
            .unwrap();
            let payload = unsafe { std::slice::from_raw_parts(payload, len as usize) };

            #fn_name(&mut context, sender_id, type_, payload)
        }

        #item_fn
    };

    TokenStream::from(output)
}
