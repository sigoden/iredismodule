extern crate proc_macro;

use proc_macro::{TokenStream};

macro_rules! use_macro {
    ($name:ident) => {
        mod $name;
        #[proc_macro_attribute]
        pub fn $name(attr: TokenStream, input: TokenStream) -> TokenStream {
            $name::$name(attr, input)
        }
    };
}

use_macro!(rcall);
use_macro!(rcmd);
use_macro!(rtypedef);
use_macro!(rwrap);