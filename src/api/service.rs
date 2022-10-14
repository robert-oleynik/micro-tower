use super::codec::{Decode, Encode};
use super::{Error, Message};
use crate::util::{BoxError, BoxFuture};
use bytes::{Buf, BufMut, BytesMut};
use std::marker::PhantomData;
use std::task::{Context, Poll};

/// API service which translates bytes to requests of type `T` and response to bytes.
pub struct Service<R, C, S> {
	inner: S,
	_p: PhantomData<(C, R)>,
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
	S::Future: Send + 'static,
	C: Decode<R> + Encode<Message<S::Response>>,
	<C as Encode<Message<S::Response>>>::Error: std::error::Error + Send + Sync + 'static,
	<C as Decode<R>>::Error: Unpin + std::error::Error + Send + Sync + 'static,
{
	type Response = bytes::BytesMut;
	type Error = Error;
	type Future = BoxFuture<Result<Self::Response, Self::Error>>;

	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		match self.inner.poll_ready(cx) {
			Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
			Poll::Ready(Err(_)) => todo!(),
			Poll::Pending => Poll::Pending,
		}
	}

	fn call(&mut self, buf: BytesMut) -> Self::Future {
		let mut reader = buf.reader();
		match C::decode(&mut reader) {
			Ok(request) => {
				let buf = reader.into_inner();
				let fut = self.inner.call(request);
				Box::pin(async move {
					match fut.await {
						Ok(response) => {
							let message = Message::Ok { data: response };
							let mut writer = buf.writer();
							if let Err(err) = C::encode(&mut writer, message) {
								let err = Error {
									buf: writer.into_inner(),
									err: Box::new(err),
								};
								return Err(err);
							}
							Ok(writer.into_inner())
						}
						Err(err) => {
							let message = Message::InternalServerError;
							let mut writer = buf.writer();
							C::encode(&mut writer, message).unwrap();
							let err = Error {
								buf: writer.into_inner(),
								err,
							};
							Err(err)
						}
					}
				})
			}
			Err(err) => {
				let message = Message::BadRequest;
				let mut buf = reader.into_inner();
				buf.clear();
				let mut writer = buf.writer();
				C::encode(&mut writer, message).unwrap();
				let buf = writer.into_inner();
				let err = Error {
					buf,
					err: Box::new(err),
				};
				Box::pin(async move { Err(err) })
			}
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
