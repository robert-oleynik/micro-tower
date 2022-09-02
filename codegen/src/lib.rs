#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use service::Service;
use syn::parse_macro_input;

mod service;
mod utils;

#[proc_macro_attribute]
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as syn::AttributeArgs);
    let items = parse_macro_input!(item as service::Items);
    let service = Service::new(args, items);

    // items.generate()
    quote::quote!().into()
}
