use crate::shutdown::Controller;
use tokio::task::JoinHandle;
use tower::BoxError;

pub mod builder;
pub mod registry;

/// Used to manage and maintain services.
pub struct Runtime {
	controller: Controller,
	session_handles: Vec<JoinHandle<Result<(), BoxError>>>,
}

impl Runtime {
	/// Returns new runtime builder.
	#[must_use]
	pub fn builder() -> builder::Builder {
		builder::Builder::default()
	}

	/// Start runtime and wait for shutdown signal. Will register SIGTERM and SIGQUIT signal.
	pub async fn run(self) {
		match self.controller.spawn_handler() {
			Ok(handler) => {
				if let Err(err) = handler.await {
					let report = crate::report!(err);
					tracing::error!("Failed to await system signals. Reason: {report:?}");
				}
			}
			Err(err) => {
				let report = crate::report!(err);
				tracing::error!("Failed to register signal handler. Reason: {report:?}");
			}
		}
		tracing::info!("waiting for shutdown");

		for (i, session) in self.session_handles.into_iter().enumerate() {
			tracing::trace!(message = "waiting for session", i);
			if let Err(err) = session.await {
				let report = crate::report!(err);
				tracing::error!("{report:?}");
			}
		}
	}
}
