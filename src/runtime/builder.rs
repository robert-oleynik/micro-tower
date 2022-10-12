use std::sync::{Arc, RwLock};

use tokio::task::JoinHandle;
use tower::BoxError;

use crate::service::Create;

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
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            registry: Default::default(),
            handles: Vec::new(),
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
                    guard.insert(name, Box::new(service));
                }
                tracing::info!(message = "service registered", name);
                return Ok(());
            }
        });
        self.handles.push((S::name(), handle));
        todo!()
    }

    /// Register new service builder to port `port`.
    pub fn bind_service<S>(&mut self, port: u16) -> &mut Self {
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
