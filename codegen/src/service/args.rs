use micro_tower_codegen_macros::diagnostic;

#[derive(darling::FromMeta)]
pub struct Args {
    #[darling(rename = "crate")]
    crate_path: Option<syn::LitStr>,
    buffer: Option<syn::LitInt>,
}

impl Args {
    fn default_crate_path() -> syn::Path {
        syn::parse_str("::micro_tower").unwrap()
    }

    pub fn parse_crate_path(&self) -> syn::parse::Result<syn::Path> {
        if let Some(crate_path) = &self.crate_path {
            let parse: syn::Path = syn::parse_str(&crate_path.value())?;
            Ok(parse)
        } else {
            Ok(Self::default_crate_path())
        }
    }

    /// Verify inputs and send errors/warnings.
    pub fn verify(&self) -> syn::parse::Result<bool> {
        if let Some(buffer) = &self.buffer {
            let buf_size: usize = buffer.base10_parse()?;
            if buf_size == 0 {
                diagnostic!(error at [buffer.span().unwrap()], "expected buffer len unequals `0`");
            }
        }

        if let Some(crate_path) = &self.crate_path {
            if let Err(err) = self.parse_crate_path() {
                diagnostic!(error at [crate_path.span().unwrap()], "invalid module path");
                return Err(err);
            }
        }

        Ok(true)
    }

    pub fn crate_path(&self) -> syn::Path {
        self.parse_crate_path()
            .unwrap_or(Self::default_crate_path())
    }

    pub fn derive_builder_path(&self) -> syn::Path {
        let path = self.crate_path();
        syn::parse2(quote::quote!( #path :: export :: derive_builder )).unwrap()
    }

    pub fn tower_path(&self) -> syn::Path {
        let path = self.crate_path();
        syn::parse2(quote::quote!( #path :: export :: tower )).unwrap()
    }

    pub fn tracing_path(&self) -> syn::Path {
        let path = self.crate_path();
        syn::parse2(quote::quote!( #path :: export :: tracing )).unwrap()
    }

    pub fn buffer_len(&self) -> Option<&syn::LitInt> {
        self.buffer.as_ref()
    }

    /// Returns true if any option is enabled which modifies the error type.
    pub fn require_map_error(&self) -> bool {
        self.buffer.is_some()
    }
}
