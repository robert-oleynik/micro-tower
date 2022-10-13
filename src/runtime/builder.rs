use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use tokio::task::JoinHandle;
use tower::{util::BoxCloneService, BoxError, ServiceBuilder};

use crate::{
    service::{Create, Info, NotReady, Service},
    session::Session,
    shutdown::Controller,
    util::BoxFuture,
};

use super::{registry, Runtime};

/// Implements builder pattern used to generate runtime.
///
/// # Usage
///
/// ```rust
/// Builder::default()
///     .service::<your_service>()
///     .bind_service::<other_service>(8080)
///     .build()
///     .await;
/// ```
pub struct Builder {
    registry: Arc<RwLock<registry::Type>>,
    handles: Vec<(&'static str, JoinHandle<Result<(), BoxError>>)>,
    session_handles: Vec<JoinHandle<Result<(), BoxError>>>,
    controller: Controller,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            registry: Default::default(),
            handles: Vec::new(),
            session_handles: Vec::new(),
            controller: Controller::default(),
        }
    }
}

impl Builder {
    /// Register new service builder to runtime service registry.
    pub fn service<S: Create + 'static>(&mut self) -> &mut Self
    where
        S::Error: std::error::Error + Send + Sync + 'static,
    {
        let registry = Arc::clone(&self.registry);
        let handle = tokio::spawn(async move {
            loop {
                let service = {
                    let guard = registry.read().unwrap();
                    S::with_registry(&*guard)
                };
                let service = match service {
                    Ok(Some(service)) => service,
                    Ok(None) => continue,
                    Err(err) => return Err(Box::new(err).into()),
                };
                let name = S::name();
                {
                    let mut guard = registry.write().unwrap();
                    guard.insert(S::name(), Box::new(service));
                }
                tracing::info!(message = "service registered", name);
                return Ok(());
            }
        });
        self.handles.push((S::name(), handle));
        self
    }

    /// Register new service builder to port `port`.
    pub fn bind_service<S, T>(&mut self, session: T) -> &mut Self
    where
        S: Info + Create,
        S::Error: std::error::Error + Send + Sync + 'static,
        T: Session<BoxCloneService<SocketAddr, Service<S>, BoxError>> + Send + 'static,
    {
        let controller = self.controller.clone();
        let registry = Arc::clone(&self.registry);
        let handle = tokio::spawn(async move {
            let service =
                ServiceBuilder::new()
                    .boxed_clone()
                    .service_fn(move |addr: SocketAddr| {
                        let registry = registry.clone();
                        async move {
                            tracing::info!(message = "new connection", addr = format!("{addr}"));
                            let service = {
                                let guard = registry.read().unwrap();
                                S::with_registry(&*guard)
                            };
                            let service = match service {
                                Ok(Some(service)) => service,
                                Ok(None) => return Err(Box::new(NotReady(S::name())).into()),
                                Err(err) => return Err(Box::new(err).into()),
                            };
                            Ok::<_, BoxError>(service)
                        }
                    });

            session.run(service, controller).await;
            Ok(())
        });
        self.session_handles.push(handle);
        todo!()
    }

    /// Build service runtime. Can only build once.
    pub async fn build(&mut self) -> Runtime {
        // TODO: Detect dependency cycles.
        for (name, service) in self.handles.drain(0..) {
            if let Err(err) = service.await {
                let report = crate::report!(err);
                tracing::error!("failed to register service '{name}'. reason: {report:?}");
                panic!("failed to register service '{name}'")
            }
        }
        todo!()
    }
}
