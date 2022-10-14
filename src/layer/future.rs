use crate::util::BoxFuture;
use std::task::{Context, Poll};

/// Wraps an inner service and `Box` the used future.
#[derive(Default)]
pub struct BoxLayer;

#[derive(Debug, Clone)]
pub struct Service<S> {
	inner: S,
}

impl<S> tower::Layer<S> for BoxLayer {
	type Service = Service<S>;

	fn layer(&self, inner: S) -> Self::Service {
		Service { inner }
	}
}

impl<Req, S> tower::Service<Req> for Service<S>
where
	S: tower::Service<Req>,
	S::Future: Send + 'static,
{
	type Response = S::Response;
	type Error = S::Error;
	type Future = BoxFuture<Result<Self::Response, Self::Error>>;

	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx)
	}

	fn call(&mut self, req: Req) -> Self::Future {
		Box::pin(self.inner.call(req))
	}
}
