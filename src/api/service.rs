use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{Buf, BufMut, BytesMut};
use futures::FutureExt;

use super::codec::{Decode, Encode};
use super::{Error, Message};
use crate::util::BoxError;

/// API service which translates bytes to requests of type `T` and response to bytes.
pub struct Service<R, C, S> {
    inner: S,
    _p: PhantomData<(C, R)>,
}

pub struct Future<R, C, S>
where
    S: tower::Service<R>,
    S::Future: Unpin,
    C: Decode<R>,
    C::Error: Unpin,
{
    inner: FutureState<<C as Decode<R>>::Error, S::Future>,
    buf: Option<BytesMut>,
    _p: PhantomData<Pin<Box<(R, C, S)>>>,
}

enum FutureState<E, F> {
    BadRequest(E),
    Future(F),
}

impl<R, C, S> Service<R, C, S> {
    /// Creates new api layer by wrapping inner service
    ///
    /// # Parameters
    /// - `inner` Service wrapped by API layer.
    pub fn from_service(inner: S) -> Self {
        Self {
            inner,
            _p: PhantomData,
        }
    }
}

impl<R, C, S> tower::Service<BytesMut> for Service<R, C, S>
where
    S: tower::Service<R, Error = BoxError>,
    S::Future: Unpin,
    C: Decode<R> + Encode<Message<S::Response>>,
    <C as Encode<Message<S::Response>>>::Error: std::error::Error + Send + Sync + 'static,
    <C as Decode<R>>::Error: Unpin,
{
    type Response = bytes::BytesMut;
    type Error = Error;
    type Future = Future<R, C, S>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.inner.poll_ready(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(_)) => todo!(),
            Poll::Pending => Poll::Pending,
        }
    }

    fn call(&mut self, req: BytesMut) -> Self::Future {
        let mut reader = req.reader();
        match C::decode(&mut reader) {
            Ok(request) => Future {
                inner: FutureState::Future(self.inner.call(request)),
                buf: Some(reader.into_inner()),
                _p: PhantomData,
            },
            Err(err) => Future {
                inner: FutureState::BadRequest(err),
                buf: Some(reader.into_inner()),
                _p: PhantomData,
            },
        }
    }
}

impl<R, C, S: Clone> Clone for Service<R, C, S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _p: PhantomData,
        }
    }
}

impl<R, C, S> std::future::Future for Future<R, C, S>
where
    S: tower::Service<R, Error = BoxError>,
    S::Future: Unpin,
    C: Decode<R> + Encode<Message<S::Response>>,
    <C as Encode<Message<S::Response>>>::Error: std::error::Error + Send + Sync + 'static,
    <C as Decode<R>>::Error: Unpin,
{
    type Output = Result<bytes::BytesMut, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let buf = match self.buf.take() {
            Some(buf) => buf,
            None => return Poll::Pending,
        };
        match self.inner {
            FutureState::Future(ref mut fut) => match fut.poll_unpin(cx) {
                Poll::Ready(Ok(response)) => {
                    let message = Message::Ok { data: response };
                    let mut writer = buf.writer();
                    if let Err(err) = C::encode(&mut writer, message) {
                        let err = Error {
                            buf: Some(writer.into_inner()),
                            err: Box::new(err),
                        };
                        return Poll::Ready(Err(err));
                    }
                    Poll::Ready(Ok(writer.into_inner()))
                }
                Poll::Ready(Err(err)) => {
                    let message = Message::<S::Response>::InternalServerError;
                    let mut writer = buf.writer();
                    if let Err(err) = C::encode(&mut writer, message) {
                        panic!("{err}");
                    }
                    let err = Error { buf: None, err };
                    return Poll::Ready(Err(err));
                }
                Poll::Pending => {
                    self.buf.replace(buf);
                    Poll::Pending
                }
            },
            FutureState::BadRequest(_) => todo!(),
        }
    }
}
