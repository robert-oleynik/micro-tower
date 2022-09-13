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

impl Items {
    /// Returns the response type of the services described by this items.
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
    fn response_type(&self) -> syn::Type {
        let def = syn::parse_str("()").unwrap();
        match self.signature.output {
            syn::ReturnType::Default => syn::parse_str("()").unwrap(),
            syn::ReturnType::Type(rarrow_token, ty) => {
                let rarrow_token = rarrow_token.clone();
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
                                if let PathArguments::AngleBracketed(args) = seg.arguments {
                                    if let Some(GenericArgument::Type(ty)) = args.args.first() {
                                        return Some(ty.clone());
                                    }
                                }
                                None
                            });

                        if let Some(ty) = result {
                            return ty;
                        }
                        (*ty).clone()
                    }
                    syn::Type::Array(_) => {
                        diagnostic!(error at [ty.span().unwrap()], "Arrays are not allowed as response");
                        def
                    }
                    syn::Type::BareFn(_) => {
                        diagnostic!(error at [ty.span().unwrap()], "Bare functions are not allowed as response");
                        def
                    }
                    syn::Type::Group(_) => {
                        diagnostic!(error at [ty.span().unwrap()], "Groups are not allowed as response type");
                        def
                    }
                    syn::Type::ImplTrait(_) => {
                        diagnostic!(error at [ty.span().unwrap()], "Impl traits are not allowed as response type");
                        def
                    }
                    syn::Type::Infer(_) => {
                        diagnostic!(error at [ty.span().unwrap()], "Response type must be explicitly specified");
                        def
                    }
                    syn::Type::Macro(_) => {
                        diagnostic!(error at [ty.span().unwrap()], "Cannot infer reponse type of macros");
                        def
                    }
                    syn::Type::Never(_) => {
                        diagnostic!(error at [ty.span().unwrap()], "Service must return a response");
                        def
                    }
                    syn::Type::Paren(_) => {
                        diagnostic!(error at [ty.span().unwrap()], "Parenthesized types are not allowed as response");
                        def
                    }
                    syn::Type::Ptr(_) => {
                        diagnostic!(error at [ty.span().unwrap()], "Pointers are not allowed as response");
                        def
                    }
                    syn::Type::Reference(_) => {
                        diagnostic!(error at [ty.span().unwrap()], "References are not allowed as response");
                        def
                    }
                    syn::Type::Slice(_) => {
                        diagnostic!(error at [ty.span().unwrap()], "Slices are not allowed as response");
                        def
                    }
                    syn::Type::TraitObject(_) => {
                        diagnostic!(error at [ty.span().unwrap()], "Dynamic types are not allowed as response");
                        def
                    }
                    syn::Type::Tuple(_) => {
                        diagnostic!(error at [ty.span().unwrap()], "Tuples are not allowed as response");
                        def
                    }
                    _ => unimplemented!(),
                }
            }
        }
    }

    /// List all service by its argument types. This function will remove all parameters containing
    /// `self` and emit warnings for each of them.
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
