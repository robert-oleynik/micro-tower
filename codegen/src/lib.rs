#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use service::Service;
use syn::parse_macro_input;

mod service;
mod util;

#[proc_macro_attribute]
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as syn::AttributeArgs);
    let items = parse_macro_input!(item as service::Items);
    let service = Service::new(args, items);

    let struct_decl = service.generate_struct();
    let handle_impl = service.generate_handle();
    let buildable_impl = service.generate_buildable_impl();
    let service_impl = service.generate_service_impl();

    quote::quote!(
        #struct_decl
        #handle_impl
        #buildable_impl
        #service_impl
    )
    .into()
}
