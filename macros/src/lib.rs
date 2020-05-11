extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, Ident};
use std::collections::HashSet;
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
pub fn rcommand(attr: TokenStream, input: TokenStream) -> TokenStream {
    let cmd_fn = parse_macro_input!(input as syn::ItemFn);
    let attr_args =  parse_macro_input!(attr as AttributeArgs);
    let opts: CmdAttributeOpts = CmdAttributeOpts::from_list(&attr_args).unwrap();
    let fn_name = cmd_fn.sig.ident.clone();
    let vis = cmd_fn.vis.clone();

    let raw_fn_name = Ident::new(&format!("{}_c", &fn_name), Span::call_site());
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
pub fn rcall(_: TokenStream, input: TokenStream) -> TokenStream {
    let cmd_fn = parse_macro_input!(input as syn::ItemFn);
    let fn_name = cmd_fn.sig.ident.clone();
    let raw_fn_name = Ident::new(&format!("{}_c", &fn_name), Span::call_site());
    let vis = cmd_fn.vis.clone();
    let output = quote! {
        #vis extern "C" fn #raw_fn_name(
            ctx: *mut redismodule::raw::RedisModuleCtx,
            argv: *mut *mut redismodule::raw::RedisModuleString,
            argc: std::os::raw::c_int,
        ) -> std::os::raw::c_int {
            let args = redismodule::parse_args(argv, argc);
            let mut context = redismodule::Context::from_ptr(ctx);
            let result = #fn_name(&mut context, args);
            if result.is_err() {
                redismodule::raw::REDISMODULE_ERR as std::os::raw::c_int
            } else {
                redismodule::raw::REDISMODULE_OK as std::os::raw::c_int
            }
        }
        #cmd_fn
    };
    TokenStream::from(output)
}

#[derive(Debug, FromMeta)]
struct TypeDefAttributeOpts {
    name: String,
    version: i32,
}

