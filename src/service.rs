use std::future::Future;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

mod error;

use crate::util::Buildable;
pub use error::Error;
pub struct MapErrorService<R, S> {
    inner: S,
    _p: PhantomData<R>,
}

pub struct MapErrorLayer<R, S> {
    _p: PhantomData<(R, S)>,
}

impl<R, S> Default for MapErrorLayer<R, S> {
    fn default() -> Self {
        Self { _p: PhantomData }
    }
}

impl<R, S> tower::Layer<S> for MapErrorLayer<R, S> {
    type Service = MapErrorService<R, S>;

    fn layer(&self, inner: S) -> Self::Service {
        MapErrorService {
            inner,
            _p: PhantomData,
        }
    }
}

impl<R, S: Clone> Clone for MapErrorService<R, S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _p: PhantomData,
        }
    }
}

impl<R, S: tower::Service<R, Error = tower::BoxError>> tower::Service<R> for MapErrorService<R, S>
where
    S::Future: Send + 'static,
    S::Error: std::error::Error,
{
    type Response = S::Response;
    type Error = crate::service::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|err| Error::from(err))
    }

    fn call(&mut self, req: R) -> Self::Future {
        let fut = self.inner.call(req);

        Box::pin(async move {
            match fut.await {
                Ok(result) => Ok(result),
                Err(err) => Err(Error::from(err)),
            }
        })
    }
}

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
