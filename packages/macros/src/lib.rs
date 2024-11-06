#![feature(proc_macro_quote)]

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Item};

mod kvp;
mod secret;

#[proc_macro_derive(KeyValueProvider, attributes(kvp))]
pub fn key_value_provider_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    kvp::expand_struct(ast)
}

#[proc_macro_attribute]
pub fn secret(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);

    match item {
        Item::Struct(s) => secret::expand_struct(s),
        _ => panic!("can only be used on structs"),
    }
}