//! Reexport [`tower`] utilities.

pub mod pool;

use crate::runtime::registry;
use crate::util::BoxFuture;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};
pub use tower::*;

#[derive(Debug, thiserror::Error)]
#[error("service `{0}` not ready")]

pub struct NotReady(pub &'static str);

/// Interface to obtain information about services. Used at compile and runtime time.
pub trait Info {
	/// Request type of service.
	type Request;
	/// Response type of service.
	type Response;

	/// Returns generic service name. Used to identify a service and should be unique other all
	/// services.
	fn name() -> &'static str;
}

/// Interface used to create service with request and response specified as in [`Info`].
pub trait Create: Sized + Info {
	type Error;

	/// Create service by reading required services from `registry`. Will return `Ok(None)` if not all services
	///
	/// # Errors
	///
	/// Will return `Err` if service is of the wrong type.
	fn with_registry(
		registry: Arc<RwLock<registry::Type>>,
	) -> Result<Option<Service<Self>>, Self::Error>;
}

#[doc(hidden)]
trait Clonable<Req> {
	type Response;

	fn clone(&self) -> Box<dyn Clonable<Req, Response = Self::Response> + Send + Sync>;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), BoxError>>;
	fn call(&mut self, req: Req) -> BoxFuture<Result<Self::Response, BoxError>>;
}

type BoxServiceClone<S> =
	Box<dyn Clonable<<S as Info>::Request, Response = <S as Info>::Response> + Send + Sync>;

impl<S, Req> Clonable<Req> for S
where
	S: tower::Service<Req, Error = BoxError> + Clone + Send + Sync + 'static,
	S::Future: Into<BoxFuture<Result<S::Response, BoxError>>>,
{
	type Response = S::Response;

	fn clone(&self) -> Box<dyn Clonable<Req, Response = Self::Response> + Send + Sync> {
		Box::new(<S as Clone>::clone(self))
	}

	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), BoxError>> {
		<Self as tower::Service<Req>>::poll_ready(self, cx)
	}

	fn call(&mut self, req: Req) -> BoxFuture<Result<Self::Response, BoxError>> {
		<Self as tower::Service<Req>>::call(self, req).into()
	}
}

/// A wrapper around a boxed service using [`Info`] to describe request and response type. Can be
/// used to wrap any service which accepts the same request and response type as `S`
pub struct Service<S: Info> {
	inner: BoxServiceClone<S>,
}

impl<S: Info, T> From<Box<T>> for Service<S>
where
	T: tower::Service<
			S::Request,
			Response = S::Response,
			Error = BoxError,
			Future = BoxFuture<Result<S::Response, BoxError>>,
		> + Clone
		+ Sync
		+ Send
		+ 'static,
{
	fn from(inner: Box<T>) -> Self {
		Self { inner }
	}
}

impl<S: Info> From<BoxServiceClone<S>> for Service<S> {
	fn from(inner: BoxServiceClone<S>) -> Self {
		Self { inner }
	}
}

impl<S> Clone for Service<S>
where
	S: Info + Send + Sync + 'static,
	S: tower::Service<
		S::Request,
		Response = <S as Info>::Response,
		Error = BoxError,
		Future = BoxFuture<Result<<S as Info>::Response, BoxError>>,
	>,
{
	fn clone(&self) -> Self {
		Self {
			inner: self.inner.clone(),
		}
	}
}

impl<S: Info> Info for Service<S> {
	type Request = S::Request;
	type Response = S::Response;

	fn name() -> &'static str {
		S::name()
	}
}

impl<S: Info> tower::Service<S::Request> for Service<S> {
	type Response = S::Response;
	type Error = BoxError;
	type Future = BoxFuture<Result<S::Response, Self::Error>>;

	fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx)
	}

	fn call(&mut self, req: S::Request) -> Self::Future {
		self.inner.call(req)
	}
}
