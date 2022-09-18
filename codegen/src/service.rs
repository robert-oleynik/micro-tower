use quote::__private::TokenStream;

pub mod args;
pub mod decl;

pub fn generate(args: args::Args, decl: decl::Declaration) -> TokenStream {
    decl.emit_errors();
    let crate_path = args.crate_path();
    let name = decl.name();
    let name_str = args.name_str(name);
    let pub_token = decl.pub_token();

    let request_arg = decl.request_arg();
    let request_ty = decl.request_type();
    let response_ty = decl.response_type();

    let block = decl.block();

    let service_names0 = decl.service_names();
    let service_ty0 = decl.service_types();

    quote::quote!(
        #pub_token struct #name {
            #( #service_names0: #service_ty0 ),*
        }

        impl #crate_path::Service<#request_ty> for #name {
            type Response = #response_ty;
            type Error = #crate_path::util::BoxError;
            type Future = #crate_path::util::BoxFuture<Result<Self::Response, Self::Error>>;

            fn poll_ready(&mut self, _: ::std::task::Context<'_>) -> ::std::task::Poll<Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }

            fn call(&mut self, #request_arg) -> Self::Future {
                use #crate_path::prelude::Instrument;

                let fut = async move #block;

                let fut = fut.instrument(#crate_path::tracing::info_span!(#name_str));

                Box::pin(fut)
            }
        }
    )
}
