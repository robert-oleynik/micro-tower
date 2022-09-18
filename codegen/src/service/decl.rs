use std::ops::Deref;

use quote::__private::Span;
use syn::{parse::Parse, spanned::Spanned, ReturnType};

use crate::util::diagnostic;

pub struct Declaration {
    pub_token: Option<syn::token::Pub>,
    signature: syn::Signature,
    block: Box<syn::Block>,
}

impl Declaration {
    /// Find errors in service signature and generate error messages from them.
    pub fn emit_errors(&self) {
        if let Some(tk) = self.signature.constness {
            diagnostic::emit_error(tk.span(), "`const` services are not allowed");
        }
        if self.signature.asyncness.is_none() {
            diagnostic::emit_error(self.signature.span(), "service must be async");
        }
        if let Some(tk) = self.signature.unsafety {
            diagnostic::emit_error(tk.span(), "service must be safe");
        }
        if let Some(tk) = &self.signature.abi {
            diagnostic::emit_error(
                tk.extern_token.span(),
                "`extern` is not allowed in this context",
            );
        }
        if self.signature.generics.lt_token.is_some() {
            diagnostic::emit_error(self.signature.generics.span(), "Not implemented yet");
        }
        if self.signature.variadic.is_some() {
            diagnostic::emit_error(
                self.signature.variadic.span(),
                "Variadic service arguments are not supported",
            );
        }
    }

    /// Returns the service name as identifier.
    pub fn name(&self) -> &syn::Ident {
        &self.signature.ident
    }

    /// Return `pub` token if service should be public and `None` if not.
    pub fn pub_token(&self) -> Option<&syn::token::Pub> {
        self.pub_token.as_ref()
    }

    /// Returns reference to service's request argument used by the service implementation.
    pub fn request_arg(&self) -> syn::FnArg {
        self.signature
            .inputs
            .first()
            .map(syn::FnArg::clone)
            .unwrap_or_else(|| {
                diagnostic::emit_error(Span::call_site(), "Missing request argument");
                syn::parse_str("_: ()").unwrap()
            })
    }

    /// Returns type of request argument.
    pub fn request_type(&self) -> syn::Type {
        match self.request_arg() {
            syn::FnArg::Receiver(tk) => {
                diagnostic::emit_error(
                    tk.self_token.span(),
                    "`self` is not allowed as service argument",
                );
                syn::parse_str("()").unwrap()
            }
            syn::FnArg::Typed(typed) => typed.ty.deref().clone(),
        }
    }

    /// Returns the type of the response. Will extract the inner type in case of an `Result`
    pub fn response_type(&self) -> syn::Type {
        match self.signature.output {
            ReturnType::Default => syn::parse_str("()").unwrap(),
            ReturnType::Type(_, ref ty) => match ty.deref() {
                syn::Type::Path(p) => {
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
                        ty.clone()
                    } else {
                        ty.deref().clone()
                    }
                }
                _ => ty.deref().clone(),
            },
        }
    }
}

impl Parse for Declaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            pub_token: input.parse()?,
            signature: input.parse()?,
            block: input.parse()?,
        })
    }
}
