use proc_macro::{TokenStream};
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Ident, Lit};

pub fn rwrap(attr: TokenStream, input: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(input as syn::ItemFn);
    let attr_args =  parse_macro_input!(attr as syn::AttributeArgs);
    let kind = rwrap_parse_attrs(attr_args).expect("fail to parse attr");
    match kind.as_str() {
        "cluster_msg" => cluster_msg(item_fn),
        "free" => free(item_fn),
        _ => panic!("unsupported the kind of wrap fn"),
    }
}


fn rwrap_parse_attrs(args: syn::AttributeArgs) -> Option<String> {
    if let Some(syn::NestedMeta::Lit(Lit::Str(kind))) = args.first() {
        Some(kind.value())
    } else {
        None
    }
}

fn cluster_msg(item_fn: syn::ItemFn) -> TokenStream {
    let fn_name = item_fn.sig.ident.clone();
    let c_fn_name = Ident::new(&format!("{}_c", &fn_name), Span::call_site());
    let vis = item_fn.vis.clone();

    let output = quote! {
        #vis extern "C" fn #c_fn_name(
            ctx: *mut raw::RedisModuleCtx,
            sender_id: *const std::os::raw::c_char,
            type_: u8,
            payload: *const std::os::raw::c_uchar,
            len: u32,
        ) {
            let mut context = redismodule::Context::from_ptr(ctx);
            let sender_id = std::str::from_utf8(unsafe {
                std::slice::from_raw_parts(
                    sender_id as *const std::os::raw::c_uchar,
                    redismodule::raw::REDISMODULE_NODE_ID_LEN as usize,
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

pub fn free(item_fn: syn::ItemFn) -> TokenStream {
    let fn_name = item_fn.sig.ident.clone();
    let c_fn_name = Ident::new(&format!("{}_c", &fn_name), Span::call_site());
    let fn_arg_2 = item_fn
        .sig
        .inputs
        .last()
        .cloned()
        .expect("fn need 2 parameters");
    let fn_arg_2_type = free_get_param2_type(fn_arg_2).expect("fn 2nd parameter must be Box<T>");
    let vis = item_fn.vis.clone();
    let c_fn = quote! {
        #vis extern "C" fn #c_fn_name(
            ctx: *mut redismodule::raw::RedisModuleCtx,
            data: *mut std::os::raw::c_void
        ) {
            let mut context = redismodule::Context::from_ptr(ctx);
            let data = data as *mut #fn_arg_2_type;
            let data = unsafe { Box::from_raw(data) };
            #fn_name(&mut context, data);
        }
    };
    let output = quote! {
        #c_fn
        #item_fn
    };
    TokenStream::from(output)
}

fn free_get_param2_type(fn_arg: syn::FnArg) -> Option<syn::Type> {
    if let syn::FnArg::Typed(syn::PatType { ty, .. }) = fn_arg {
        if let syn::Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) = ty.as_ref()
        {
            if let syn::PathSegment {
                arguments:
                    syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                        args, ..
                    }),
                ..
            } = segments.first().expect("fn 2nd params must be Box<T>")
            {
                if let syn::GenericArgument::Type(v) = args.first().unwrap().clone() {
                    return Some(v.clone());
                }
            }
        }
    }
    None
}