use proc_macro2::TokenStream;
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
        let item = &self.item;
        Ok(quote! {
            #[link_section = ".maps"]
            #[export_name = #name]
            #item
        })
    }
}
