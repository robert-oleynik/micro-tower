use syn::parse::Parse;

/// Used to represent service signatures from service.
pub struct Signature {
    pub_token: Option<syn::Token!(pub)>,
    _async: syn::Token!(async),
    _fn: syn::Token!(fn),
    ident: syn::Ident,
    _paren_token: syn::token::Paren,
    inputs: syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    _ra: syn::token::RArrow,
    output: syn::Type,
}

impl Parse for Signature {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            pub_token: input.parse()?,
            _async: input.parse()?,
            _fn: input.parse()?,
            ident: input.parse()?,
            _paren_token: syn::parenthesized!(content in input),
            inputs: content.parse_terminated(syn::FnArg::parse)?,
            _ra: input.parse()?,
            output: input.parse()?,
        })
    }
}

impl Signature {
    pub fn pub_token(&self) -> Option<&syn::Token!(pub)> {
        self.pub_token.as_ref()
    }

    pub fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    pub fn request_arg(&self) -> &syn::FnArg {
        &self.inputs.first().expect("Expected reqeust argument")
    }

    pub fn request_type(&self) -> &syn::Type {
        match self.request_arg() {
            syn::FnArg::Typed(ty) => &ty.ty,
            _ => panic!("self is not allowed in this context"),
        }
    }

    pub fn response_type(&self) -> &syn::Type {
        &self.output
    }
}
