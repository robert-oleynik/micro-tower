#![feature(proc_macro_diagnostic)]

use darling::FromMeta;
use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs};

mod service;
mod util;

#[proc_macro_attribute]
pub fn service(args: TokenStream, items: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let args = match service::args::Args::from_list(&args) {
        Ok(args) => args,
        Err(err) => return TokenStream::from(err.write_errors()),
    };
    let decl = parse_macro_input!(items as service::decl::Declaration);
    service::generate(args, decl).into()
}
