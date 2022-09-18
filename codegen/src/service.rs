use quote::__private::TokenStream;

pub mod args;
pub mod decl;

pub fn generate(args: args::Args, decl: decl::Declaration) -> TokenStream {
    let _crate_path = args.crate_path();
    quote::quote!()
}
