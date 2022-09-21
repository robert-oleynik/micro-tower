use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{Buf, BufMut, BytesMut};
use futures::FutureExt;

use super::codec::{Decode, Encode};

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
    S: tower::Service<R>,
    S::Future: Unpin,
    C: Decode<R> + Encode<S::Response>,
    <C as Decode<R>>::Error: Unpin,
{
    type Response = bytes::BytesMut;
    type Error = bytes::BytesMut;
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
    S: tower::Service<R>,
    S::Future: Unpin,
    C: Decode<R> + Encode<S::Response>,
    <C as Decode<R>>::Error: Unpin,
{
    type Output = Result<bytes::BytesMut, bytes::BytesMut>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.inner {
            FutureState::Future(ref mut fut) => match fut.poll_unpin(cx) {
                Poll::Ready(Ok(response)) => {
                    if let Some(buf) = self.buf.take() {
                        let mut writer = buf.writer();
                        if let Err(_) = C::encode(&mut writer, response) {
                            todo!()
                        }
                        return Poll::Ready(Ok(writer.into_inner()));
                    }
                    Poll::Pending
                }
                Poll::Ready(Err(_)) => todo!(),
                Poll::Pending => Poll::Pending,
            },
            FutureState::BadRequest(_) => todo!(),
        }
    }
}
