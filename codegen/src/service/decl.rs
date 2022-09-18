use syn::parse::Parse;

pub struct Declaration {
    signature: syn::Signature,
    block: Box<syn::Block>,
}

impl Parse for Declaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            signature: input.parse()?,
            block: input.parse()?,
        })
    }
}
