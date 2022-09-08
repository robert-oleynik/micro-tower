use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::task::JoinHandle;
use tower::Layer;

use crate::api;
use crate::service::Service;

#[derive(Default)]
pub struct Runtime {
    services: Vec<JoinHandle<()>>,
}

impl Runtime {
    /// Bind a service against the tcp `port`.
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
        let handle = crate::tcp::run_service(port, service);
        self.services.push(handle);
        self
    }

    pub async fn run(&mut self) {
        for task in self.services.drain(0..) {
            task.await.unwrap()
        }
    }
}
