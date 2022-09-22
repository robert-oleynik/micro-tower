use darling::FromMeta;
use quote::__private::Span;

use crate::util::diagnostic;

#[derive(FromMeta)]
pub struct Args {
    #[darling(rename = "crate")]
    crate_path: Option<syn::LitStr>,
    name: Option<String>,
}

impl Args {
    // Returns the module's base path. If option is not set will return the path `::micro_tower`.
    pub fn crate_path(&self) -> syn::Path {
        self.crate_path
            .as_ref()
            .and_then(|p| match syn::parse_str::<syn::Path>(&p.value()) {
                Ok(path) => Some(path),
                Err(err) => {
                    diagnostic::emit_error(p.span(), format!("{err}"));
                    None
                }
            })
            .unwrap_or_else(|| syn::parse_str("::micro_tower").unwrap())
    }

    /// Will return the service name as string literal. If option `name` is set will return this
    /// instead.
    pub fn name_str(&self, name: &syn::Ident) -> syn::LitStr {
        self.name.as_ref().map_or_else(
            || syn::LitStr::new(&name.to_string(), Span::call_site()),
            |name| syn::LitStr::new(name, Span::call_site()),
        )
    }
}
