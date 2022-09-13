use super::Error;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct Service<R, S> {
    inner: S,
    _p: PhantomData<R>,
}

pub struct Layer<R, S> {
    _p: PhantomData<(R, S)>,
}

impl<R, S> Default for Layer<R, S> {
    fn default() -> Self {
        Self { _p: PhantomData }
    }
}

impl<R, S> tower::Layer<S> for Layer<R, S> {
    type Service = Service<R, S>;

    fn layer(&self, inner: S) -> Self::Service {
        Service {
            inner,
            _p: PhantomData,
        }
    }
}

impl<R, S: Clone> Clone for Service<R, S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _p: PhantomData,
        }
    }
}

impl<R, S: tower::Service<R, Error = tower::BoxError>> tower::Service<R> for Service<R, S>
where
    S::Future: Send + 'static,
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
