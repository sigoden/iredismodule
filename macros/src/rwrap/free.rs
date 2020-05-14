use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::Ident;

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
            ctx: *mut iredismodule::raw::RedisModuleCtx,
            data: *mut std::os::raw::c_void
        ) {
            use iredismodule::FromPtr;
            let mut context = iredismodule::context::Context::from_ptr(ctx);
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
