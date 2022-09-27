use proc_macro::{Diagnostic, Level};
use quote::__private::Span;

/// Generate compile error which is associated with `span` and contains the message `msg`.
pub fn emit_error(span: Span, msg: impl Into<String>) {
    Diagnostic::spanned(vec![span.unwrap()], Level::Error, msg).emit();
}

/// Generate a compile time warning which is associated with `span` and contains the message `msg`.
pub fn emit_warning(span: Span, msg: impl Into<String>) {
    Diagnostic::spanned(vec![span.unwrap()], Level::Warning, msg).emit();
}
