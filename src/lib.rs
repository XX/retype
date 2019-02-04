extern crate proc_macro;
extern crate syn;
extern crate quote;
extern crate toml;
extern crate lazy_static;

use std::env;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use proc_macro::TokenStream;
use syn::{ItemType, Ident, Type, Meta};
use quote::quote;
use lazy_static::lazy_static;
use toml::Value as TomlValue;

static CONFIG_FILE: &'static str = "Retype.toml";

lazy_static! {
    static ref CONFIG: TomlValue = {
        let path = env::var("RETYPE_CONFIG").unwrap_or_else(|_| CONFIG_FILE.to_string());

        let mut file = File::open(&path)
            .expect(&format!("Can't find `{}` file", path));

        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect(&format!("Can't read content from `{}` file", path));

        content.parse::<TomlValue>()
            .expect(&format!("Can't parse content of `{}` file as Toml", path))
    };
}

#[proc_macro_attribute]
pub fn retype(attr: TokenStream, item: TokenStream) -> TokenStream {
    let type_item = syn::parse::<ItemType>(item)
        .expect("Replace attribute can only be applied to type item");
    let ident = syn::parse::<Ident>(attr)
        .expect("Replace attribute must use ident param")
        .to_string();

    let attrs = CONFIG.as_table()
        .expect("Replace config must contain a table");

    let mut new_type_item = type_item.clone();
    let mut items = Vec::new();

    for (attr, value) in attrs.iter() {
        let attr = syn::parse::<Meta>(
            TokenStream::from_str(attr.as_str())
                .expect(&format!("Can't parse `{}` as token stream", attr))
            ).expect(&format!("Can't parse `{}` as attribute meta", attr));

        if let Some(value) = value.as_table()
            .and_then(|table| table.get(&ident))
        {
            let replacement = value.as_str()
                .expect(&format!("Can't parse replacement value `{:?}` as string", value));
            let new_type = syn::parse::<Type>(
                TokenStream::from_str(replacement)
                    .expect(&format!("Can't parse replacement value `{}` as token stream", replacement))
                ).expect(&format!("Can't parse replacement value `{}` as type", replacement));

            new_type_item.ty = Box::new(new_type);
            items.push(quote! { #[#attr] #new_type_item });
        }
    }

    if items.is_empty() {
        items.push(quote! { #type_item });
    }

    quote!(#(#items)*).into()
}
