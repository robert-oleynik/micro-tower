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
pub enum Message<T> {
    Ok(T),
    InternalServerError,
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

impl<R, S: tower::Service<R>> tower::Layer<S> for Layer<R, S>
where
    R: DeserializeOwned,
    S::Response: Serialize,
{
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
    R: 'static,
    R: DeserializeOwned,
    S: tower::Service<R> + Clone + 'static,
    S::Response: Serialize,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    type Response = bytes::BytesMut;
    type Error = tower::BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner
            .poll_ready(cx)
            .map_err(Box::new)
            .map_err(Box::into)
    }

    fn call(&mut self, req: bytes::BytesMut) -> Self::Future {
        let mut reader = req.reader();
        let request: R = match serde_json::from_reader(&mut reader) {
            Ok(req) => req,
            Err(err) => return Box::pin(async move { Err(Box::new(err).into()) }),
        };
        let buf = reader.into_inner();
        let mut inner = self.inner.clone();

        let fut = async move {
            let response = inner.ready().await?.call(request).await?;
            Ok::<_, S::Error>(response)
        };

        let fut = async move {
            let response = match fut.await {
                Ok(response) => Message::Ok(response),
                Err(err) => {
                    let report = Report::new(err).pretty(true);
                    tracing::error!("Failed to handle request. {report:?}");
                    Message::<S::Response>::InternalServerError
                }
            };
            let mut writer = buf.writer();
            serde_json::to_writer(&mut writer, &response)?;
            return Ok(writer.into_inner());
        };

        Box::pin(fut)
    }
}
