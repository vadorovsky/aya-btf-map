use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemStatic};

mod expand;

use expand::{Args, BtfMap};

#[proc_macro_attribute]
pub fn btf_map(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attrs as Args);
    let item = parse_macro_input!(item as ItemStatic);

    BtfMap::from_syn(args, item)
        .and_then(|u| u.expand())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
