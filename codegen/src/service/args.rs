use darling::FromMeta;

#[derive(FromMeta)]
pub struct Args {
    #[darling(rename = "crate")]
    crate_path: Option<String>,
}

impl Args {
    // Returns the module's base path. If option is not set will return the path `::micro_tower`.
    pub fn crate_path(&self) -> syn::Path {
        self.crate_path
            .as_ref()
            .and_then(|p| match syn::parse_str::<syn::Path>(&p) {
                Ok(path) => Some(path),
                Err(err) => {
                    // TODO: emit compile error
                    None
                }
            })
            .unwrap_or_else(|| syn::parse_str("::micro_tower").unwrap())
    }
}
