use quote::__private::{Span, TokenStream};

pub mod args;
pub mod decl;

pub fn generate(args: args::Args, decl: decl::Declaration) -> TokenStream {
    decl.emit_errors();
    let crate_path = args.crate_path();
    let name = decl.name();
    let name_builder = syn::Ident::new(format!("{name}Builder").as_ref(), Span::call_site());
    let name_str = args.name_str(name);
    let pub_token = decl.pub_token();

    let request_arg = decl.request_arg();
    let request_ty = decl.request_type();
    let response_ty = decl.response_type();

    let block = decl.block();

    let service_names0 = decl.service_names();
    let service_names1 = decl.service_names();
    let service_names2 = decl.service_names();
    let service_names3 = decl.service_names();
    let service_ty0 = decl.service_types();
    let service_ty2 = decl.service_types();

    quote::quote!(
        #[derive(#crate_path::export::derive_builder::Builder)]
        #[builder(pattern = "owned")]
        #pub_token struct #name {
            #(
                #[builder(setter(custom))]
                #service_names0: #crate_path::util::borrow::Cell<#service_ty0>
            ),*
        }

        impl #name_builder {
            #(
            pub fn #service_names2(mut self, inner: #service_ty2) -> Self {
                self.#service_names2 = Some(#crate_path::util::borrow::Cell::new(inner));
                self
            }
            )*
        }

        impl #crate_path::Service<#request_ty> for #name {
            type Response = #response_ty;
            type Error = #crate_path::util::BoxError;
            type Future = #crate_path::util::BoxFuture<Result<Self::Response, Self::Error>>;

            fn poll_ready(&mut self, cx: ::std::task::Context<'_>) -> ::std::task::Poll<Result<(), Self::Error>> {
                #(
                    if let Some(inner) = self.#service_names3.borrow() {
                        match inner.poll_ready(cx) {
                            ::std::task::Poll::Ready(Ok(_)) => {}
                            ::std::task::Poll::Ready(Err(err)) => {
                                return ::std::task::Poll::Ready(::std::boxed::Box::new(err).into)
                            },
                            ::std::task::Poll::Pending => {
                                return::std::task::Poll::Pending
                            }
                        }
                    }
                )*
                Poll::Ready(Ok(()))
            }

            fn call(&mut self, #request_arg) -> Self::Future {
                use #crate_path::prelude::Instrument;

                #(
                    let #service_names1 = match self.#service_names1.borrow() {
                        Some(inner) => inner.
                        None = return Box::pin(async move { Err(Box::new(#crate_path::service::NotReady).into()) })
                    };
                ),*

                let fut = async move #block;

                let fut = fut.instrument(#crate_path::tracing::info_span!(#name_str));

                Box::pin(fut)
            }
        }
    )
}
