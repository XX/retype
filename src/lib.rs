extern crate proc_macro;

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
static CONFIG_ENV_VAR: &'static str = "RETYPE_CONFIG";

lazy_static! {
    static ref CONFIG: TomlValue = {

        let first = File::open(CONFIG_FILE).ok().map(|mut file| {
            let mut content = String::new();
            file.read_to_string(&mut content)
                .expect(&format!("Can't read content from `{}` file", CONFIG_FILE));

            content.parse::<TomlValue>()
                .expect(&format!("Can't parse content of `{}` file as Toml", CONFIG_FILE))
        });

        let second = env::var(CONFIG_ENV_VAR).ok().map(|path| {
            let mut file = File::open(&path)
                .expect(&format!("Can't find `{}` file", path));

            let mut content = String::new();
            file.read_to_string(&mut content)
                .expect(&format!("Can't read content from `{}` file", path));

            content.parse::<TomlValue>()
                .expect(&format!("Can't parse content of `{}` file as Toml", path))
        });

        match (first, second) {
            (Some(mut first), Some(second)) => {
                merge_toml(&mut first, second);
                first
            },
            (Some(first), None) => first,
            (None, Some(second)) => second,
            (None, None) => panic!("Can't find `{}` file or `{}` env variable", CONFIG_FILE, CONFIG_ENV_VAR)
        }
    };
}

pub(crate) fn merge_toml(first: &mut TomlValue, second: TomlValue) {
    if first.is_table() && second.is_table() {
        if let TomlValue::Table(table) = second {
            for (key, value) in table.into_iter() {
                first.as_table_mut()
                    .map(|table| {
                        if let Some(first_value) = table.get_mut(&key) {
                            merge_toml(first_value, value);
                        } else {
                            table.insert(key, value);
                        }
                    });
            }
        }
    } else {
        *first = second;
    }
}

#[proc_macro_attribute]
pub fn retype(attr: TokenStream, item: TokenStream) -> TokenStream {
    let type_item = syn::parse::<ItemType>(item)
        .expect("Retype attribute can only be applied to type item");
    let ident = syn::parse::<Ident>(attr)
        .expect("Retype attribute must use ident param")
        .to_string();

    let attrs = CONFIG.as_table()
        .expect("Retype config must contain a table");

    let mut new_type_item = type_item.clone();
    let mut items = Vec::new();

    for (attr, value) in attrs.iter() {
        let attr = syn::parse::<Meta>(
            TokenStream::from_str(attr.as_str())
                .expect(&format!("Can't parse `{}` as token stream", attr))
            ).expect(&format!("Can't parse `{}` as attribute meta", attr));

        match value.as_table().and_then(|table| table.get(&ident)) {
            Some(value) => {
                let replacement = value.as_str()
                    .expect(&format!("Can't parse replacement value `{:?}` as string", value));
                let new_type = syn::parse::<Type>(
                    TokenStream::from_str(replacement)
                        .expect(&format!("Can't parse replacement value `{}` as token stream", replacement))
                ).expect(&format!("Can't parse replacement value `{}` as type", replacement));

                new_type_item.ty = Box::new(new_type);
                items.push(quote! { #[#attr] #new_type_item });
            },
            None => {
                items.push(quote! { #[#attr] #type_item });
            },
        }
    }

    if items.is_empty() {
        items.push(quote! { #type_item });
    }

    quote!(#(#items)*).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge() {
        let cases = [
            ("key = 2", "key = 42", "key = 42"),
            ("key = 2\narr = [1]", "arr = [1, 2]", "key = 2\narr = [1, 2]"),
            ("[test]\na = 1\nb = 2", "", "[test]\na = 1\nb = 2"),
            ("[test]\na = 1\nb = 2", "[test]", "[test]\na = 1\nb = 2"),
            ("[test]\na = 1\nb = 2", "[test]\na = 3", "[test]\na = 3\nb = 2"),
            ("[test]\na = 1", "[test]\na = 3\nb = 2", "[test]\na = 3\nb = 2"),
            ("[test]\nb = 1", "[test]\na = 3\nb = 2", "[test]\na = 3\nb = 2"),
            ("[test]\na = 1\n[other]\nb = 2", "[test]\na = 3\nb = 1", "[test]\na = 3\nb = 1\n[other]\nb = 2"),
        ];
        for (first, second, merged) in cases.iter() {
            let mut first = first.parse::<TomlValue>().unwrap();
            let second = second.parse::<TomlValue>().unwrap();
            let merged = merged.parse::<TomlValue>().unwrap();

            merge_toml(&mut first, second);
            assert_eq!(merged, first);
        }
    }
}