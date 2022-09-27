use quote::__private::{Span, TokenStream};
use syn::spanned::Spanned;

use crate::util::diagnostic;

pub mod args;
pub mod decl;

pub fn generate(args: &args::Args, decl: &decl::Declaration) -> TokenStream {
    decl.emit_errors();
    let crate_path = args.crate_path();
    let name = decl.name();
    let name_builder = syn::Ident::new(format!("{name}_builder").as_ref(), Span::call_site());
    let name_str = args.name_str(name);
    let vis = decl.visibility();
    let attr = decl.attributes();
    let docs = decl.docs();

    let request_arg = decl.request_arg();
    let request_ty = decl.request_type();
    let (is_result, response_ty) = decl.response_type();

    let output = decl.output();

    let block = decl.block();

    let service_names0 = decl.service_names();
    let service_names1 = decl.service_names();
    let service_names2 = decl.service_names();
    let service_names3 = decl.service_names();
    let service_names4 = decl.service_names();
    let service_names5 = decl.service_names();
    let service_names6 = decl.service_names();
    let service_names7 = decl.service_names();
    let service_names_borrowed0 = decl.borrowed_service_names();
    let service_names_borrowed1 = decl.borrowed_service_names();
    let service_names_borrowed2 = decl.borrowed_service_names();
    let service_names_borrowed3 = decl.borrowed_service_names();
    let service_names_borrowed4 = decl.borrowed_service_names();
    let service_ty0 = decl.service_types();
    let service_ty1 = decl.service_types();
    let service_ty2 = decl.service_types();
    let service_ty3 = decl.service_types();

    let service_mut = decl.service_mut();

    let ret = if is_result {
        quote::quote!(Ok(result?))
    } else {
        quote::quote!(Ok(result))
    };

    if args.extend() && !attr.is_empty() {
        diagnostic::emit_warning(
            attr[0].span(),
            "Service extended but got service attributes. Declare attributes on first service declaration"
        );
    }

    let decl = if !args.extend() {
        quote::quote!(
            #( #attr )*
            #[allow(non_camel_case_types)]
            #vis struct #name {
                #( #service_names0: #crate_path::util::borrow::Cell<#service_ty0>, )*
                #( #service_names_borrowed0: Option<#crate_path::util::borrow::Borrowed<#service_ty3>> ),*
            }

            #[derive(Default)]
            #[allow(non_camel_case_types)]
            #vis struct #name_builder {
                #( #service_names4: Option<#service_ty1> ),*
            }

            impl #name {
                pub fn builder() -> #name_builder {
                    #name_builder::default()
                }
            }

            impl #name_builder {
                #(
                    #[must_use]
                    pub fn #service_names2(mut self, inner: #service_ty2) -> Self {
                        self.#service_names2 = Some(inner);
                        self
                    }
                )*

                #[must_use]
                pub fn build(mut self) -> #name {
                    #(
                        let #service_names5 = match self.#service_names5.take() {
                            Some(inner) => inner,
                            None => panic!("service `{0}` is not set", ::std::stringify!(#service_names5))
                        };
                    ),*

                    #name {
                        #( #service_names6: #crate_path::util::borrow::Cell::new(#service_names6), )*
                        #( #service_names_borrowed1: None ),*
                    }
                }
            }
        )
    } else {
        quote::quote!()
    };

    quote::quote!(
        #decl

        #( #docs )*
        impl #crate_path::Service<#request_ty> for #name {
            type Response = #response_ty;
            type Error = #crate_path::util::BoxError;
            type Future = #crate_path::util::BoxFuture<Result<Self::Response, Self::Error>>;

            fn poll_ready(&mut self, cx: &mut ::std::task::Context<'_>) -> ::std::task::Poll<Result<(), Self::Error>> {
                #(
                    self.#service_names_borrowed2 = None;
                    let #service_names3 = match self.#service_names3.try_borrow() {
                        Some(service) => service,
                        None => return ::std::task::Poll::Pending
                    };
                )*
                #( self.#service_names_borrowed3 = Some(#service_names7); )*
                ::std::task::Poll::Ready(Ok(()))
            }

            fn call(&mut self, #request_arg) -> Self::Future {
                use #crate_path::prelude::Instrument;

                #(
                    let #service_mut #service_names1 = match self.#service_names_borrowed4.take() {
                        Some(inner) => inner,
                        None => {
                            return ::std::boxed::Box::pin(async move {
                                let err = #crate_path::service::NotReady(::std::stringify!(#service_names1));
                                Err(::std::boxed::Box::new(err).into())
                            })
                        }
                    };
                ),*

                let fut: #crate_path::util::BoxFuture<#output> = Box::pin(async move #block);
                let fut = async move {
                    let result = fut.await;
                    #ret
                };

                let fut = fut.instrument(#crate_path::export::tracing::info_span!(#name_str));

                Box::pin(fut)
            }
        }
    )
}
