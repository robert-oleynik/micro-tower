use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use tower::ServiceExt;

pub fn run_service<S>(port: u16, service: S) -> tokio::task::JoinHandle<()>
where
    S: tower::Service<bytes::BytesMut, Response = bytes::BytesMut> + Clone + Send + 'static,
    S::Future: Send,
    S::Error: std::error::Error,
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
                    let service = match service.ready().await {
                        Ok(service) => service,
                        Err(err) => {
                            tracing::error!(
                                message = "Failed to wait for service",
                                error = format!("{err}")
                            );
                            continue;
                        }
                    };
                    buf = match service.call(buf).await {
                        Ok(buf) => buf,
                        Err(err) => {
                            tracing::error!(
                                message = "Failed to call service",
                                error = format!("{err}")
                            );
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
            });
        }
    };

    tokio::spawn(fut)
}
