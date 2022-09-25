use std::net::SocketAddr;

use bytes::BytesMut;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tower::{BoxError, Service, ServiceExt};

use crate::{api, shutdown::Controller};

pub fn spawn<SB>(
    addr: SocketAddr,
    mut builder: SB,
    controller: &Controller,
) -> JoinHandle<Result<(), BoxError>>
where
    SB: Service<SocketAddr, Error = BoxError> + Send + 'static,
    SB::Future: Send,
    SB::Response: Service<BytesMut, Response = BytesMut, Error = api::Error> + Send,
    <SB::Response as Service<BytesMut>>::Future: Send,
{
    let controller = controller.clone();
    tokio::spawn(async move {
        let listener = TcpListener::bind(addr).await?;
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
                    tracing::error!("{report:?}")
                }
            });
        }
    })
}
