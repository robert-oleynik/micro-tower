use micro_tower_codegen_macros::diagnostic;
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::FnArg;
use syn::{GenericArgument, PathArguments};

use super::signature::Signature;

/// Parse proc attribute items.
pub struct Items {
    signature: Signature,
    block: Box<syn::Block>,
}

fn type_names(ty: &syn::Type) -> &'static str {
    match ty {
        syn::Type::Array(_) => "arrays",
        syn::Type::BareFn(_) => "bare function",
        syn::Type::Group(_) => "group",
        syn::Type::ImplTrait(_) => "impl trait",
        syn::Type::Infer(_) => "type infer",
        syn::Type::Macro(_) => "macro",
        syn::Type::Never(_) => "return never",
        syn::Type::Paren(_) => "parenthized type",
        syn::Type::Path(_) => "path",
        syn::Type::Ptr(_) => "pointer",
        syn::Type::Reference(_) => "reference",
        syn::Type::Slice(_) => "slice",
        syn::Type::TraitObject(_) => "trait object",
        syn::Type::Tuple(_) => "tuple",
        syn::Type::Verbatim(_) => "verbatim",
        _ => unimplemented!(),
    }
}

impl Items {
    /// Returns name of the service.
    pub fn name(&self) -> &syn::Ident {
        &self.signature.ident
    }

    /// Returns a reference to the `pub` token (if exists).
    pub fn pub_token(&self) -> Option<&syn::token::Pub> {
        self.signature.pub_token.as_ref()
    }

    /// Returns a iterator other all service inputs.
    pub fn inputs(&self) -> impl Iterator<Item = &syn::FnArg> {
        self.signature.inputs.iter()
    }

    /// Returns a reference to service's output type.
    pub fn output(&self) -> &syn::ReturnType {
        &self.signature.output
    }

    /// Returns a reference to the block.
    pub fn block(&self) -> &syn::Block {
        self.block.as_ref()
    }

    /// Returns service's request type.
    ///
    /// # Compiler Errors
    ///
    /// Will emit an compiler error if `self` is used in service signature at first place.
    pub fn request_type(&self) -> syn::Type {
        if let Some(arg) = self.signature.inputs.first() {
            match arg {
                FnArg::Receiver(recv) => {
                    diagnostic!(error at [recv.self_token.span().unwrap()], "`self` is not allowed in this context");
                    syn::parse_str("()").unwrap()
                }
                FnArg::Typed(ty) => (*ty.ty).clone(),
            }
        } else {
            diagnostic!(error at [self.signature.inputs.span().unwrap()], "No request type specified");
            syn::parse_str("()").unwrap()
        }
    }

    /// Returns service's error type. If no error type was found returns
    /// [`::std::convert::Infallible`].
    ///
    /// # Compile Errors
    ///
    /// Will emit compile errors in the same cases as [`Self::response_type`].
    pub fn error_type(&self) -> syn::Type {
        let infallible = syn::parse_str("::std::convert::Infallible").unwrap();
        match &self.signature.output {
            syn::ReturnType::Default => {}
            syn::ReturnType::Type(_, ty) => match ty.as_ref() {
                syn::Type::Path(p) => {
                    let result = p
                            .path
                            .segments
                            .iter()
                            .last()
                            .and_then(|seg| {
                                if seg.ident == "Result" {
                                    return Some(seg);
                                }
                                None
                            })
                            .and_then(|seg| {
                                if let PathArguments::AngleBracketed(args) = &seg.arguments {
                                    if let Some(GenericArgument::Type(ty)) = args.args.iter().nth(1)
                                    {
                                        return Some(ty.clone());
                                    }
                                    diagnostic!(warn at [args.span().unwrap()], "Couldn't infer error type");
                                }
                                None
                            });

                    if let Some(ty) = result {
                        return ty;
                    }
                }
                syn::Type::Infer(_) => {
                    diagnostic!(error at [ty.span().unwrap()], "Response type must be explicitly specified");
                }
                syn::Type::Macro(_) => {
                    diagnostic!(error at [ty.span().unwrap()], "Cannot infer reponse type of macros");
                }
                syn::Type::Never(_) => {
                    diagnostic!(error at [ty.span().unwrap()], "Service must return a response");
                }
                _ => {
                    let name = type_names(ty.as_ref());
                    diagnostic!(error at [ty.span().unwrap()], "{name} is not allowed in this context");
                }
            },
        }
        infallible
    }

    /// Returns the response type of the services described by this items and if the return type
    /// was extracted from result type.
    ///
    /// The response type will determined by:
    ///
    /// 1. In case of any type named `Result` with first param as type param `T` will return `T` or
    /// 2. If non of the above cases applies the output type will be used (see below).
    ///
    /// ```rust
    /// #[micro_tower::codegen::service]
    /// async fn service(_: ()) -> T {
    ///     // ..
    /// }
    /// ```
    ///
    /// # Compiler Errors
    ///
    /// Will emit compiler errors if return type is:
    ///
    /// - Array of type `[T; n]`
    /// - Bare function like `fn(A) -> B`
    /// - impl trait like `impl Trait`
    /// - infer/never type: `_`/`!`
    /// - macro type
    /// - Parenthesized type `(...)`
    /// - Pointer like `*const T`
    /// - Reference like `&mut T`
    /// - Slice `[T]`
    /// - Tuple `(A, B)`
    ///
    /// This function will return type `()` in case of an error.
    pub fn response_type(&self) -> (syn::Type, bool) {
        let def = syn::parse_str("()").unwrap();
        match &self.signature.output {
            syn::ReturnType::Default => (def, false),
            syn::ReturnType::Type(_, ty) => {
                match ty.as_ref() {
                    syn::Type::Path(p) => {
                        let result = p
                            .path
                            .segments
                            .iter()
                            .last()
                            .and_then(|seg| {
                                if seg.ident == "Result" {
                                    return Some(seg);
                                }
                                None
                            })
                            .and_then(|seg| {
                                if let PathArguments::AngleBracketed(args) = &seg.arguments {
                                    if let Some(GenericArgument::Type(ty)) = args.args.first() {
                                        return Some(ty.clone());
                                    }
                                    diagnostic!(warn at [args.span().unwrap()], "Couldn't infer return type");
                                }
                                None
                            });

                        if let Some(ty) = result {
                            return (ty, true);
                        }
                        return ((**ty).clone(), false);
                    }
                    syn::Type::Infer(_) => {
                        diagnostic!(error at [ty.span().unwrap()], "Response type must be explicitly specified");
                    }
                    syn::Type::Macro(_) => {
                        diagnostic!(error at [ty.span().unwrap()], "Cannot infer reponse type of macros");
                    }
                    syn::Type::Never(_) => {
                        diagnostic!(error at [ty.span().unwrap()], "Service must return a response");
                    }
                    _ => {
                        let name = type_names(ty.as_ref());
                        diagnostic!(error at [ty.span().unwrap()], "{name} is not allowed in this context");
                    }
                }
                (def, false)
            }
        }
    }

    /// List all service by its argument types. This function will remove all parameters containing
    /// `self` and emit errors for each of them.
    ///
    /// # Compiler Errors
    ///
    /// Will emit a compiler error for each use of `self` inside of service signature.
    pub fn services(&self) -> impl Iterator<Item = &'_ syn::PatType> {
        self.signature
            .inputs
            .iter()
            .skip(1)
            .filter_map(|arg| match arg {
                FnArg::Receiver(recv) => {
                    diagnostic!(error at [recv.self_token.span().unwrap()], "`self` is not allowed in this context");
                    None
                }
                FnArg::Typed(ty) => {
                    Some(ty)
                }
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
