#![feature(proc_macro_diagnostic)]

use darling::FromMeta;
use proc_macro::TokenStream;
use syn::parse_macro_input;

mod service;

use service::Args;

#[proc_macro_attribute]
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as syn::AttributeArgs);
    let items = parse_macro_input!(item as service::Items);
    let args = match Args::from_list(&args) {
        Ok(v) => {
            if let Err(err) = v.verify() {
                return err.to_compile_error().into();
            }
            v
        }
        Err(err) => return err.write_errors().into(),
    };

    service::generate(args, items).into()
}
