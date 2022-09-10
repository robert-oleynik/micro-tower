use std::error::Report;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tower::ServiceExt;

use crate::shutdown::Watcher;

pub fn run_service<S>(port: u16, watcher: Watcher, service: S) -> tokio::task::JoinHandle<()>
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
        tracing::info!(message = "listening", port);

        let counter = Arc::new(AtomicI64::new(0));

        loop {
            let mut w = watcher.clone();
            let res = tokio::select! {
                res = listener.accept() => res,
                _ = w.wait() => {
                    tracing::debug!(message = "stop accepting new connections", port);
                    break;
                }
            };
            let (mut socket, addr) = match res {
                Ok(result) => result,
                Err(err) => {
                    tracing::error!(
                        message = "failed to accept new connection",
                        error = format!("{err}")
                    );
                    break;
                }
            };
            counter.fetch_add(1, Ordering::AcqRel);
            tracing::trace!(message = "new connection", addr = format!("{addr}"));
            let mut watcher = watcher.clone();
            let mut service = service.clone();
            let counter = Arc::clone(&counter);
            tokio::spawn(async move {
                let mut buf = bytes::BytesMut::new();
                loop {
                    let res = tokio::select!(
                        res = socket.read_buf(&mut buf) => res,
                        _ = watcher.wait() => {
                            tracing::debug!(
                                message = "received shutdown request",
                                addr = format!("{addr}")
                            );
                            break;
                        }
                    );
                    if let Err(err) = res {
                        tracing::error!(
                            message = "failed to read message",
                            error = format!("{err}")
                        );
                        break;
                    }
                    tracing::trace!(message = "read message", len = buf.len());

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

                    tracing::trace!(message = "send message", len = buf.len());
                    if let Err(err) = socket.write_buf(&mut buf).await {
                        tracing::error!(
                            message = "Failed to send response",
                            error = format!("{err}")
                        );
                        break;
                    }
                }
                tracing::trace!(message = "tcp session closed", addr = format!("{addr}"));
                counter.fetch_sub(1, Ordering::AcqRel);
            });
        }

        while counter.load(Ordering::Acquire) > 0 {
            tracing::trace!(message = "yield process while waiting for shutdown", port);
            tokio::task::yield_now().await;
        }

        tracing::trace!(message = "socket closed", port);
    };

    tokio::spawn(fut)
}
