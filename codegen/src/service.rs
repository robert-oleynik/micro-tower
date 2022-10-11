use quote::__private::{Span, TokenStream};
use syn::spanned::Spanned;

use crate::util::diagnostic;

pub mod args;

pub struct Service {
    // Meta-Info
    crate_path: syn::Path,
    vis: syn::Visibility,
    asyncness: Option<syn::token::Async>,
    // Names
    name: syn::Ident,
    name_str: syn::LitStr,
    name_builder: syn::Ident,
    // Request/Response
    request_arg: syn::PatType,
    output: syn::ReturnType,
    response: syn::Type,
    failable: bool,
    // Services
    inner_srv: Vec<syn::Ident>,
    inner_srv_m: Vec<Option<syn::token::Mut>>,
    inner_srv_b: Vec<syn::Ident>,
    inner_srv_t: Vec<syn::Type>,
    // Attributes
    doc_attrs: Vec<syn::Attribute>,
    // Block
    block: Box<syn::Block>,
}

fn default_pat_type() -> syn::PatType {
    syn::PatType {
        attrs: Vec::new(),
        pat: Box::new(syn::Pat::Wild(syn::PatWild {
            attrs: Vec::new(),
            underscore_token: syn::token::Underscore {
                spans: [Span::call_site(); 1],
            },
        })),
        colon_token: syn::token::Colon {
            spans: [Span::call_site(); 1],
        },
        ty: syn::parse_str("()").unwrap(),
    }
}

