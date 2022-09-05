use syn::parse::Parse;
use syn::punctuated::Punctuated;

pub struct Manifest {
    pub pub_token: Option<syn::token::Pub>,
    pub name: syn::Ident,
    _colon: syn::token::Colon,
    pub brackets: syn::token::Bracket,
    pub services: Punctuated<Service, syn::token::Comma>,
}

pub struct Service {
    pub name: syn::Ident,
    _colon: Option<syn::token::Colon>,
    pub port: Option<syn::LitInt>,
}

impl Parse for Manifest {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Manifest {
            pub_token: input.parse()?,
            name: input.parse()?,
            _colon: input.parse()?,
            brackets: syn::bracketed!(content in input),
            services: content.parse_terminated(Service::parse)?,
        })
    }
}

impl Parse for Service {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let colon: Option<syn::token::Colon> = input.parse()?;
        let port: Option<syn::LitInt> = match colon.is_some() {
            true => Some(input.parse()?),
            false => None,
        };
        Ok(Service {
            name,
            _colon: colon,
            port,
        })
    }
}
