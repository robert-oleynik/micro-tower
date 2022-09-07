use std::ops::{Deref, DerefMut};

use crate::utils::Buildable;

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
pub struct Service<S> {
    inner: S,
}

impl<S> Service<S> {
    pub fn from_service(inner: S) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S: Buildable> Buildable for Service<S> {
    type Builder = S::Builder;

    fn builder() -> Self::Builder {
        S::builder()
    }
}

impl<S> Deref for Service<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S> DerefMut for Service<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
