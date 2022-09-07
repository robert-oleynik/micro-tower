use micro_tower_codegen_macros::diagnostic;
use syn::parse::Parse;
use syn::spanned::Spanned;

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
            diagnostic!(error at [token.span().unwrap()], "Const service functions are not supported");
        }
        if signature.asyncness.is_none() {
            diagnostic!(error at [signature.span().unwrap()], "Service functions must be async");
        }
        if let Some(token) = signature.unsafety {
            diagnostic!(error at [token.span().unwrap()], "Unsafe service functions are not supported");
        }
        if let Some(token) = signature.generics.lt_token {
            diagnostic!(error at [token.span().unwrap()], "Generic service functions are not supported");
        }
        if let Some(token) = signature.variadic {
            diagnostic!(error at [token.span().unwrap()], "Variadic service functions are not supported");
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
