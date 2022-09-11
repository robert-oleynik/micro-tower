use micro_tower_codegen_macros::diagnostic;
use syn::spanned::Spanned;
use syn::{Lit, Meta, MetaNameValue, NestedMeta};

pub struct Args {
    pub crate_path: syn::Path,
    pub tower_path: syn::Path,
    pub tracing_path: syn::Path,
    pub derive_builder_path: syn::Path,
    pub pool: Option<syn::LitInt>,
    pub buffer: Option<syn::LitInt>,
}

fn get_module_path(args: &[MetaNameValue], name: &str, def: syn::Path) -> syn::Path {
    args.iter()
        .find(|arg| arg.path.is_ident(name))
        .and_then(|arg| {
            if let Lit::Str(l) = &arg.lit {
                    Some(l)
            } else {
                let lit_type = crate::util::lit_type_as_string(&arg.lit);
                diagnostic!(error at [arg.lit.span().unwrap()], "Expected str literal but got {lit_type}");
                None
            }
        })
        .and_then(|lit| match syn::parse_str(&lit.value()) {
            Ok(path) => Some(path),
            Err(err) => {
                diagnostic!(error at [lit.span().unwrap()], "Expected valid module path ({err})");
                None
            }
        })
        .unwrap_or(def)
}

fn get_num(args: &[MetaNameValue], name: &str) -> Option<syn::LitInt> {
    args.iter()
        .find(|arg| arg.path.is_ident(name))
        .and_then(|arg| {
            if let Lit::Int(int) = &arg.lit {
                Some(int.clone())
            } else {
                let lit_type = crate::util::lit_type_as_string(&arg.lit);
                diagnostic!(error at [arg.lit.span().unwrap()], "Expected a integer literal but found {lit_type}");
                None
            }
        })
}

impl From<syn::AttributeArgs> for Args {
    fn from(args: syn::AttributeArgs) -> Self {
        const ARGS: &[&str] = &[
            "crate",
            "tracing",
            "tower",
            "derive_builder",
            "buffer",
            "pool",
        ];

        let args: Vec<_> = args.into_iter().filter_map(|arg| match arg {
            NestedMeta::Meta(Meta::NameValue(name_value)) => Some(name_value),
            NestedMeta::Meta(Meta::Path(path)) => {
                diagnostic!(error at [path.span().unwrap()], "Expected named value (`name = value`) but got path");
                None
            },
            NestedMeta::Meta(Meta::List(path)) => {
                diagnostic!(error at [path.span().unwrap()], "Expected named value (`name = value`) but got list");
                None
            },
            NestedMeta::Lit(lit) => {
                diagnostic!(error at [lit.span().unwrap()], "Expected named value (`name = value`) but got literal");
                None
            }
        }).filter(|arg| {
            if !ARGS.iter().any(|p| arg.path.is_ident(p)) {
                diagnostic!(warn at [arg.path.span().unwrap()], "Unknwon argument");
                return false
            }
            true
        }).collect();

        let crate_path = get_module_path(&args, "crate", syn::parse_str("::micro_tower").unwrap());
        let tower_path = get_module_path(
            &args,
            "tower",
            syn::parse2(quote::quote!(#crate_path::export::tower)).unwrap(),
        );
        let tracing_path = get_module_path(
            &args,
            "tracing",
            syn::parse2(quote::quote!(#crate_path::export::tracing)).unwrap(),
        );
        let derive_builder_path = get_module_path(
            &args,
            "derive_builder",
            syn::parse2(quote::quote!(#crate_path::export::derive_builder)).unwrap(),
        );

        let buffer = get_num(&args, "buffer");
        let pool = get_num(&args, "pool");

        Self {
            crate_path,
            tower_path,
            tracing_path,
            derive_builder_path,
            buffer,
            pool,
        }
    }
}
