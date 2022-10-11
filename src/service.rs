//! Reexport [`tower`] utilities.

pub mod pool;

use std::task::Poll;

pub use tower::*;

use crate::util::BoxFuture;

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

/// A wrapper around a boxed service using [`Info`] to describe request and response type. Can be
/// used to wrap any service which accepts the same request and response type as `S`
pub struct Box<S: Info> {
    inner: std::boxed::Box<
        dyn tower::Service<
                S::Request,
                Response = S::Response,
                Error = BoxError,
                Future = BoxFuture<Result<S::Response, BoxError>>,
            > + Send
            + Sync,
    >,
}

impl<S> From<std::boxed::Box<S>> for Box<S>
where
    S: Info + Send + Sync + 'static,
    S: tower::Service<
        S::Request,
        Response = <S as Info>::Response,
        Error = BoxError,
        Future = BoxFuture<Result<<S as Info>::Response, BoxError>>,
    >,
{
    fn from(inner: std::boxed::Box<S>) -> Self {
        Self { inner }
    }
}

impl<S: Info> Info for Box<S> {
    type Request = S::Request;
    type Response = S::Response;

    fn name() -> &'static str {
        S::name()
    }
}

impl<S: Info> tower::Service<S::Request> for Box<S> {
    type Response = S::Response;
    type Error = BoxError;
    type Future = BoxFuture<Result<S::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: S::Request) -> Self::Future {
        self.inner.call(req).into()
    }
}
