use syn::Lit;

pub fn lit_type_as_string(lit: &Lit) -> &'static str {
    match lit {
        Lit::Str(_) => "string",
        Lit::ByteStr(_) => "byte string",
        Lit::Byte(_) => "byte",
        Lit::Char(_) => "char",
        Lit::Int(_) => "int",
        Lit::Float(_) => "float",
        Lit::Bool(_) => "bool",
        Lit::Verbatim(_) => todo!(),
    }
}

macro_rules! diagnostic {
    (warn at [$( $lit:expr ),*], $tokens:tt) => {
        ::proc_macro::Diagnostic::spanned(vec![$($lit),*], ::proc_macro::Level::Warn, format!($tokens)).emit()
    };
    (error at [$( $lit:expr ),*], $tokens:tt) => {
        ::proc_macro::Diagnostic::spanned(vec![$($lit),*], ::proc_macro::Level::Error, format!($tokens)).emit()
    };
}
