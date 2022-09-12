#![feature(proc_macro_diagnostic)]

use darling::FromMeta;
use proc_macro::TokenStream;
use service::Service;
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
