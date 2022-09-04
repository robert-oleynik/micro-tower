use proc_macro::{Diagnostic, Level};
use syn::{parse::Parse, GenericArgument, PathSegment};

use self::signature::Signature;

mod args;
mod signature;

/// Parse proc attribute items.
pub struct Items {
    signature: Signature,
    block: Box<syn::Block>,
}

pub struct Service {
    pub args: args::Args,
    pub pub_token: Option<syn::token::Pub>,
    pub name: syn::Ident,
    pub request: Box<syn::Type>,
    pub request_arg: syn::FnArg,
    pub output: syn::ReturnType,
    pub response: Option<syn::Type>,
    pub response_result: bool,
    pub service_dependencies: Vec<syn::FnArg>,
    pub code_block: Box<syn::Block>,
}

fn infer_result_ok_type(seg: &PathSegment) -> Option<syn::Type> {
    match &seg.arguments {
        syn::PathArguments::AngleBracketed(args)
            if args.args.len() == 1 || args.args.len() == 2 =>
        {
            if let GenericArgument::Type(ty) = args.args.first().unwrap() {
                return Some(ty.clone());
            }
            Diagnostic::spanned(
                vec![seg.ident.span().unwrap()],
                Level::Error,
                "Expected generic type as first parameter",
            )
            .emit();
            None
        }
        syn::PathArguments::AngleBracketed(_) | syn::PathArguments::None => {
            Diagnostic::spanned(
                vec![seg.ident.span().unwrap()],
                Level::Error,
                "Cannot infer result ok type.",
            )
            .emit();
            None
        }
        _ => unimplemented!(),
    }
}

impl Service {
    pub fn new(args: syn::AttributeArgs, items: Items) -> syn::Result<Self> {
        let args = args::Args::try_from(args)?;
        let name = items.signature.ident;
        let request_arg = match items.signature.inputs.first() {
            Some(input) => input.clone(),
            None => {
                Diagnostic::spanned(
                    vec![items.signature.paren_token.span.unwrap()],
                    Level::Error,
                    "Missing request parameter",
                )
                .emit();
                panic!()
            }
        };
        let request = match &request_arg {
            syn::FnArg::Typed(ty) => ty.ty.clone(),
            syn::FnArg::Receiver(recv) => {
                Diagnostic::spanned(
                    vec![recv.self_token.span.unwrap()],
                    Level::Error,
                    "`self` is not allowed in this context",
                )
                .emit();
                panic!()
            }
        };
        let output = items.signature.output.clone();
        let (response, response_result) = match items.signature.output {
            syn::ReturnType::Default => (None, false),
            syn::ReturnType::Type(_, ty) => {
                match *ty {
                    syn::Type::Infer(ref t) => {
                        Diagnostic::spanned(
                            vec![t.underscore_token.span.unwrap()],
                            Level::Error,
                            "Response type must be specified explicitly.",
                        )
                        .emit();
                    }
                    syn::Type::ImplTrait(ref t) => {
                        Diagnostic::spanned(
                            vec![t.impl_token.span.unwrap()],
                            Level::Error,
                            "`impl` is not allowed in this context",
                        )
                        .emit();
                    }
                    syn::Type::Never(ref t) => {
                        Diagnostic::spanned(
                            vec![t.bang_token.span.unwrap()],
                            Level::Error,
                            "`!` is not allowed in this context",
                        )
                        .emit();
                    }
                    _ => {}
                };

                if let syn::Type::Path(syn::TypePath { ref path, .. }) = *ty {
                    if let Some(last) = path.segments.last() {
                        if last.ident == "Result" {
                            let result = infer_result_ok_type(last);
                            (result, true)
                        } else {
                            (None, false)
                        }
                    } else {
                        (Some((*ty).clone()), false)
                    }
                } else {
                    (Some((*ty).clone()), false)
                }
            }
        };
        let service_dependencies = items
            .signature
            .inputs
            .into_iter()
            .skip(1)
            .collect::<Vec<_>>();
        let code_block = items.block;

        Ok(Self {
            args,
            name,
            pub_token: items.signature.pub_token,
            request_arg,
            request,
            response,
            response_result,
            output,
            service_dependencies,
            code_block,
        })
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
