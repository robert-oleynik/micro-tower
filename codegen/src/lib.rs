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
    let deps1 = service.deps();
    let deps2 = service.deps();
    let dep_args = service.service_dependencies.iter();

    quote::quote!(
        #[allow(non_camel_case_types)]
        #[derive(::std::clone::Clone)]
        #pub_token struct #name(#( #deps0 ),*);

        impl #name {
            async fn handle(#request_arg, #( #dep_args ),*) #output #block
        }

        impl #crate_mod::utils::Named for #name {}
        impl #crate_mod::service::Create for #name {
            type Service = ::micro_tower::tower::util::BoxCloneService<#request, #response, #crate_mod::tower::BoxError>;

            fn deps() -> &'static [::std::any::TypeId] {
                &[ #( #deps1::name() ),* ]
            }

            fn create(registry: & #crate_mod::utils::TypeRegistry) -> Self::Service {
                #crate_mod::tower::ServiceBuilder::new()
                    .boxed_clone()
                    .service(#name (
                        #( registry.get(&#deps2::name()).unwrap()),*
                    ))
            }
        }

        impl #crate_mod::tower::Service<#request> for #name {
            type Response = #response;
            type Error = #crate_mod::tower::BoxError;
            type Future = ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

            fn poll_ready(&mut self, cx: &mut ::std::task::Context<'_>) -> ::std::task::Poll<Result<(), Self::Error>> {
                ::std::task::Poll::Ready(Ok(()))
            }

            fn call(&mut self, request: #request) -> Self::Future {
                ::std::boxed::Box::pin(async move {
                    let result = Self::handle(request).await;
                    #ret
                })
            }
        }
    )
    .into()
}
