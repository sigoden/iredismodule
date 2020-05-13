use proc_macro::{TokenStream};
use proc_macro2::Span;
use quote::quote;
use syn::Ident;

pub fn call(item_fn: syn::ItemFn) -> TokenStream {
    let fn_name = item_fn.sig.ident.clone();
    let c_fn_name = Ident::new(&format!("{}_c", &fn_name), Span::call_site());
    let is_rresult = call_is_ret_rresult(item_fn.sig.output.clone());
    let vis = item_fn.vis.clone();
    let bottom_expr = if is_rresult {
        quote! {
            return context.reply(result) as std::os::raw::c_int;
        }
    } else {
        quote! {
            return redismodule::raw::REDISMODULE_OK as std::os::raw::c_int;
        }
    };
    let output = quote! {
        #vis extern "C" fn #c_fn_name(
            ctx: *mut redismodule::raw::RedisModuleCtx,
            argv: *mut *mut redismodule::raw::RedisModuleString,
            argc: std::os::raw::c_int,
        ) -> std::os::raw::c_int {
            let args = redismodule::parse_args(argv, argc);
            let mut context = redismodule::context::Context::from_ptr(ctx);
            let result = #fn_name(&mut context, args);
            if result.is_err() {
                return redismodule::raw::REDISMODULE_ERR as std::os::raw::c_int;
            }
            #bottom_expr
        }
        #item_fn
    };
    TokenStream::from(output)
}

fn call_is_ret_rresult(ty: syn::ReturnType) -> bool {
    if let syn::ReturnType::Type(_, ty2) = ty {
        if let syn::Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) = ty2.as_ref()
        {
            if let Some(syn::PathSegment { ident, .. }) = segments.last() {
                return ident.to_string() == "RResult";
            }
        }
    }
    return false;
}

