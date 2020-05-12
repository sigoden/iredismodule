use proc_macro::{TokenStream};
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Ident, Lit};

#[derive(Debug)]
struct CmdAttributeOpts {
    name: String,
    flags: String,
    first_key: usize,
    last_key: usize,
    key_step: usize,
}

pub fn rcmd(attr: TokenStream, input: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(input as syn::ItemFn);
    let attr_args =  parse_macro_input!(attr as syn::AttributeArgs);
    let opts =  rcmd_parse_attrs(attr_args).expect("fail to parse attr");
    let fn_name = item_fn.sig.ident.clone();
    let vis = item_fn.vis.clone();

    let c_fn_name = Ident::new(&format!("{}_c", &fn_name), Span::call_site());
    let cmd_fn_name = Ident::new(&format!("{}_cmd", &fn_name), Span::call_site());
    let name = opts.name.clone();
    let flags = opts.flags.clone();
    let first_key = opts.first_key;
    let last_key = opts.last_key;
    let key_step = opts.key_step;
    let raw_fn = quote! {
        #vis extern "C" fn #c_fn_name(
            ctx: *mut redismodule::raw::RedisModuleCtx,
            argv: *mut *mut redismodule::raw::RedisModuleString,
            argc: std::os::raw::c_int
        ) -> std::os::raw::c_int {
            let mut context = redismodule::Context::from_ptr(ctx);
            let response = #fn_name(&mut context, redismodule::parse_args(argv, argc));
            context.reply(response) as std::os::raw::c_int
        }
    };
    let create_fn = quote! {
        #vis fn #cmd_fn_name(ctx: &mut redismodule::Context) -> Result<(), redismodule::Error> {
            ctx.create_cmd(#name, #c_fn_name, #flags, #first_key, #last_key, #key_step)
        }
    };
    let output = quote! {
        #item_fn
        #raw_fn
        #create_fn
    };
    TokenStream::from(output)
}

fn rcmd_parse_attrs(args: syn::AttributeArgs) -> Option<CmdAttributeOpts> {
    let lits: Vec<&Lit> = args.iter()
        .filter(|v| if let syn::NestedMeta::Lit(_) = v { true } else { false })
        .map(|v| match v {
            syn::NestedMeta::Lit(i) => i,
            _ => unreachable!(),
        })
        .collect();

    match lits.as_slice() {
        [Lit::Str(name)] => {
            Some(CmdAttributeOpts {
                name: name.value(),
                flags: "".to_owned(),
                first_key: 0,
                last_key: 0,
                key_step: 0,
            })
        },
        [Lit::Str(name), Lit::Str(flags)] => {
            Some(CmdAttributeOpts {
                name: name.value(),
                flags: flags.value(),
                first_key: 0,
                last_key: 0,
                key_step: 0,
            })
        },
        [Lit::Str(name), Lit::Str(flags), 
            Lit::Int(first_key), Lit::Int(last_key), Lit::Int(key_step)
            ] => {
            Some(CmdAttributeOpts {
                name: name.value(),
                flags: flags.value(),
                first_key: first_key.base10_parse().unwrap(),
                last_key: last_key.base10_parse().unwrap(),
                key_step: key_step.base10_parse().unwrap(),
            })
        },
        _ => None,
    }
}

