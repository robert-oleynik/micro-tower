use quote::__private::TokenStream;

pub mod args;
pub mod decl;

pub fn generate(args: args::Args, decl: decl::Declaration) -> TokenStream {
    decl.emit_errors();
    let crate_path = args.crate_path();
    let name = decl.name();
    let _name_str = args.name_str(name);
    let pub_token = decl.pub_token();

    quote::quote!(
        #pub_token struct #name {}

        impl #crate_path::Service< () > for #name {
            type Response = ();
            type Error = #crate_path::util::BoxError;
            type Future = #crate_path::util::BoxFuture<Result<Self::Response, Self::Error>>;

            fn poll_ready(&mut self, _: ::std::task::Context<'_>) -> ::std::task::Poll<Result<(), Self::Error>> {
                todo!()
            }

            fn call(&mut self, _: ()) -> Self::Future {
                todo!()
            }
        }
    )
}
