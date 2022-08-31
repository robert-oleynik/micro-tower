use proc_macro::TokenStream;
use syn::parse::Parse;
use syn::Type;

/// Parse proc attribute arguments.
pub struct Args {}

impl Parse for Args {
    fn parse(_: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {})
    }
}

/// Parse proc attribute items.
pub struct Items {
    pub_token: Option<syn::Token!(pub)>,
    _async: syn::Token!(async),
    _fn: syn::Token!(fn),
    ident: syn::Ident,
    _paren_token: syn::token::Paren,
    inputs: syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    _ra: syn::token::RArrow,
    output: Type,
    block: Box<syn::Block>,
}

impl Items {
    pub fn generate(self) -> TokenStream {
        let pub_token = self.pub_token;
        let ident = self.ident;
        let block = self.block;
        let inputs = self.inputs;
        let output = self.output;
        let request = inputs.first().expect("Expected at least one parameter");
        let request_type = match request {
            syn::FnArg::Typed(ty) => ty.ty.clone(),
            _ => panic!("self is not allowed in this context"),
        };
        quote::quote!(
            #[allow(non_camel_case_types)]
            #[derive(::std::clone::Clone)]
            #pub_token struct #ident;

            impl ::micro_tower::core::service::Create for #ident {
                type Service = ::micro_tower::tower::util::BoxCloneService<#request_type, #output, ::std::convert::Infallible>;

                fn create() -> Self::Service {
                    ::micro_tower::tower::ServiceBuilder::new()
                        .boxed_clone()
                        .service(#ident)
                }
            }

            impl ::micro_tower::tower::Service< #request_type > for #ident {
                type Response = #output;
                type Error = ::std::convert::Infallible;
                type Future = ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

                fn poll_ready(&mut self, cx: &mut ::std::task::Context<'_>) -> ::std::task::Poll<Result<(), Self::Error>> {
                    ::std::task::Poll::Ready(Ok(()))
                }

                fn call(&mut self, #request) -> Self::Future {
                    let fut = async move #block;
                    ::std::boxed::Box::pin(async move {
                        Ok(fut.await)
                    })
                }
            }
        )
        .into()
    }
}

impl Parse for Items {
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
            block: input.parse()?,
        })
    }
}
