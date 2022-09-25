pub mod borrow;

use std::future::Future;
use std::pin::Pin;

pub use tower::BoxError;

pub type BoxService<R, S> = tower::util::BoxService<R, S, tower::BoxError>;
pub type BoxFuture<O> = Pin<Box<dyn Future<Output = O> + Send>>;

/// Generates a [`std::error::Report`] with `pretty` and `backtrace` enabled
#[macro_export]
macro_rules! report {
    ($err:expr) => {
        ::std::error::Report::new($err)
            .pretty(true)
            .show_backtrace(true)
    };
}
