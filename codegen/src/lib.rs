#![feature(proc_macro_diagnostic)]

use proc_macro::{Diagnostic, Level, TokenStream};
use service::Service;
use syn::{parse_macro_input, spanned::Spanned};

mod service;
mod util;

#[proc_macro_attribute]
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as syn::AttributeArgs);
    let items = parse_macro_input!(item as service::Items);
    let service = match Service::new(args, items) {
        Ok(service) => service,
        Err(e) => return e.into_compile_error().into(),
    };

    let crate_mod = &service.args.crate_path;

    let pub_token = &service.pub_token;
    let name = &service.name;
    let block = service.code_block.clone();
    let request = &service.request;
    let request_arg = &service.request_arg;
    let response = &service.response;
    let output = &service.output;
    let ret = match service.response_result {
        true => quote::quote!(Ok(result?)),
        false => quote::quote!(Ok(result)),
    };
    let deps0 = service.deps();
    let depc =
        (0..service.deps().count()).map(|l| syn::LitInt::new(&format!("{l}"), output.span()));
    let dep_args = service.service_dependencies.iter();

    let name_lit = syn::LitStr::new(&name.to_string(), name.span());

    quote::quote!(
        #[allow(non_camel_case_types)]
        #[derive(::std::clone::Clone)]
        #pub_token struct #name(#( #deps0 ),*);

        impl #name {
            async fn handle(#request_arg, #( #dep_args ),*) #output #block
        }

        impl #crate_mod::util::Buildable for #name {
            type Builder = ();
            
            fn builder() -> Self::Builder {}
        }

        impl #crate_mod::export::tower::Service<#request> for #name {
            type Response = #response;
            type Error = #crate_mod::export::tower::BoxError;
            type Future = ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

            fn poll_ready(&mut self, cx: &mut ::std::task::Context<'_>) -> ::std::task::Poll<Result<(), Self::Error>> {
                ::std::task::Poll::Ready(Ok(()))
            }

            fn call(&mut self, request: #request) -> Self::Future {
                use #crate_mod::export::tracing::Instrument;

                let this = self.clone();
                let fut = async move {
                    #crate_mod::export::tracing::trace!("called");
                    let result = Self::handle(request, #( this.#depc ),*).await;
                    #ret
                };
                let fut = fut.instrument(#crate_mod::export::tracing::info_span!(#name_lit));
                ::std::boxed::Box::pin(fut)
            }
        }
    )
    .into()
}
