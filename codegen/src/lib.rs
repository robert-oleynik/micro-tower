#![feature(proc_macro_diagnostic)]

use proc_macro::{Diagnostic, Level, TokenStream};
use service::Service;
use syn::{parse_macro_input, spanned::Spanned};

mod manifest;
mod service;
mod utils;

#[proc_macro]
pub fn manifest(item: TokenStream) -> TokenStream {
    let manifest = parse_macro_input!(item as manifest::Manifest);

    let crate_mod: syn::Path = syn::parse_str("::micro_tower").unwrap();

    let name = &manifest.name;
    let pub_token = &manifest.pub_token;

    if manifest.services.is_empty() {
        Diagnostic::spanned(
            vec![manifest.brackets.span.unwrap()],
            Level::Warning,
            "No services specified",
        );
    }

    let service_decl = manifest.services.iter().map(|service| {
        let name = &service.name;
        quote::quote!( #name: <#name as #crate_mod::service::Create>::Service )
    });

    let service_create = manifest.services.iter().map(|service| {
        let name = &service.name;
        let service_lit = syn::LitStr::new(&name.to_string(), name.span());
        quote::quote!(
        let service_name = <#name as #crate_mod::utils::Named>::name();
        if !registry.contains_key(&service_name) {
            empty = false;
            let service = ::std::stringify!(#name);
            if <#name as #crate_mod::service::Create>::deps().iter().all(|dep| registry.contains_key(dep)) {
                let span = #crate_mod::tracing::info_span!(#service_lit);
                let _guard = span.enter();
                changed = true;
                registry.insert(
                    service_name,
                    Box::new(<#name as #crate_mod::service::Create>::create(&registry))
                    );
            } else {
                #crate_mod::tracing::debug!(message = "delay init due to missing dependencies of", service);
            }
        })
    });

    let service_init = manifest.services.iter().map(|service| {
        let name = &service.name;
        quote::quote!( #name: <#crate_mod::utils::TypeRegistry as #crate_mod::service::GetByName<#name>>::get(&registry).unwrap().into_inner() )
    });

    quote::quote!(
        #[derive(::std::clone::Clone)]
        #pub_token struct #name {
            #( #service_decl ),*
        }

        impl #name {
            pub fn create() -> Self {
                let span = #crate_mod::tracing::info_span!("init");
                let _guard = span.enter();

                let mut registry = #crate_mod::utils::TypeRegistry::new();

                let mut empty = false;
                let mut changed = true;
                while !empty && changed {
                    #crate_mod::tracing::trace!("start cycle");
                    empty = true;
                    changed = false;

                    #( #service_create )*
                }

                if !empty {
                    panic!("Services contains dependency cycle");
                }

                Self {
                    #( #service_init ),*
                }
            }
        }
    )
    .into()
}

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
    let depc =
        (0..service.deps().count()).map(|l| syn::LitInt::new(&format!("{l}"), output.span()));
    let dep_args = service.service_dependencies.iter();

    let name_lit = syn::LitStr::new(&name.to_string(), name.span());
    let tracing_args = quote::quote!(message = "created");

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

            fn deps() -> ::std::vec::Vec<::std::any::TypeId> {
                vec![ #( <#deps1 as #crate_mod::utils::Named>::name() ),* ]
            }

            fn create(registry: & #crate_mod::utils::TypeRegistry) -> Self::Service {
                let s = #crate_mod::tower::ServiceBuilder::new()
                    .boxed_clone()
                    .service(#name (
                        #( <#crate_mod::utils::TypeRegistry as #crate_mod::service::GetByName<#deps2>>::get(registry).unwrap()),*
                    ));
                #crate_mod::tracing::info!(#tracing_args);
                s
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
                use #crate_mod::tracing::Instrument;

                let this = self.clone();
                let fut = async move {
                    #crate_mod::tracing::trace!("called");
                    let result = Self::handle(request, #( this.#depc ),*).await;
                    #ret
                };
                let fut = fut.instrument(#crate_mod::tracing::info_span!(#name_lit));
                ::std::boxed::Box::pin(fut)
            }
        }
    )
    .into()
}
