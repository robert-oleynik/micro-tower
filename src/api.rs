use std::error::Report;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{Buf, BufMut};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum Message<T> {
    Ok { data: T },
    InternalServerError,
    BadRequest,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to serialize message")]
    Serialize(#[source] serde_json::Error),
    #[error("Failed failed")]
    Service,
}

pub struct InnerService<R, S> {
    inner: S,
    _p: PhantomData<R>,
}

/// Used as a translation layer between byte buffer and data structures.
pub struct Layer<R, S> {
    _p: PhantomData<(R, S)>,
}

impl<R, S> Default for Layer<R, S> {
    fn default() -> Self {
        Self { _p: PhantomData }
    }
}
impl<R, S> tower::Layer<S> for Layer<R, S> {
    type Service = InnerService<R, S>;

    fn layer(&self, inner: S) -> Self::Service {
        InnerService {
            inner,
            _p: PhantomData,
        }
    }
}

impl<R, S: Clone> Clone for InnerService<R, S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _p: PhantomData,
        }
    }
}

impl<R, S> tower::Service<bytes::BytesMut> for InnerService<R, S>
where
    R: DeserializeOwned + Send + 'static,
    S: tower::Service<R> + Clone + Send + 'static,
    S::Response: Serialize,
    S::Future: Send,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    type Response = bytes::BytesMut;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.inner.poll_ready(cx) {
            Poll::Ready(Ok(_)) => Poll::Ready(Ok(())),
            Poll::Ready(Err(err)) => {
                let report = Report::new(err).pretty(true);
                tracing::error!("service failed. {report:?}");
                Poll::Ready(Err(Error::Service))
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn call(&mut self, req: bytes::BytesMut) -> Self::Future {
        let mut reader = req.reader();
        let request: R = if let Ok(req) = serde_json::from_reader(&mut reader) {
            req
        } else {
            let mut buf = reader.into_inner();
            buf.clear();
            let mut writer = buf.writer();
            let response = Message::<S::Response>::BadRequest;
            return match serde_json::to_writer(&mut writer, &response) {
                Ok(_) => {
                    let buf = writer.into_inner();
                    Box::pin(async move { Ok(buf) })
                }
                Err(err) => {
                    let err = Self::Error::Serialize(err);
                    Box::pin(async move { Err(err) })
                }
            };
        };
        let buf = reader.into_inner();
        let mut inner = self.inner.clone();

        let fut = async move {
            let response = inner.ready().await?.call(request).await?;
            Ok::<_, S::Error>(response)
        };

        let fut = async move {
            let response = match fut.await {
                Ok(response) => Message::Ok { data: response },
                Err(err) => {
                    let report = Report::new(err).pretty(true);
                    tracing::error!("Failed to handle request. {report:?}");
                    Message::<S::Response>::InternalServerError
                }
            };
            let mut writer = buf.writer();
            serde_json::to_writer(&mut writer, &response).map_err(Self::Error::Serialize)?;
            Ok(writer.into_inner())
        };

        Box::pin(fut)
    }
}
