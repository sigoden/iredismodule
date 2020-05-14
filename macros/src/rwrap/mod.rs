use proc_macro::TokenStream;
use syn::{parse_macro_input, Lit};

mod call;
mod cluster_msg;
mod free;

pub fn rwrap(attr: TokenStream, input: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(input as syn::ItemFn);
    let attr_args = parse_macro_input!(attr as syn::AttributeArgs);
    let kind = rwrap_parse_attrs(attr_args).expect("fail to parse attr");
    match kind.as_str() {
        "cluster_msg" => cluster_msg::cluster_msg(item_fn),
        "free" => free::free(item_fn),
        "call" => call::call(item_fn),
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
