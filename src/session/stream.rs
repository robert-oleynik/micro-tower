use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tower::ServiceExt;

use crate::api;
use crate::shutdown::Controller;
use crate::util::BoxError;

const BUF_SIZE: usize = 1024;

/// Spawns a future to handle streams of requests (e.g. a tcp stream).
///
/// # Errors
///
/// Will return `Err` if failed to read bytes from stream or send bytes to stream.
pub async fn spawn_fut<St, Sv>(
    mut stream: St,
    mut service: Sv,
    controller: Controller,
) -> Result<(), BoxError>
where
    St: AsyncReadExt + AsyncWriteExt + Unpin + Send + 'static,
    Sv: tower::Service<BytesMut, Response = BytesMut, Error = api::Error> + Send + 'static,
    Sv::Future: Send,
{
    let mut buf = BytesMut::new();
    let mut local_buf = [0_u8; BUF_SIZE];
    loop {
        buf.clear();
        loop {
            let num = tokio::select! {
                res = stream.read(&mut local_buf) => res,
                _ = controller.wait_for_shutdown() => return Ok(())
            };
            let num = num?;
            buf.extend_from_slice(&local_buf[..num]);
            if num < BUF_SIZE {
                break;
            }
        }
        tracing::trace!(message = "buffer read", size = buf.len());
        let ready = match service.ready().await {
            Ok(service) => service,
            Err(err) => {
                return Err(err.err);
            }
        };
        buf = match ready.call(buf).await {
            Ok(buf) => buf,
            Err(err) => {
                let report = crate::report!(err.err.as_ref());
                tracing::error!("{report:?}");
                err.buf
            }
        };
        tracing::trace!(message = "write buffer", size = buf.len());
        stream.write_buf(&mut buf).await?;
    }
}
