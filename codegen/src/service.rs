use quote::__private::TokenStream;

pub mod args;
pub mod decl;

pub fn generate(args: args::Args, decl: decl::Declaration) -> TokenStream {
    quote::quote!()
}
