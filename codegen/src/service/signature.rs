use proc_macro::{Diagnostic, Level};
use syn::parse::Parse;

/// Used to represent service signatures from service.
pub struct Signature {
    pub pub_token: Option<syn::Token!(pub)>,
    pub ident: syn::Ident,
    pub paren_token: syn::token::Paren,
    pub inputs: syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    pub output: syn::ReturnType,
}

impl Parse for Signature {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pub_token: Option<syn::Token!(pub)> = input.parse()?;
        let signature: syn::Signature = input.parse()?;
        if let Some(token) = signature.constness {
            Diagnostic::spanned(
                vec![token.span.unwrap()],
                Level::Error,
                "Const function are not supported",
            )
            .emit()
        }
        if signature.asyncness.is_none() {
            Diagnostic::spanned(
                vec![signature.fn_token.span.unwrap()],
                Level::Error,
                "Service function must be async.",
            )
            .emit()
        }
        if let Some(token) = signature.unsafety {
            Diagnostic::spanned(
                vec![token.span.unwrap()],
                Level::Error,
                "unsafe service functions are not supported",
            )
            .emit()
        }
        if let Some(token) = signature.generics.lt_token {
            Diagnostic::spanned(
                vec![token.span.unwrap()],
                Level::Error,
                "Generics are not supported",
            )
            .emit()
        }
        if let Some(token) = signature.variadic {
            Diagnostic::spanned(
                vec![token.dots.spans[0].unwrap()],
                Level::Error,
                "Variadic functions are not supported",
            )
            .emit()
        }

        Ok(Self {
            pub_token,
            ident: signature.ident,
            paren_token: signature.paren_token,
            inputs: signature.inputs,
            output: signature.output,
        })
    }
}
