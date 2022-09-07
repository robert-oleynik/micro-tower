use micro_tower_codegen_macros::diagnostic;
use quote::__private::TokenStream;
use syn::parse::Parse;
use syn::spanned::Spanned;

use self::signature::Signature;

mod args;
mod signature;

/// Parse proc attribute items.
pub struct Items {
    signature: Signature,
    block: Box<syn::Block>,
}

pub struct Service {
    args: args::Args,
    items: Items,
}

pub fn pat_ident(pat: &syn::Pat) -> Option<&syn::Ident> {
    match pat {
        syn::Pat::Box(b) => pat_ident(&*b.pat),
        syn::Pat::Ident(ident) => Some(&ident.ident),
        syn::Pat::Reference(r) => pat_ident(&*r.pat),
        syn::Pat::Wild(_) => None,
        _ => {
            diagnostic!(error at [pat.span().unwrap()], "Pattern is not allowed in this context.");
            None
        }
    }
}

pub fn pat_type_to_field(arg: &syn::PatType) -> Option<syn::Field> {
    match &*arg.pat {
        syn::Pat::Ident(id) => Some(syn::Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Inherited,
            ident: Some(id.ident.clone()),
            colon_token: syn::parse2(quote::quote!(:)).unwrap(),
            ty: (*arg.ty).clone(),
        }),
        syn::Pat::Wild(_) => None,
        _ => {
            diagnostic!(error at [arg.span().unwrap()], "Pattern is not allowed in this context.");
            None
        }
    }
}

impl Service {
    pub fn new(args: syn::AttributeArgs, items: Items) -> Self {
        let args = args::Args::from(args);
        Self { args, items }
    }

    pub fn generate_struct(&self) -> TokenStream {
        let pub_token = &self.items.signature.pub_token;
        let name = &self.items.signature.ident;

        let fields = self
            .items
            .signature
            .inputs
            .iter()
            .skip(1)
            .filter_map(|arg| match arg {
                syn::FnArg::Receiver(recv) => {
                    diagnostic!(error at [recv.self_token.span().unwrap()], "`self` is not allowed in this context");
                    None
                },
                syn::FnArg::Typed(ty) => pat_type_to_field(ty),
            });

        quote::quote!(
            #[allow(non_camel_case_types)]
            #[derive(::std::clone::Clone)]
            #pub_token struct #name {
                #( #fields ),*
            }
        )
    }

    pub fn generate_handle(&self) -> TokenStream {
        let name = &self.items.signature.ident;
        let inputs = &self.items.signature.inputs;
        let output = &self.items.signature.output;
        let block = &self.items.block;

        quote::quote!(
            impl #name {
                async fn handle( #inputs ) #output #block
            }
        )
    }

    pub fn generate_buildable_impl(&self) -> TokenStream {
        let crate_path = &self.args.crate_path;
        let name = &self.items.signature.ident;

        quote::quote!(
            impl #crate_path::util::Buildable for #name {
                type Builder = ();

                fn builder() -> Self::Builder {}
            }
        )
    }

    pub fn generate_service_impl(&self) -> TokenStream {
        let tower_path = &self.args.tower_path;
        let tracing_path = &self.args.tracing_path;
        let name = &self.items.signature.ident;
        let name_lit = syn::LitStr::new(&name.to_string(), name.span());

        let request: syn::Type = match self.items.signature.inputs.first().and_then(|arg| match arg {
            syn::FnArg::Receiver(recv) => {
                diagnostic!(error at [recv.self_token.span().unwrap()], "`self` is not allowed in this context");
                None
            },
            syn::FnArg::Typed(ty) => Some((*ty.ty).clone()),
        }) {
            Some(ty) => ty,
            None => {
                diagnostic!(error at [self.items.signature.inputs.span().unwrap()], "No request type specified (Reason: Missing first parameter)");
                syn::parse2(quote::quote!(())).unwrap()
            }
        };

        let (response, ret) = match &self.items.signature.output {
            syn::ReturnType::Default => (quote::quote!(()), quote::quote!(Ok(result))),
            syn::ReturnType::Type(_, ty) => match **ty {
                syn::Type::Path(ref path)
                    if path
                        .path
                        .segments
                        .last()
                        .map(|seg| seg.ident == "Result")
                        .unwrap_or(false) =>
                {
                    path.path
                        .segments
                        .last()
                        .and_then(|last| match &last.arguments {
                            syn::PathArguments::AngleBracketed(args) if args.args.len() == 2 || args.args.len() == 1 => {
                                let ok_type = args.args.first().unwrap();
                                Some((quote::quote!( #ok_type ), quote::quote!( Ok(result?) )))
                            },
                            _ => None
                        })
                        .unwrap_or_else(|| {
                            diagnostic!(error at [ty.span().unwrap()], "Failed to infer response type. Couldn't infer ok result type.");
                            (quote::quote!(()), quote::quote!(Ok(result)))
                        })
                }
                _ => (quote::quote!(#ty), quote::quote!(Ok(result))),
            },
        };

        let fields = self.items
            .signature
            .inputs
            .iter()
            .skip(1)
            .filter_map(|arg| match arg {
                syn::FnArg::Receiver(recv) => {
                    diagnostic!(error at [recv.self_token.span().unwrap()], "`self` is not allowed in this context");
                    None
                },
                syn::FnArg::Typed(ty) => Some(ty),
            })
            .filter_map(|arg| pat_ident(&arg.pat));

        quote::quote!(
            impl #tower_path::Service<#request> for #name {
                type Response = #response;
                type Error = #tower_path::BoxError;
                type Future = ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

                fn poll_ready(&mut self, _: &mut ::std::task::Context<'_>) -> ::std::task::Poll<Result<(), Self::Error>> {
                    ::std::task::Poll::Ready(Ok(()))
                }

                fn call(&mut self, request: #request) -> Self::Future {
                    use #tracing_path::Instrument;

                    let this = self.clone();
                    let fut = async move {
                        #tracing_path::trace!("called");
                        let result = Self::handle(request, #( this.#fields ),*).await;
                        #ret
                    };
                    let fut = fut.instrument(#tracing_path::info_span!(#name_lit));
                    ::std::boxed::Box::pin(fut)
                }
            }
        )
    }
}

impl Parse for Items {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            signature: input.parse()?,
            block: input.parse()?,
        })
    }
}
