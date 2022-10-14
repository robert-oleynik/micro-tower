use futures::FutureExt;
use std::marker::PhantomData;
use std::task::{Context, Poll};
use tokio::task::{JoinError, JoinHandle};
use tower::balance::p2c::Balance;
use tower::discover::ServiceList;
use tower::{BoxError, Service, ServiceExt};

use crate::util::BoxFuture;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create service pool")]
    Create(
        #[from]
        #[source]
        BoxError,
    ),
    #[error("failed to join created services pool")]
    CreateJoin(#[source] JoinError),
    #[error("failed to create service pool")]
    Failed,
}

type CreateHandle<T, E> = JoinHandle<Result<T, E>>;
type ServiceSet<S, Req> = Balance<ServiceList<Vec<S>>, Req>;

enum CreateFuture<MS, Target, Req>
where
    MS: Service<Target>,
    MS::Response: tower::Service<Req, Error = BoxError>,
{
    Pending {
        handle: CreateHandle<ServiceSet<MS::Response, Req>, MS::Error>,
    },
    Ready {
        services: ServiceSet<MS::Response, Req>,
    },
    Failed,
}

/// Balances requests between multiple services.
pub struct Pool<MS, Target, Req>
where
    MS: Service<Target>,
    MS::Response: tower::Service<Req, Error = BoxError>,
{
    services: CreateFuture<MS, Target, Req>,
    _p: PhantomData<Target>,
}

pub struct Layer<Target, Req> {
    size: usize,
    target: Target,
    _p: PhantomData<Req>,
}

impl<MS, Target, Req> Pool<MS, Target, Req>
where
    Target: Clone + Send + 'static,
    Req: Send + 'static,
    MS: Service<Target> + Send + 'static,
    MS::Response: tower::Service<Req, Error = BoxError> + Send,
    MS::Error: Send,
    MS::Future: Send,
{
    /// Create new pool with `count` many services and `make_service` to create the inner services.
    pub fn with_size(size: usize, mut make_service: MS, target: Target) -> Self {
        tracing::debug!(message = "creating service pool", size);
        let handle = tokio::spawn(async move {
            let mut services = Vec::with_capacity(size);
            for _ in 0..size {
                let target = target.clone();
                let service = make_service.ready().await?.call(target).await?;
                services.push(service);
            }
            tracing::debug!(message = "service pool created", size);
            Ok(Balance::new(ServiceList::new(services)))
        });

        Self {
            services: CreateFuture::Pending { handle },
            _p: PhantomData,
        }
    }
}

impl<MS, Target, Req> tower::Service<Req> for Pool<MS, Target, Req>
where
    MS: Service<Target>,
    MS::Response: tower::Service<Req, Error = BoxError> + tower::load::Load,
    MS::Error: Into<BoxError>,
    <MS::Response as tower::Service<Req>>::Future: Send + 'static,
    <MS::Response as tower::load::Load>::Metric: std::fmt::Debug,
{
    type Response = <MS::Response as tower::Service<Req>>::Response;
    type Error = BoxError;
    type Future = BoxFuture<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.services {
            CreateFuture::Pending { ref mut handle } => match handle.poll_unpin(cx) {
                Poll::Ready(Ok(Ok(services))) => {
                    self.services = CreateFuture::Ready { services };
                    self.poll_ready(cx)
                }
                Poll::Ready(Ok(Err(err))) => {
                    self.services = CreateFuture::Failed;
                    Poll::Ready(Err(err.into()))
                }
                Poll::Ready(Err(err)) => {
                    self.services = CreateFuture::Failed;
                    Poll::Ready(Err(err.into()))
                }
                Poll::Pending => Poll::Pending,
            },
            CreateFuture::Ready { ref mut services } => services.poll_ready(cx),
            CreateFuture::Failed => Poll::Ready(Err(Error::Failed.into())),
        }
    }

    fn call(&mut self, req: Req) -> Self::Future {
        match self.services {
            CreateFuture::Ready { ref mut services } => Box::pin(services.call(req)),
            _ => unimplemented!("called before ready"),
        }
    }
}

impl<Target, Req> Layer<Target, Req> {
    #[must_use]
    pub fn with_size(size: usize, target: Target) -> Self {
        Self {
            size,
            target,
            _p: PhantomData,
        }
    }
}

impl<MS, Target, Req> tower::Layer<MS> for Layer<Target, Req>
where
    Req: Send + 'static,
    MS: Service<Target> + Send + 'static,
    MS::Response: tower::Service<Req, Error = BoxError> + Send,
    MS::Error: Send,
    MS::Future: Send,
    Target: Clone + Send + 'static,
{
    type Service = Pool<MS, Target, Req>;

    fn layer(&self, inner: MS) -> Self::Service {
        let target = self.target.clone();
        Pool::with_size(self.size, inner, target)
    }
}
