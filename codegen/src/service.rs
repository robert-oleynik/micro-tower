use micro_tower_codegen_macros::diagnostic;
use quote::__private::{Span, TokenStream};
use syn::{spanned::Spanned, LitStr};

mod args;
mod items;
mod signature;

pub use args::Args;
pub use items::Items;

/// Generate service implementation where `args` specify the service attributes and `items` the
/// service functionality.
///
/// Will generate:
///
/// - a struct with name of service signature
/// - a builder to build this struct. The builder will create a setter for each service and
///   connection specified.
/// - an implementation of [`micro_tower::util::Buildable`]
/// - an implementation of [`micro_tower::tower::Service`]
pub fn generate(args: Args, items: Items) -> TokenStream {
    let crate_path = args.crate_path();
    let derive_builder_path = args.derive_builder_path();
    let tower_path = args.tower_path();
    let tracing_path = args.tracing_path();

    let name = items.name();
    let name_builder = syn::Ident::new(&format!("{name}Builder"), Span::call_site());
    let pub_token = items.pub_token();

    let mut error_path: String = derive_builder_path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string() + "::")
        .collect();

    if derive_builder_path.leading_colon.is_some() {
        error_path = String::from("::") + &error_path;
    }
    let error_ty_lit: syn::LitStr =
        syn::parse_str(&format!("\"{error_path}UninitializedFieldError\"")).unwrap();

    let fields0 = items.services().map(pat_type_to_field);
    let fields1 = items.services().map(pat_type_to_field);
    let field_names = items.services().map(|arg| pat_ident(&arg.pat));
    let field_names_lit = items
        .services()
        .map(|arg| pat_ident(&arg.pat))
        .flatten()
        .map(|field| field.to_string())
        .map(|field| syn::LitStr::new(&field, Span::call_site()));

    let buffer = if let Some(buffer) = args.buffer_len() {
        quote::quote!( .buffer(#buffer) )
    } else {
        quote::quote!()
    };

    let concurrency = if let Some(concurrency) = args.concurrency_limit() {
        quote::quote!( .concurrency_limit(#concurrency) )
    } else {
        quote::quote!()
    };

    let map_err = if args.require_map_error() {
        quote::quote!( .layer(#crate_path::service::map_error::Layer::default()) )
    } else {
        quote::quote!()
    };

    let inputs = items.inputs();
    let output = items.output();
    let block = items.block();

    let request = items.request_type();
    let (response, extr_err) = items.response_type();
    let error = items.error_type();

    let mut service_type = quote::quote!( #name );
    if args.concurrency_limit().is_some() {
        service_type =
            quote::quote!( #tower_path::limit::concurrency::ConcurrencyLimit<#service_type> );
    }
    if args.buffer_len().is_some() {
        service_type = quote::quote!( #tower_path::buffer::Buffer<#service_type, #request> );
    }
    if args.require_map_error() {
        service_type =
            quote::quote!( #crate_path::service::map_error::Service<#request, #service_type> );
    }

    let ret = if extr_err {
        quote::quote!(result)
    } else {
        quote::quote!(Ok(result))
    };
    let name_lit = LitStr::new(&name.to_string(), Span::call_site());

    quote::quote!(
        #[allow(non_camel_case_types)]
        #[derive(::std::clone::Clone, #derive_builder_path::Builder)]
        #[builder(build_fn(skip, error = #error_ty_lit))]
        #pub_token struct #name {
            #( #fields0 ),*
        }

        impl #name_builder {
            pub fn build(&mut self) -> Result<#crate_path::service::Service<#name>, #derive_builder_path::UninitializedFieldError> {
                let service = #name {
                    #( #field_names: self.#field_names.clone()
                        .ok_or(#derive_builder_path::UninitializedFieldError::new(#field_names_lit))? ),*
                };

                let service = #tower_path::ServiceBuilder::new()
                    #map_err
                    #buffer
                    #concurrency
                    .service(service);
                Ok(#crate_path::service::Service::from_service(service))
            }
        }

        impl #name {
            async fn handle(#(#inputs),*) #output #block
        }

        impl #crate_path::util::Buildable for #name {
            type Target = #service_type;
            type Builder = #name_builder;

            fn builder() -> Self::Builder {
                #name_builder::default()
            }
        }

        impl #tower_path::Service<#request> for #name {
            type Response = #response;
            type Error = #error;
            type Future = ::std::pin::Pin<::std::boxed::Box<dyn  ::std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

            fn poll_ready(&mut self, _: &mut ::std::task::Context<'_>) -> ::std::task::Poll<Result<(), Self::Error>> {
                ::std::task::Poll::Ready(Ok(()))
            }

            fn call(&mut self, request: #request) -> Self::Future {
                use #tracing_path::Instrument;

                let this = self.clone();
                let fut = async move {
                    #tracing_path::trace!("called");
                    let result = Self::handle(request, #( this.#fields1 ),*).await;
                    #ret
                };
                let fut = fut.instrument(#tracing_path::info_span!(#name_lit));
                ::std::boxed::Box::pin(fut)
            }
        }
    )
}

pub fn pat_ident(pat: &syn::Pat) -> Option<&syn::Ident> {
    match pat {
        syn::Pat::Box(b) => pat_ident(&b.pat),
        syn::Pat::Ident(ident) => Some(&ident.ident),
        syn::Pat::Reference(r) => pat_ident(&r.pat),
        syn::Pat::Wild(_) => None,
        _ => {
            diagnostic!(error at [pat.span().unwrap()], "Pattern is not allowed in this context.");
            None
        }
    }
}

pub fn pat_type_to_field(arg: &syn::PatType) -> Option<syn::Field> {
    match &*arg.pat {
        syn::Pat::Ident(id) => Some(syn::Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Inherited,
            ident: Some(id.ident.clone()),
            colon_token: syn::parse2(quote::quote!(:)).unwrap(),
            ty: (*arg.ty).clone(),
        }),
        syn::Pat::Wild(_) => None,
        _ => {
            diagnostic!(error at [arg.span().unwrap()], "Pattern is not allowed in this context.");
            None
        }
    }
}
