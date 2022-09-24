use std::error::Report;
use std::future::Future;

use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tower::ServiceExt;

use crate::api;
use crate::util::BoxError;

const BUF_SIZE: usize = 1024;

pub fn spawn_fut<St, Sv>(
    mut stream: St,
    mut service: Sv,
) -> impl Future<Output = Result<(), BoxError>>
where
    St: AsyncReadExt + AsyncWriteExt + Unpin + Send + 'static,
    Sv: tower::Service<BytesMut, Response = BytesMut, Error = api::Error> + Send + 'static,
    Sv::Future: Send,
{
    async move {
        let mut buf = BytesMut::new();
        let mut lbuf = [0_u8; BUF_SIZE];
        loop {
            buf.clear();
            loop {
                let num = stream.read(&mut lbuf).await?;
                buf.extend_from_slice(&lbuf[..num]);
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
                    let report = Report::new(err.err.as_ref()).pretty(true);
                    tracing::error!("{report:?}");
                    err.buf
                }
            };
            tracing::trace!(message = "writer buffer", size = buf.len());
            stream.write_buf(&mut buf).await?;
        }
    }
}
