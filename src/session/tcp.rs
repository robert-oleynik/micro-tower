use std::net::SocketAddr;

use bytes::BytesMut;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tower::{BoxError, Service, ServiceExt};

use crate::api;
use crate::shutdown::Controller;
use crate::util::BoxFuture;

pub struct Session {
    addr: SocketAddr,
    listener: TcpListener,
}

impl Session {
    /// Create tcp session that binds to address `addr`.
    ///
    /// # Errors
    ///
    /// Will return `Err` if failed to create tcp listener.
    pub async fn with_addr(addr: SocketAddr) -> std::io::Result<Self> {
        let listener = TcpListener::bind(&addr).await?;
        Ok(Self { addr, listener })
    }
}

impl<SB> super::Session<SB> for Session
where
    SB: Service<SocketAddr, Error = BoxError> + Send + 'static,
    SB::Future: Send,
    SB::Response: Service<BytesMut, Response = BytesMut, Error = api::Error> + Send,
    <SB::Response as Service<BytesMut>>::Future: Send,
{
    fn run(self, mut builder: SB, controller: Controller) -> BoxFuture<Result<(), BoxError>> {
        Box::pin(async move {
            let addr = self.addr;
            let listener = self.listener;
            tracing::info!(message = "listening on", port = addr.port());

            loop {
                tracing::trace!(message = "wait for new connections", port = addr.port());

                let (stream, addr) = tokio::select! {
                    result = listener.accept() => result?,
                    _ = controller.wait_for_shutdown() => return Ok(())
                };

                let service = match builder.ready().await {
                    Ok(service) => service,
                    Err(err) => {
                        let report = crate::report!(err.as_ref());
                        tracing::error!("{report:?}");
                        continue;
                    }
                };
                let service = match service.call(addr).await {
                    Ok(service) => service,
                    Err(err) => {
                        let report = crate::report!(err.as_ref());
                        tracing::error!("{report:?}");
                        continue;
                    }
                };

                tracing::info!(message = "new connection", addr = format!("{addr}"));

                let controller = controller.clone();
                tokio::spawn(async move {
                    if let Err(err) = super::stream::spawn_fut(stream, service, controller).await {
                        let report = crate::report!(err.as_ref());
                        tracing::error!("{report:?}");
                    }
                });
            }
        })
    }
}
