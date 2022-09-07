#[macro_export]
macro_rules! diagnostic {
    (warn at [$( $lit:expr ),*], $tokens:tt) => {
        ::proc_macro::Diagnostic::spanned(vec![$($lit),*], ::proc_macro::Level::Warning, format!($tokens)).emit()
    };
    (error at [$( $lit:expr ),*], $tokens:tt) => {
        ::proc_macro::Diagnostic::spanned(vec![$($lit),*], ::proc_macro::Level::Error, format!($tokens)).emit()
    };
}
