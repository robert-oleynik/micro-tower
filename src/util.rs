pub mod borrow;

use std::future::Future;
use std::pin::Pin;

pub use tower::BoxError;

pub type BoxService<R, S> = tower::util::BoxService<R, S, tower::BoxError>;
pub type BoxFuture<O> = Pin<Box<dyn Future<Output = O> + Send>>;
