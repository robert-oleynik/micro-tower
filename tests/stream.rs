#![feature(error_reporter)]

use bytes::{BufMut, BytesMut};
use micro_tower::api::codec;
use micro_tower::prelude::ServiceBuilderExt;
use micro_tower::shutdown::Controller;
use micro_tower::ServiceBuilder;
use std::cmp::min;
use std::io::ErrorKind;
use std::num::ParseIntError;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

struct Stream {
	input: String,
	ipos: usize,
	output: BytesMut,
}

impl Stream {
	pub fn from_input(input: impl Into<String>) -> Self {
		Self {
			input: input.into(),
			ipos: 0,
			output: BytesMut::new(),
		}
	}
}

impl AsyncRead for Stream {
	fn poll_read(
		mut self: Pin<&mut Self>,
		_cx: &mut Context<'_>,
		buf: &mut ReadBuf<'_>,
	) -> Poll<std::io::Result<()>> {
		if self.ipos < self.input.len() {
			let count = min(buf.remaining(), self.input.len() - self.ipos);
			buf.put_slice(&self.input.as_bytes()[self.ipos..self.ipos + count]);
			self.ipos += count;
			Poll::Ready(Ok(()))
		} else {
			Poll::Ready(Err(std::io::Error::new(
				ErrorKind::UnexpectedEof,
				"Buffer Empty",
			)))
		}
	}
}
impl AsyncWrite for Stream {
	fn poll_write(
		mut self: Pin<&mut Self>,
		_cx: &mut Context<'_>,
		buf: &[u8],
	) -> Poll<Result<usize, std::io::Error>> {
		self.output.put_slice(buf);
		Poll::Ready(Ok(buf.len()))
	}

	fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
		Poll::Ready(Ok(()))
	}

	fn poll_shutdown(
		self: Pin<&mut Self>,
		_cx: &mut Context<'_>,
	) -> Poll<Result<(), std::io::Error>> {
		Poll::Ready(Ok(()))
	}
}

#[micro_tower::codegen::service(buffer = 1)]
async fn parse(input: String) -> Result<i32, ParseIntError> {
	input.parse()
}

#[tokio::test]
async fn parse_stream_test() {
	let service = parse::builder().build();
	let service = ServiceBuilder::new()
		.api::<String, codec::Json>()
		.service(service);
	let stream = Stream::from_input(r#""42""#);
	let controller = Controller::default();

	if let Err(err) = micro_tower::session::stream::spawn_fut(stream, service, controller).await {
		if let Some(err) = err.downcast_ref::<std::io::Error>() {
			assert_eq!(err.kind(), ErrorKind::UnexpectedEof);
			assert_eq!(format!("{err}"), "Buffer Empty");
		} else {
			let report = micro_tower::report!(err.as_ref());
			panic!("{report:?}")
		}
	}
}

#[tokio::test]
async fn parse_stream_err_test() {
	let service = parse::builder().build();
	let service = ServiceBuilder::new()
		.api::<String, codec::Json>()
		.service(service);
	let stream = Stream::from_input(r#""test""#);
	let controller = Controller::default();

	if let Err(err) = micro_tower::session::stream::spawn_fut(stream, service, controller).await {
		if let Some(err) = err.downcast_ref::<std::io::Error>() {
			assert_eq!(err.kind(), ErrorKind::UnexpectedEof);
			assert_eq!(format!("{err}"), "Buffer Empty");
		} else {
			let report = micro_tower::report!(err.as_ref());
			panic!("Expected parse int error but got: {report:?}")
		}
	}
}

#[tokio::test]
async fn parse_stream_bad_request() {
	let service = parse::builder().build();
	let service = ServiceBuilder::new()
		.api::<String, codec::Json>()
		.service(service);
	let stream = Stream::from_input(r#"test"#);
	let controller = Controller::default();

	if let Err(err) = micro_tower::session::stream::spawn_fut(stream, service, controller).await {
		if let Some(err) = err.downcast_ref::<std::io::Error>() {
			assert_eq!(err.kind(), ErrorKind::UnexpectedEof);
			assert_eq!(format!("{err}"), "Buffer Empty");
		} else {
			let report = micro_tower::report!(err.as_ref());
			panic!("Expected parse int error but got: {report:?}")
		}
	}
}
