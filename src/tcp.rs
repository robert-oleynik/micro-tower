use std::error::Report;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tower::ServiceExt;

pub fn run_service<S>(port: u16, service: S) -> tokio::task::JoinHandle<()>
where
    S: tower::Service<bytes::BytesMut, Response = bytes::BytesMut> + Clone + Send + Sync + 'static,
    S::Future: Send,
    S::Error: std::error::Error + Send,
{
    let fut = async move {
        let listener = match TcpListener::bind(format!("127.0.0.1:{port}")).await {
            Ok(listener) => listener,
            Err(error) => {
                tracing::error!(
                    message = "failed to bind socket",
                    port,
                    error = format!("{error}")
                );
                return;
            }
        };
        tracing::info!("listening on port {port}");

        loop {
            let (mut socket, addr) = match listener.accept().await {
                Ok(result) => result,
                Err(err) => {
                    tracing::error!(
                        message = "failed to accept new connection",
                        error = format!("{err}")
                    );
                    break;
                }
            };
            tracing::trace!(message = "new connection", addr = format!("{addr}"));
            let mut service = service.clone();
            tokio::spawn(async move {
                let mut buf = bytes::BytesMut::new();
                loop {
                    if let Err(err) = socket.read_buf(&mut buf).await {
                        tracing::error!(
                            message = "failed to read message",
                            error = format!("{err}")
                        );
                        break;
                    }

                    let service = &mut service;
                    let fut = async move {
                        let buf = service.ready().await?.call(buf).await?;
                        Ok::<bytes::BytesMut, S::Error>(buf)
                    };

                    buf = match fut.await {
                        Ok(buf) => buf,
                        Err(err) => {
                            let report = Report::new(err).pretty(true);
                            tracing::error!("Failed to call service: {report:?}");
                            break;
                        }
                    };

                    if let Err(err) = socket.write_buf(&mut buf).await {
                        tracing::error!(
                            message = "Failed to send response",
                            error = format!("{err}")
                        );
                        break;
                    }
                }
                tracing::trace!(message = "tcp session closed", addr = format!("{addr}"));
            });
        }

        tracing::trace!(message = "socket closed", port);
    };

    tokio::spawn(fut)
}
