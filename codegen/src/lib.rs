#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod service;

#[proc_macro_attribute]
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = parse_macro_input!(attr as service::Args);
    let items = parse_macro_input!(item as service::Items);

    items.generate()
}