impl Service {
    pub fn new(args: &args::Args, decl: syn::ItemFn) -> Self {
        if let Some(tk) = decl.sig.constness {
            diagnostic::emit_error(tk.span(), "`const` services are not allowed");
        }
        if let Some(tk) = decl.sig.unsafety {
            diagnostic::emit_error(tk.span(), "service must be safe");
        }
        if let Some(tk) = decl.sig.abi {
            diagnostic::emit_error(
                tk.extern_token.span(),
                "`extern` is not allowed in this context",
            );
        }
        if decl.sig.generics.lt_token.is_some() {
            diagnostic::emit_error(decl.sig.generics.span(), "Not implemented yet");
        }
        if decl.sig.variadic.is_some() {
            diagnostic::emit_error(
                decl.sig.variadic.span(),
                "Variadic service arguments are not supported",
            );
        }
        let name_str = args.name_str(&decl.sig.ident);
        let doc_attrs = decl
            .attrs
            .into_iter()
            .filter(|attr| {
                if attr.path.is_ident("doc") {
                    true
                } else {
                    diagnostic::emit_error(
                        attr.span(),
                        "Attributes are not allowed before function declaration.",
                    );
                    false
                }
            })
            .collect::<Vec<_>>();
        let inputs = decl
            .sig
            .inputs
            .into_iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Receiver(recv) => {
                    diagnostic::emit_error(recv.span(), "`self` is not allowed in this context");
                    None
                }
                syn::FnArg::Typed(ty) => Some(ty),
            })
            .collect::<Vec<_>>();
        let request_arg = inputs.first().map_or_else(
            || {
                diagnostic::emit_error(decl.sig.paren_token.span, "Missing request parameter");
                default_pat_type()
            },
            syn::PatType::clone,
        );
        let inner_srv = inputs
            .iter()
            .skip(1)
            .filter_map(|arg| match arg.pat.as_ref() {
                syn::Pat::Ident(id) => Some(id.ident.clone()),
                syn::Pat::Wild(_) => Some(syn::parse_str("_").unwrap()),
                _ => {
                    diagnostic::emit_error(arg.span(), "Invalid argument declaration");
                    None
                }
            })
            .collect::<Vec<_>>();
        let inner_srv_b = inner_srv
            .iter()
            .map(|srv| syn::Ident::new(&format!("__borrowed_{srv}"), Span::call_site()))
            .collect();
        let inner_srv_m = inputs
            .iter()
            .skip(1)
            .filter_map(|arg| match arg.pat.as_ref() {
                syn::Pat::Ident(id) => Some(id.mutability),
                syn::Pat::Wild(_) => Some(None),
                _ => None,
            })
            .collect();
        let inner_srv_t = inputs
            .into_iter()
            .skip(1)
            .map(|input| (*input.ty).clone())
            .collect();

        let output = decl.sig.output;
        let (failable, response) = match output {
            syn::ReturnType::Default => (false, syn::parse_str("()").unwrap()),
            syn::ReturnType::Type(_, ref ty) => match **ty {
                syn::Type::Path(ref p) => {
                    if let Some(ty) = p
                        .path
                        .segments
                        .last()
                        .and_then(|seg| {
                            if seg.ident == "Result" {
                                return Some(seg);
                            }
                            None
                        })
                        .and_then(|seg| match &seg.arguments {
                            syn::PathArguments::AngleBracketed(args) => args.args.first(),
                            _ => None,
                        })
                        .and_then(|arg| match arg {
                            syn::GenericArgument::Type(ty) => Some(ty.clone()),
                            _ => None,
                        })
                    {
                        (true, ty)
                    } else {
                        (false, *ty.clone())
                    }
                }
                _ => (false, *ty.clone()),
            },
        };

        Self {
            crate_path: args.crate_path(),
            vis: decl.vis,
            asyncness: decl.sig.asyncness,
            name_builder: syn::Ident::new(&format!("{}Builder", decl.sig.ident), Span::call_site()),
            name: decl.sig.ident,
            name_str,
            request_arg,
            output,
            response,
            failable,
            inner_srv,
            inner_srv_b,
            inner_srv_t,
            inner_srv_m,
            doc_attrs,
            block: decl.block,
        }
    }

    pub fn gen_service_decl(&self) -> TokenStream {
        let crate_path = &self.crate_path;
        let name = &self.name;
        let name_builder = &self.name_builder;
        let vis = &self.vis;
        let srv_names = &self.inner_srv;
        let srv_names_b = &self.inner_srv_b;
        let srv_ty = &self.inner_srv_t;
        quote::quote!(
            #[allow(non_camel_case_types)]
            #vis struct #name {
                #( #srv_names: #crate_path::util::borrow::Cell<#srv_ty>, )*
                #( #srv_names_b: Option<#crate_path::util::borrow::Borrowed<#srv_ty>> ),*
            }

            impl #name {
                #[must_use]
                pub fn builder() -> #name_builder {
                    #name_builder::default()
                }
            }
        )
    }

    pub fn gen_service_builder(&self) -> TokenStream {
        let crate_path = &self.crate_path;
        let name = &self.name;
        let name_builder = &self.name_builder;
        let vis = &self.vis;
        let srv_names = &self.inner_srv;
        let srv_names_b = &self.inner_srv_b;
        let srv_ty = &self.inner_srv_t;
        quote::quote!(
            #[derive(Default)]
            #[allow(non_camel_case_types)]
            #vis struct #name_builder {
                #( #srv_names: Option<#srv_ty> ),*
            }

            impl #name_builder {
                #(
                    #[must_use]
                    pub fn #srv_names(mut self, inner: #srv_ty) -> Self {
                        self.#srv_names = Some(inner);
                        self
                    }
                )*

                #[must_use]
                pub fn build(mut self) -> #name {
                    #(
                        let #srv_names = match self.#srv_names.take() {
                            Some(inner) => inner,
                            None => panic!("service `{}` is not set for `{}`", ::std::stringify!(#srv_names), ::std::stringify!(#name))
                        };
                    )*

                    #name {
                        #( #srv_names: #crate_path::util::borrow::Cell::new(#srv_names), )*
                        #( #srv_names_b: None ),*
                    }
                }
            }
        )
    }

    pub fn gen_service_block(&self) -> TokenStream {
        let crate_path = &self.crate_path;
        let name_str = &self.name_str;
        let block = self.block.as_ref();
        let ret = if self.failable {
            quote::quote!(Ok(result?))
        } else {
            quote::quote!(Ok(result))
        };
        if self.asyncness.is_some() {
            let output = match &self.output {
                syn::ReturnType::Default => quote::quote!(()),
                syn::ReturnType::Type(_, t) => quote::quote!(#t),
            };
            quote::quote!(
                use #crate_path::prelude::Instrument;
                let fut: #crate_path::util::BoxFuture<#output> = Box::pin(async move #block);
                let fut = async move {
                    let result = fut.await;
                    #ret
                };
                let fut = fut.instrument(#crate_path::export::tracing::trace_span!(#name_str));
                Box::pin(fut)
            )
        } else {
            let output = &self.output;
            quote::quote!(
                Box::pin(async move {
                    let result = #crate_path::export::tokio::task::spawn_blocking(move || #output {
                        let _span_ = #crate_path::export::tracing::trace_span!(#name_str).entered();
                        #block
                    }).await?;
                    #ret
                })
            )
        }
    }

    pub fn gen_service_impl(&self) -> TokenStream {
        let crate_path = &self.crate_path;
        let name = &self.name;
        let name_str = &self.name_str;
        let docs = &self.doc_attrs;
        let request_ty = self.request_arg.ty.as_ref();
        let request_arg = &self.request_arg;
        let response_ty = &self.response;
        let srv_names_b = &self.inner_srv_b;
        let srv_names = &self.inner_srv;
        let srv_mut = &self.inner_srv_m;
        let block = self.gen_service_block();
        quote::quote!(
            #( #docs )*
            impl #crate_path::Service<#request_ty> for #name {
                type Response = #response_ty;
                type Error = #crate_path::util::BoxError;
                type Future = #crate_path::util::BoxFuture<Result<Self::Response, Self::Error>>;

                fn poll_ready(&mut self, cx: &mut ::std::task::Context<'_>) -> ::std::task::Poll<Result<(), Self::Error>> {
                    #(
                        self.#srv_names_b = None;
                        let #srv_names = match self.#srv_names.try_borrow() {
                            Some(service) => service,
                            None => return ::std::task::Poll::Pending
                        };
                    )*
                    #( self.#srv_names_b = Some(#srv_names); )*
                    ::std::task::Poll::Ready(Ok(()))
                }

                fn call(&mut self, #request_arg) -> Self::Future {
                    #(
                        let #srv_mut #srv_names = match self.#srv_names_b.take() {
                            Some(inner) => inner,
                            None => return ::std::boxed::Box::pin(async move {
                                    let err = #crate_path::service::NotReady(::std::stringify!(#srv_names));
                                    Err(::std::boxed::Box::new(err).into())
                            })
                        };
                    )*
                    #block
                }
            }

            impl #crate_path::service::Info for #name {
                type Request = #request_ty;

                fn name() -> &'static str {
                    #name_str
                }
            }
        )
    }

    pub fn generate(&mut self) -> TokenStream {
        let decl = self.gen_service_decl();
        let builder = self.gen_service_builder();
        let service_impl = self.gen_service_impl();

        quote::quote!(
            #decl
            #builder

            #service_impl
        )
    }
}
