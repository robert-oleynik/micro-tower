use quote::__private::TokenStream;

pub mod args;
pub mod decl;

pub fn generate(args: args::Args, decl: decl::Declaration) -> TokenStream {
    decl.emit_errors();
    let crate_path = args.crate_path();
    let name = decl.name();
    let _name_str = args.name_str(name);
    let pub_token = decl.pub_token();

    let request_arg = decl.request_arg();
    let request_ty = decl.request_type();
    let response_ty = decl.response_type();

    quote::quote!(
        #pub_token struct #name {}

        impl #crate_path::Service<#request_ty> for #name {
            type Response = #response_ty;
            type Error = #crate_path::util::BoxError;
            type Future = #crate_path::util::BoxFuture<Result<Self::Response, Self::Error>>;

            fn poll_ready(&mut self, _: ::std::task::Context<'_>) -> ::std::task::Poll<Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }

            fn call(&mut self, #request_arg) -> Self::Future {
                todo!()
            }
        }
    )
}
