use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::collections::HashSet;
use syn::{parse_macro_input, Ident, Lit};

#[derive(Debug)]
struct TypeDefAttributeOpts {
    name: String,
    version: i32,
}

pub fn rtypedef(attr: TokenStream, input: TokenStream) -> TokenStream {
    let item_impl = parse_macro_input!(input as syn::ItemImpl);
    let attr_args = parse_macro_input!(attr as syn::AttributeArgs);
    let opts = rtypedef_parse_attrs(attr_args).expect("fail to parse attr");
    let type_name_raw = opts.name.as_str();
    let type_version = opts.version;
    let type_name = type_name_raw.replace("-", "_");
    let type_static_ident = Ident::new(&type_name.to_ascii_uppercase(), Span::call_site());
    let data_name_ident = if let syn::Type::Path(syn::TypePath {
        path: syn::Path { segments, .. },
        qself: _,
    }) = item_impl.self_ty.as_ref()
    {
        segments.clone().first().unwrap().ident.clone()
    } else {
        panic!("expected impl single type");
    };
    let method_names: HashSet<String> = item_impl
        .items
        .iter()
        .map(|impl_item| -> Option<String> {
            if let syn::ImplItem::Method(syn::ImplItemMethod {
                sig: syn::Signature { ident, .. },
                ..
            }) = impl_item
            {
                Some(ident.to_string())
            } else {
                None
            }
        })
        .filter_map(|v| v)
        .collect();
    let have_method = |name: &str| method_names.contains(name);

    let type_name_rdb_load = Ident::new(&format!("{}_rdb_load", &type_name), Span::call_site());
    let (rdb_load_fn, rdb_load_field) = if have_method("rdb_load") {
        (
            quote! {
                extern "C" fn #type_name_rdb_load(rdb: *mut iredismodule::raw::RedisModuleIO, encver: std::os::raw::c_int) -> *mut std::os::raw::c_void {
                    use iredismodule::FromPtr;
                    let mut io = iredismodule::io::IO::from_ptr(rdb);
                    let ret = #data_name_ident::rdb_load(&mut io, encver as u32);
                    if ret.is_none() {
                        return  0 as *mut std::os::raw::c_void;
                    }
                    Box::into_raw(ret.unwrap()) as *mut std::os::raw::c_void
                }
            },
            quote! {
                Some(#type_name_rdb_load)
            },
        )
    } else {
        (proc_macro2::TokenStream::new(), quote! { None })
    };

    let type_name_rdb_save = Ident::new(&format!("{}_rdb_save", &type_name), Span::call_site());
    let (rdb_save_fn, rdb_save_field) = if have_method("rdb_save") {
        (
            quote! {
                unsafe extern "C" fn #type_name_rdb_save(rdb: *mut iredismodule::raw::RedisModuleIO, value: *mut std::os::raw::c_void) {
                    use iredismodule::FromPtr;
                    let mut io = iredismodule::io::IO::from_ptr(rdb);
                    let hto = &*(value as *mut #data_name_ident);
                    hto.rdb_save(&mut io)
                }
            },
            quote! {
                Some(#type_name_rdb_save)
            },
        )
    } else {
        (proc_macro2::TokenStream::new(), quote! { None })
    };

    let type_name_aof_rewrite =
        Ident::new(&format!("{}_aof_rewrite", &type_name), Span::call_site());
    let (aof_rewrite_fn, aof_rewrite_field) = if have_method("aof_rewrite") {
        (
            quote! {
                unsafe extern "C" fn #type_name_aof_rewrite(aof: *mut iredismodule::raw::RedisModuleIO, key: *mut iredismodule::raw::RedisModuleString, value: *mut std::os::raw::c_void) {
                    use iredismodule::FromPtr;
                    let mut io = iredismodule::io::IO::from_ptr(aof);
                    let hto = &*(value as *mut #data_name_ident);
                    let key = iredismodule::string::RStr::from_ptr(key);
                    hto.aof_rewrite(&mut io, &key)
                }
            },
            quote! {
                Some(#type_name_aof_rewrite)
            },
        )
    } else {
        (proc_macro2::TokenStream::new(), quote! { None })
    };

    let type_name_mem_usage = Ident::new(&format!("{}_mem_usage", &type_name), Span::call_site());
    let (mem_usage_fn, mem_usage_field) = if have_method("mem_usage") {
        (
            quote! {
                unsafe extern "C" fn #type_name_mem_usage(value: *const std::os::raw::c_void) -> usize {
                    let hto = &*(value as *const #data_name_ident);
                    hto.mem_usage()
                }
            },
            quote! {
                Some(#type_name_mem_usage)
            },
        )
    } else {
        (proc_macro2::TokenStream::new(), quote! { None })
    };

    let type_name_digest = Ident::new(&format!("{}_digest", &type_name), Span::call_site());
    let (digest_fn, digest_field) = if have_method("digest") {
        (
            quote! {
                unsafe extern "C" fn #type_name_digest(md: *mut iredismodule::raw::RedisModuleDigest, value: *mut std::os::raw::c_void) {
                    use iredismodule::FromPtr;
                    let mut digest = iredismodule::io::Digest::from_ptr(md);
                    let hto = &*(value as *const #data_name_ident);
                    hto.digest(&mut digest)
                }
            },
            quote! {
                Some(#type_name_digest)
            },
        )
    } else {
        (proc_macro2::TokenStream::new(), quote! { None })
    };

    let type_name_free = Ident::new(&format!("{}_free", &type_name), Span::call_site());
    let (free_fn, free_field) = if have_method("free") {
        (
            quote! {
                unsafe extern "C" fn #type_name_free(value: *mut std::os::raw::c_void) {
                    #data_name_ident::free(Box::from_raw(value as *mut #data_name_ident))
                }
            },
            quote! {
                Some(#type_name_free)
            },
        )
    } else {
        (proc_macro2::TokenStream::new(), quote! { None })
    };

    let type_name_aux_load = Ident::new(&format!("{}_aux_load", &type_name), Span::call_site());
    let (aux_load_fn, aux_load_field) = if have_method("aux_load") {
        (
            quote! {
                unsafe extern "C" fn #type_name_aux_load(rdb: *mut iredismodule::raw::RedisModuleIO, encver: std::os::raw::c_int, when: std::os::raw::c_int) {
                    use iredismodule::FromPtr;
                    let mut io = iredismodule::io::IO::from_ptr(rdb);
                    #data_name_ident::aux_load(&mut io, encver as u32, when as u32)
                }
            },
            quote! {
                Some(#type_name_aux_load)
            },
        )
    } else {
        (proc_macro2::TokenStream::new(), quote! { None })
    };

    let type_name_aux_save = Ident::new(&format!("{}_aux_save", &type_name), Span::call_site());
    let (aux_save_fn, aux_save_field) = if have_method("aux_save") {
        (
            quote! {
                unsafe extern "C" fn #type_name_aux_save(rdb: *mut iredismodule::raw::RedisModuleIO, when: std::os::raw::c_int) {
                    use iredismodule::FromPtr;
                    let mut io = iredismodule::io::IO::from_ptr(rdb);
                    #data_name_ident::aux_save(&mut io, when as u32)
                }
            },
            quote! {
                Some(#type_name_aux_save)
            },
        )
    } else {
        (proc_macro2::TokenStream::new(), quote! { None })
    };

    let aux_save_triggers = {
        quote! { #data_name_ident::AUX_SAVE_TRIGGERS as i32 }
    };

    let type_static = quote! {
        pub static #type_static_ident: iredismodule::rtype::RType<#data_name_ident> = iredismodule::rtype::RType::new(
            #type_name_raw,
            #type_version,
            iredismodule::raw::RedisModuleTypeMethods {
                version: iredismodule::raw::REDISMODULE_TYPE_METHOD_VERSION as u64,
                rdb_load: #rdb_load_field,
                rdb_save: #rdb_save_field,
                aof_rewrite: #aof_rewrite_field,
                mem_usage:  #mem_usage_field,
                free: #free_field,
                digest:#digest_field,
                aux_load: #aux_load_field,
                aux_save: #aux_save_field,
                aux_save_triggers: #aux_save_triggers,
            },
        );
    };

    let output = quote! {
        #type_static
        #rdb_load_fn
        #rdb_save_fn
        #aof_rewrite_fn
        #mem_usage_fn
        #digest_fn
        #free_fn
        #aux_load_fn
        #aux_save_fn
        #item_impl
    };
    TokenStream::from(output)
}

fn rtypedef_parse_attrs(args: syn::AttributeArgs) -> Option<TypeDefAttributeOpts> {
    let lits: Vec<&Lit> = args
        .iter()
        .filter(|v| {
            if let syn::NestedMeta::Lit(_) = v {
                true
            } else {
                false
            }
        })
        .map(|v| match v {
            syn::NestedMeta::Lit(i) => i,
            _ => unreachable!(),
        })
        .collect();
    match lits.as_slice() {
        [Lit::Str(name)] => Some(TypeDefAttributeOpts {
            name: name.value(),
            version: 0,
        }),
        [Lit::Str(name), Lit::Int(version)] => Some(TypeDefAttributeOpts {
            name: name.value(),
            version: version.base10_parse().unwrap(),
        }),
        _ => None,
    }
}
