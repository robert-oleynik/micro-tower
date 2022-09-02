use proc_macro::{Diagnostic, Level};
use syn::*;

pub struct Args {
    pub crate_path: syn::Path,
}

impl TryFrom<syn::AttributeArgs> for Args {
    type Error = syn::Error;

    fn try_from(args: syn::AttributeArgs) -> Result<Self> {
        let mut crate_path: syn::Path = syn::parse_str("::micro_tower")?;
        for arg in args {
            match arg {
                NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. }))
                    if path.is_ident("crate") =>
                {
                    match lit {
                        Lit::Str(str) => {
                            crate_path = syn::parse_str(&str.value())?;
                        }
                        _ => {
                            let type_str = crate::utils::lit_type_as_string(&lit);
                            Diagnostic::spanned(
                                vec![lit.span().unwrap()],
                                Level::Error,
                                format!("Expected 'string' but found '{type_str}'"),
                            )
                            .emit()
                        }
                    }
                }
                NestedMeta::Meta(Meta::Path(path))
                | NestedMeta::Meta(Meta::List(MetaList { path, .. }))
                | NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, .. })) => {
                    let last = path.segments.last().unwrap();
                    let id = last.ident.clone();
                    Diagnostic::spanned(
                        vec![last.ident.span().unwrap()],
                        Level::Error,
                        format!("Unknown argument '{id}'"),
                    )
                    .emit()
                }
                NestedMeta::Lit(lit) => {
                    Diagnostic::spanned(
                        vec![lit.span().unwrap()],
                        Level::Error,
                        "Unexpected literal",
                    )
                    .emit();
                }
            }
        }
        Ok(Self { crate_path })
    }
}
