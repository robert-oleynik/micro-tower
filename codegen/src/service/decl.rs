use syn::{parse::Parse, spanned::Spanned};

use crate::util::diagnostic;

pub struct Declaration {
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
}

impl Parse for Declaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            signature: input.parse()?,
            block: input.parse()?,
        })
    }
}
