use super::{registry, Runtime};
use crate::service::{Create, Info, NotReady, Service};
use crate::session::Session;
use crate::shutdown::Controller;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::task::JoinHandle;
use tower::util::BoxCloneService;
use tower::{BoxError, ServiceBuilder};

/// Implements builder pattern used to generate runtime.
///
/// # Usage
///
/// ```rust
/// Builder::default()
/// 	.service::<your_service>()
/// 	.bind_service::<other_service>(8080)
/// 	.build()
/// 	.await;
/// ```
#[derive(Default)]
pub struct Builder {
	registry: Arc<RwLock<registry::Type>>,
	handles: Vec<(&'static str, JoinHandle<Result<(), BoxError>>)>,
	session_handles: Vec<JoinHandle<Result<(), BoxError>>>,
	controller: Controller,
}

impl Builder {
	/// Register new service builder to runtime service registry.
	///
	/// # Panics
	///
	/// Will panic if internal mutex failed to lock registry.
	#[must_use]
	pub fn service<S: Create + 'static>(mut self) -> Self
	where
		S::Error: std::error::Error + Send + Sync + 'static,
	{
		let registry = Arc::clone(&self.registry);
		let handle = tokio::spawn(async move {
			loop {
				let service = S::with_registry(registry.clone());
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
	///
	/// # Panics
	///
	/// Will panic if internal mutex failed to lock registry.
	#[must_use]
	pub fn bind_service<S, T>(mut self, session: T) -> Self
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
							let service = S::with_registry(registry.clone());
							let service = match service {
								Ok(Some(service)) => service,
								Ok(None) => return Err(Box::new(NotReady(S::name())).into()),
								Err(err) => return Err(Box::new(err).into()),
							};
							Ok::<_, BoxError>(service)
						}
					});

			session.run(service, controller).await
		});
		self.session_handles.push(handle);
		self
	}

	/// Build service runtime. Can only build once.
	///
	/// # Panics
	///
	/// Will panic if failed to create any service.
	pub async fn build(self) -> Runtime {
		// TODO: Detect dependency cycles.
		for (name, service) in self.handles {
			if let Err(err) = service.await {
				let report = crate::report!(err);
				tracing::error!("failed to register service '{name}'. reason: {report:?}");
				panic!("failed to register service '{name}'")
			}
		}
		Runtime {
			controller: self.controller,
			session_handles: self.session_handles,
		}
	}
}
