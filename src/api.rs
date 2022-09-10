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
pub enum Error<E: std::error::Error> {
    #[error("Failed to serialize message")]
    Serialize(#[source] serde_json::Error),
    #[error("Failed")]
    Service(#[source] E),
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

impl<R, S: Clone> tower::Layer<S> for Layer<R, S> {
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
    type Error = Error<S::Error>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Self::Error::Service)
    }

    fn call(&mut self, req: bytes::BytesMut) -> Self::Future {
        let mut reader = req.reader();
        let request: R = match serde_json::from_reader(&mut reader) {
            Ok(req) => req,
            Err(_) => {
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
            }
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
            return Ok(writer.into_inner());
        };

        Box::pin(fut)
    }
}
