use quote::__private::TokenStream;

pub mod args;
pub mod decl;

pub fn generate(args: args::Args, decl: decl::Declaration) -> TokenStream {
    decl.emit_errors();
    let _crate_path = args.crate_path();
    let name = decl.name();
    let _name_str = args.name_str(name);
    quote::quote!()
}
