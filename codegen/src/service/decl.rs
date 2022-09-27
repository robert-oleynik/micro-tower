use std::ops::Deref;

use quote::__private::Span;
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::ReturnType;

use crate::util::diagnostic;

pub struct Declaration {
    attr: Vec<syn::Attribute>,
    docs: Vec<syn::Attribute>,
    vis: syn::Visibility,
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
    pub fn visibility(&self) -> &syn::Visibility {
        &self.vis
    }

    /// Returns reference to service's request argument used by the service implementation.
    pub fn request_arg(&self) -> syn::FnArg {
        self.signature.inputs.first().map_or_else(
            || {
                diagnostic::emit_error(Span::call_site(), "Missing request argument");
                syn::parse_str("_: ()").unwrap()
            },
            syn::FnArg::clone,
        )
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
    pub fn response_type(&self) -> (bool, syn::Type) {
        match self.signature.output {
            ReturnType::Default => (false, syn::parse_str("()").unwrap()),
            ReturnType::Type(_, ref ty) => match **ty {
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
        }
    }

    fn service_args(&self) -> impl Iterator<Item = &syn::PatType> + '_ {
        self.signature
            .inputs
            .iter()
            .skip(1)
            .filter_map(|arg| match arg {
                syn::FnArg::Receiver(recv) => {
                    diagnostic::emit_error(recv.span(), "`self` is not allowed in this context");
                    None
                }
                syn::FnArg::Typed(ty) => Some(ty),
            })
    }

    pub fn service_names(&self) -> impl Iterator<Item = syn::Ident> + '_ {
        self.service_args()
            .filter_map(|arg| match arg.pat.as_ref() {
                syn::Pat::Ident(id) => Some(id.ident.clone()),
                syn::Pat::Wild(_) => Some(syn::parse_str("_").unwrap()),
                _ => None,
            })
    }

    pub fn service_mut(&self) -> impl Iterator<Item = Option<syn::token::Mut>> + '_ {
        self.service_args()
            .filter_map(|arg| match arg.pat.as_ref() {
                syn::Pat::Ident(id) => Some(id.mutability),
                _ => None,
            })
    }

    pub fn service_types(&self) -> impl Iterator<Item = &'_ syn::Type> {
        self.service_args().map(|arg| arg.ty.as_ref())
    }

    pub fn block(&self) -> &syn::Block {
        &self.block
    }

    pub fn attributes(&self) -> &[syn::Attribute] {
        &self.attr[..]
    }

    pub fn docs(&self) -> &[syn::Attribute] {
        &self.docs[..]
    }

    pub fn output(&self) -> syn::Type {
        match &self.signature.output {
            ReturnType::Default => syn::parse_str("()").unwrap(),
            ReturnType::Type(_, ty) => ty.deref().clone(),
        }
    }
}

impl Parse for Declaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item: syn::ItemFn = input.parse()?;
        let docs = item
            .attrs
            .iter()
            .filter(|attr| attr.path.is_ident("doc"))
            .cloned()
            .collect();
        let attr = item
            .attrs
            .into_iter()
            .filter(|attr| match &attr.path {
                p if p.is_ident("derive") => {
                    diagnostic::emit_error(attr.span(), "Service cannot be derived");
                    false
                }
                p if p.is_ident("doc") => false,
                _ => true,
            })
            .collect();
        let vis = item.vis;
        let signature = item.sig;
        let block = item.block;
        Ok(Self {
            attr,
            docs,
            vis,
            signature,
            block,
        })
    }
}
