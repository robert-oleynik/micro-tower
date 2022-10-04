use futures::FutureExt;
use std::future::Future;
use std::marker::PhantomData;
use std::task::{Context, Poll};
use tokio::task::{JoinError, JoinHandle};
use tower::balance::p2c::Balance;
use tower::discover::ServiceList;
use tower::{BoxError, Service, ServiceExt};

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

enum CreateFuture<MS, Req>
where
    MS: Service<()>,
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
pub struct Pool<MS, Req>
where
    MS: Service<()>,
    MS::Response: tower::Service<Req, Error = BoxError>,
{
    services: CreateFuture<MS, Req>,
}

pub struct Layer<Req> {
    size: usize,
    _p: PhantomData<Req>,
}

impl<MS, Req> Pool<MS, Req>
where
    Req: Send + 'static,
    MS: Service<()> + Send + 'static,
    MS::Response: tower::Service<Req, Error = BoxError> + Send,
    MS::Error: Send,
    MS::Future: Send,
{
    /// Create new pool with `count` many services and `make_service` to create the inner services.
    pub fn with_size(size: usize, mut make_service: MS) -> Self {
        tracing::debug!(message = "creating service pool", size);
        let handle = tokio::spawn(async move {
            let mut services = Vec::with_capacity(size);
            for i in 0..size {
                let service = make_service.ready().await?.call(()).await?;
                tracing::trace!(message = "created pooled service", i);
                services.push(service);
            }
            Ok(Balance::new(ServiceList::new(services)))
        });

        Self {
            services: CreateFuture::Pending { handle },
        }
    }
}

impl<MS, Req> tower::Service<Req> for Pool<MS, Req>
where
    MS: Service<()>,
    MS::Response: tower::Service<Req, Error = BoxError> + tower::load::Load,
    MS::Error: std::error::Error + Send + Sync + 'static,
    <MS::Response as tower::load::Load>::Metric: std::fmt::Debug,
{
    type Response = <MS::Response as tower::Service<Req>>::Response;
    type Error = BoxError;
    type Future = impl Future<Output = Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.services {
            CreateFuture::Pending { ref mut handle } => match handle.poll_unpin(cx) {
                Poll::Ready(Ok(Ok(services))) => {
                    self.services = CreateFuture::Ready { services };
                    Poll::Ready(Ok(()))
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
            CreateFuture::Ready { ref mut services } => services.call(req),
            _ => unimplemented!("called before ready"),
        }
    }
}

impl<Req> Layer<Req> {
    #[must_use]
    pub fn with_size(size: usize) -> Self {
        Self {
            size,
            _p: PhantomData,
        }
    }
}

impl<MS, Req> tower::Layer<MS> for Layer<Req>
where
    Req: Send + 'static,
    MS: Service<()> + Send + 'static,
    MS::Response: tower::Service<Req, Error = BoxError> + Send,
    MS::Error: Send,
    MS::Future: Send,
{
    type Service = Pool<MS, Req>;

    fn layer(&self, inner: MS) -> Self::Service {
        Pool::with_size(self.size, inner)
    }
}
