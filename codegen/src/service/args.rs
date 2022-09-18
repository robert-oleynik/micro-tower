use darling::FromMeta;

#[derive(FromMeta)]
pub struct Args {
    #[darling(rename = "crate")]
    crate_path: Option<String>,
}
