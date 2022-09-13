use std::ops::{Deref, DerefMut};

mod error;
pub mod map_error;

use crate::util::Buildable;
pub use error::Error;

/// Wrapper around services.
///
/// # Usage
///
/// ```rust
/// #[micro_tower::codegen::service]
/// async fn hello_world(_: ()) -> &'static str {
///     "Hello, World!"
/// }
///
/// #[micro_tower::codegen::service]
/// async fn hello_world2(_: (), other: Service<hello_world>) -> &'static str {
///     let result = other.ready().await.call(()).await.unwrap();
///     result
/// }
/// ```
#[derive(Clone)]
pub struct Service<S: Buildable> {
    inner: S::Target,
}

impl<S: Buildable> Service<S> {
    pub fn from_service(inner: S::Target) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> S::Target {
        self.inner
    }
}

impl<S: Buildable> Buildable for Service<S> {
    type Target = S::Target;
    type Builder = S::Builder;

    fn builder() -> Self::Builder {
        S::builder()
    }
}

impl<S: Buildable> Deref for Service<S> {
    type Target = <S as Buildable>::Target;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S: Buildable> DerefMut for Service<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
