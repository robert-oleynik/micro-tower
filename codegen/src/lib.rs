use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn service(args: TokenStream, items: TokenStream) -> TokenStream {
    items
}
