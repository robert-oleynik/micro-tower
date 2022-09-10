use std::error::Report;
use std::sync::{Arc, Mutex};

use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::signal::unix::{signal, SignalKind};
use tokio::task::JoinHandle;
use tower::Layer;

use crate::api;
use crate::service::Service;
use crate::shutdown::Controller;

#[derive(Default)]
pub struct Runtime {
    controller: Arc<Mutex<Controller>>,
    services: Vec<JoinHandle<()>>,
}

impl Runtime {
    /// Bind a service against the tcp `port`.
    ///
    /// # Panics
    ///
    /// Will panic if failed to lock internal controller.
    pub fn bind_service<R, S>(&mut self, port: u16, service: Service<S>) -> &mut Self
    where
        R: DeserializeOwned + Send + Sync + 'static,
        S: tower::Service<R> + Clone + Send + Sync + 'static,
        S::Error: std::error::Error + Sync + Send,
        S::Future: Send,
        S::Response: Serialize,
    {
        let layer = api::Layer::default();
        let service = service.into_inner();
        let service = layer.layer(service);
        let watcher = self.controller.lock().unwrap().watcher();
        let handle = crate::tcp::run_service(port, watcher, service);
        self.services.push(handle);
        self
    }

    fn shutdown_on(&mut self, kind: SignalKind) -> JoinHandle<()> {
        // TODO: Not only unix specific stuff
        let controller = Arc::clone(&self.controller);
        tokio::spawn(async move {
            let mut watcher = controller.lock().unwrap().watcher();
            let mut sig_int = signal(kind).unwrap();
            tokio::select! {
                _ = sig_int.recv() => {
                    tracing::info!(
                        message = "signal received",
                        kind = format!("{}", kind.as_raw_value())
                    );
                    controller.lock().unwrap().send().unwrap();
                }
                _ = watcher.wait() => {}
            }
        })
    }

    pub async fn run(&mut self) {
        let signal_handles = vec![
            self.shutdown_on(SignalKind::interrupt()),
            self.shutdown_on(SignalKind::quit()),
            self.shutdown_on(SignalKind::terminate()),
        ];
        for task in self.services.drain(0..) {
            if let Err(err) = task.await {
                let report = Report::new(err).pretty(true);
                tracing::error!(
                    messsage = "failed to join task",
                    error = format!("{report:?}")
                );
            }
        }
        for signal in signal_handles {
            if let Err(err) = signal.await {
                let report = Report::new(err).pretty(true);
                tracing::error!(
                    messsage = "failed to join signal",
                    error = format!("{report:?}")
                );
            }
        }
    }
}