#[proc_macro_attribute]
pub fn rtypedef(attr: TokenStream, input: TokenStream) -> TokenStream {
    let item_impl = parse_macro_input!(input as syn::ItemImpl);
    let attr_args =  parse_macro_input!(attr as AttributeArgs);
    let opts: TypeDefAttributeOpts = TypeDefAttributeOpts::from_list(&attr_args).unwrap();
    let type_name_raw = opts.name.as_str();
    let type_version = opts.version;
    let type_name = type_name_raw.replace("-", "_");
    let type_static_ident = Ident::new(&type_name.to_ascii_uppercase(), Span::call_site());
    let data_name_ident = 
        if let syn::Type::Path(
                syn::TypePath {
                    path: syn::Path { segments, .. },
                    qself: _ 
                }
            ) = item_impl.self_ty.as_ref() 
        {
            segments.clone().first().unwrap().ident.clone()
        } else {
            panic!("expected impl single type");
        };
    let method_names: HashSet<String> = item_impl
        .items
        .iter()
        .map(|impl_item| -> Option<String> {
            if let syn::ImplItem::Method(
                syn::ImplItemMethod {
                    sig: syn::Signature {
                        ident,
                        ..
                    },
                    ..
                }
            ) = impl_item  {
                Some(ident.to_string())
            } else {
                None
            }
        })
        .filter_map(|v| v)
        .collect();
    let have_method = |name: &str| method_names.contains(name);

    let type_name_rdb_load = Ident::new(&format!("{}_rdb_load", &type_name), Span::call_site());
    let (rdb_load_fn, rdb_load_field) = if have_method("rdb_load")  {
        (
            quote! {
                extern "C" fn #type_name_rdb_load(rdb: *mut redismodule::raw::RedisModuleIO, encver: c_int) -> *mut std::os::raw::c_void {
                    let mut io = redismodule::IO::from_ptr(rdb);
                    let ret = #data_name_ident::rdb_load(&mut io, encver as u32);
                    if ret.is_none() {
                        return  0 as *mut std::os::raw::c_void;
                    }
                    Box::into_raw(ret.unwrap()) as *mut std::os::raw::c_void
                }
            },
            quote! {
                Some(#type_name_rdb_load)
            }
        )
    } else {
        (proc_macro2::TokenStream::new(), quote! { None })
    };

    let type_name_rdb_save = Ident::new(&format!("{}_rdb_save", &type_name), Span::call_site());
    let (rdb_save_fn, rdb_save_field) = if have_method("rdb_save")  {
        (
            quote! {
                unsafe extern "C" fn #type_name_rdb_save(rdb: *mut redismodule::raw::RedisModuleIO, value: *mut std::os::raw::c_void) {
                    let mut io = redismodule::IO::from_ptr(rdb);
                    let hto = &*(value as *mut #data_name_ident);
                    hto.rdb_save(&mut io)
                }
            },
            quote! {
                Some(#type_name_rdb_save)
            }
        )
    } else {
        (proc_macro2::TokenStream::new(), quote! { None })
    };

    let type_name_aof_rewrite = Ident::new(&format!("{}_aof_rewrite", &type_name), Span::call_site());
    let (aof_rewrite_fn, aof_rewrite_field) = if have_method("aof_rewrite")  {
        (
            quote! {
                unsafe extern "C" fn #type_name_aof_rewrite(aof: *mut redismodule::raw::RedisModuleIO, key: *mut redismodule::raw::RedisModuleString, value: *mut std::os::raw::c_void) {
                    let mut io = redismodule::IO::from_ptr(aof);
                    let hto = &*(value as *mut #data_name_ident);
                    let key = redismodule::RStr::from_ptr(key);
                    hto.aof_rewrite(&mut io, &key)
                }
            },
            quote! {
                Some(#type_name_aof_rewrite)
            }
        )
    } else {
        (proc_macro2::TokenStream::new(), quote! { None })
    };

    let type_name_mem_usage = Ident::new(&format!("{}_mem_usage", &type_name), Span::call_site());
    let (mem_usage_fn, mem_usage_field) = if have_method("mem_usage")  {
        (
            quote! {
                unsafe extern "C" fn #type_name_mem_usage(value: *const std::os::raw::c_void) -> usize {
                    let hto = &*(value as *const #data_name_ident);
                    hto.mem_usage()
                }
            },
            quote! {
                Some(#type_name_mem_usage)
            }
        )
    } else {
        (proc_macro2::TokenStream::new(), quote! { None })
    };

    let type_name_digest = Ident::new(&format!("{}_digest", &type_name), Span::call_site());
    let (digest_fn, digest_field) = if have_method("digest")  {
        (
            quote! {
                unsafe extern "C" fn #type_name_digest(md: *mut redismodule::raw::RedisModuleDigest, value: *mut std::os::raw::c_void) {
                    let mut digest = redismodule::Digest::from_ptr(md);
                    let hto = &*(value as *const #data_name_ident);
                    hto.digest(&mut digest)
                }
            },
            quote! {
                Some(#type_name_digest)
            }
        )
    } else {
        (proc_macro2::TokenStream::new(), quote! { None })
    };

    let type_name_free = Ident::new(&format!("{}_free", &type_name), Span::call_site());
    let (free_fn, free_field) = if have_method("free")  {
        (
            quote! {
                unsafe extern "C" fn #type_name_free(value: *mut std::os::raw::c_void) {
                    #data_name_ident::free(Box::from_raw(value as *const #data_name_ident))
                }
            },
            quote! {
                Some(#type_name_free)
            }
        )
    } else {
        (proc_macro2::TokenStream::new(), quote! { None })
    };

    let type_name_aux_load = Ident::new(&format!("{}_aux_load", &type_name), Span::call_site());
    let (aux_load_fn, aux_load_field) = if have_method("aux_load")  {
        (
            quote! {
                unsafe extern "C" fn #type_name_aux_load(rdb: *mut redismodule::raw::RedisModuleIO, encver: c_int, when: c_int) {
                    let mut io = redismodule::IO::from_ptr(rdb);
                    #data_name_ident::aux_load(&mut io, encver as u32, when as u32)
                }
            },
            quote! {
                Some(#type_name_aux_load)
            }
        )
    } else {
        (proc_macro2::TokenStream::new(), quote! { None })
    };

    let type_name_aux_save = Ident::new(&format!("{}_aux_save", &type_name), Span::call_site());
    let (aux_save_fn, aux_save_field) = if have_method("aux_save")  {
        (
            quote! {
                unsafe extern "C" fn #type_name_aux_save(rdb: *mut redismodule::raw::RedisModuleIO, when: c_int) {
                    let mut io = redismodule::IO::from_ptr(rdb);
                    #data_name_ident::aux_save(&mut io, when as u32)
                }
            },
            quote! {
                Some(#type_name_aux_save)
            }
        )
    } else {
        (proc_macro2::TokenStream::new(), quote! { None })
    };


    let aux_save_triggers = if have_method("aux_save_triggers")  {
        quote! { #data_name_ident::aux_save_triggers() }
    } else {
        quote! { 0 }
    };

    let type_static =  quote! {
        pub static #type_static_ident: redismodule::RType = redismodule::RType::new(
            #type_name_raw,
            #type_version,
            redismodule::raw::RedisModuleTypeMethods {
                version: redismodule::raw::REDISMODULE_TYPE_METHOD_VERSION as u64,
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
