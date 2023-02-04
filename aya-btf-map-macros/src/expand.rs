use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::{Pair, Punctuated},
    token::Eq,
    Error, Ident, ItemStatic, LitStr, Result, Token,
};

pub struct NameValue {
    name: Ident,
    _eq: Eq,
    value: LitStr,
}

impl Parse for NameValue {
    fn parse(input: ParseStream) -> Result<NameValue> {
        let name = input.parse()?;
        let _eq = input.parse()?;
        let value = input.parse()?;

        Ok(NameValue { name, _eq, value })
    }
}

pub struct Args {
    args: Vec<NameValue>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Args> {
        let args = Punctuated::<NameValue, Token![,]>::parse_terminated(input)?
            .into_pairs()
            .map(|pair| match pair {
                Pair::Punctuated(name_val, _) => name_val,
                Pair::End(name_val) => name_val,
            })
            .collect();
        Ok(Args { args })
    }
}

fn pop_arg(args: &mut Args, name: &str) -> Option<String> {
    match args.args.iter().position(|arg| arg.name == name) {
        Some(index) => Some(args.args.remove(index).value.value()),
        None => None,
    }
}

fn err_on_unknown_args(args: &Args) -> Result<()> {
    if let Some(arg) = args.args.get(0) {
        return Err(Error::new_spanned(&arg.name, "invalid argument"));
    }

    Ok(())
}

fn name_arg(args: &mut Args) -> Result<Option<String>> {
    let name = pop_arg(args, "name");
    err_on_unknown_args(args)?;

    Ok(name)
}

pub struct BtfMap {
    item: ItemStatic,
    name: String,
}

impl BtfMap {
    pub fn from_syn(mut args: Args, item: ItemStatic) -> Result<BtfMap> {
        let name = name_arg(&mut args)?.unwrap_or_else(|| item.ident.to_string());
        Ok(BtfMap { item, name })
    }

    pub fn expand(&self) -> Result<TokenStream> {
        let name = &self.name;
        let struct_name = Ident::new(&format!("_anon_{}", name), Span::call_site());

        let name_str = LitStr::new(&name, Span::call_site());

        // TODO: use proper values
        let map_type = 1;
        let max_entries = 1024;
        let map_flags = 0;

        Ok(quote! {
            pub struct #struct_name {
                pub r#type: *const [i32; #map_type as usize],
                // pub key: *const #key_type,
                // pub value: *const #value_type,
                pub max_entries: *const [i32; #max_entries as usize],
                pub map_flags: *const [i32; #map_flags as usize],
            }

            #[link_section = ".maps"]
            #[export_name = #name_str]
            pub static mut #name: #struct_name = #struct_name {
                r#type: &[0i32; #map_type as usize] as *const [i32; #map_type as usize],
                key: ::core::ptr::null(),
                value: ::core::ptr::null(),
                max_entries: &[0i32; #max_entries as usize] as *const [i32; #max_entries as usize],
                map_flags: &[0i32; #map_flags as usize] as *const [i32; #map_flags as usize],
            };
        })
    }
}
