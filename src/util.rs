pub mod borrow;

use std::future::Future;
use std::pin::Pin;
pub use tower::BoxError;
use tower::Service;

pub type BoxService<R, S> = Box<
	dyn Service<
			R,
			Response = <S as Service<R>>::Response,
			Future = BoxFuture<Result<<S as Service<R>>::Response, BoxError>>,
			Error = BoxError,
		> + Send,
>;

pub type BoxCloneService<R, S> =
	tower::util::BoxCloneService<R, <S as Service<R>>::Response, BoxError>;

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
