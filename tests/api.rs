#![feature(error_reporter)]

use std::error::Report;
use std::task::{Context, Poll};

use bytes::{Buf, BufMut, BytesMut};
use micro_tower::api;
use micro_tower::api::codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use tower::{BoxError, Layer, Service as TowerService, ServiceExt};

#[derive(Deserialize, Serialize)]
struct Request {
    input: String,
}

#[derive(Deserialize, Serialize)]
struct Response {
    m: i32,
}

#[derive(Default)]
struct Service;

impl tower::Service<Request> for Service {
    type Response = Response;
    type Error = BoxError;
    type Future = micro_tower::util::BoxFuture<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        Box::pin(async move {
            Ok(Response {
                m: req.input.parse()?,
            })
        })
    }
}

#[test]
fn setup_api_layer() {
    let service = Service::default();
    let layer = api::Layer::<Request, api::codec::Json>::default();
    let _ = layer.layer(service);
}

#[test]
fn json_codec_encode() {
    let buf = BytesMut::new();
    let mut writer = buf.writer();
    let request = Request {
        input: String::new(),
    };
    api::codec::Json::encode(&mut writer, request).unwrap();
    let mut buf = writer.into_inner();
    assert_eq!(String::from_utf8_lossy(&buf[..]), r#"{"input":""}"#);

    buf.clear();
    let mut writer = buf.writer();
    let response = Response { m: 42 };
    api::codec::Json::encode(&mut writer, response).unwrap();
    let buf = writer.into_inner();
    assert_eq!(String::from_utf8_lossy(&buf[..]), "{\"m\":42}");
}

#[test]
fn json_codec_decode() {
    let mut buf = BytesMut::new();
    buf.put(&b"{\"input\":\"\"}"[..]);
    let mut reader = buf.reader();
    let _request: Request = api::codec::Json::decode(&mut reader).unwrap();

    let mut buf = BytesMut::new();
    buf.put(&b"{\"m\":42}"[..]);
    let mut reader = buf.reader();
    let r: Response = api::codec::Json::decode(&mut reader).unwrap();
    assert_eq!(r.m, 42);
}

#[tokio::test]
async fn api_call() {
    let call = || async move {
        let layer = api::Layer::<Request, api::codec::Json>::default();
        let mut service = layer.layer(Service::default());

        let mut buf = BytesMut::new();
        buf.put(&b"{\"input\":\"42\"}"[..]);

        service.ready().await?.call(buf).await
    };

    match (call)().await {
        Ok(r) => assert_eq!(
            String::from_utf8_lossy(&r[..]),
            r#"{"type":"ok","data":{"m":42}}"#
        ),
        Err(err) => {
            let report = Report::new(err).pretty(true);
            panic!("{report:?}")
        }
    };
}

#[tokio::test]
async fn api_bad_request() {
    let call = || async move {
        let layer = api::Layer::<Request, api::codec::Json>::default();
        let mut service = layer.layer(Service::default());

        let mut buf = BytesMut::new();
        buf.put(&b"{input:42}"[..]);

        service.ready().await?.call(buf).await
    };

    match (call)().await {
        Ok(_) => panic!("expected error"),
        Err(err) => {
            let report = Report::new(&err).pretty(true);
            eprintln!("{report:?}");
            assert_eq!(String::from_utf8_lossy(&err.buf[..]), r#"{"type":"400"}"#);
        }
    };
}

#[tokio::test]
async fn api_call_trailing() {
    let call = || async move {
        let layer = api::Layer::<Request, api::codec::Json>::default();
        let mut service = layer.layer(Service::default());

        let mut buf = BytesMut::new();
        buf.put(&b"{\"input\":\"42\"}{trailing}"[..]);

        service.ready().await?.call(buf).await
    };

    match (call)().await {
        Ok(_) => panic!("expected error"),
        Err(err) => {
            let report = Report::new(&err).pretty(true);
            eprintln!("{report:?}");
            assert_eq!(String::from_utf8_lossy(&err.buf[..]), r#"{"type":"400"}"#);
        }
    };
}

#[tokio::test]
async fn api_call_internal_error() {
    let call = || async move {
        let layer = api::Layer::<Request, api::codec::Json>::default();
        let mut service = layer.layer(Service::default());

        let mut buf = BytesMut::new();
        buf.put(&b"{\"input\":\"not an int\"}"[..]);

        service.ready().await?.call(buf).await
    };

    match (call)().await {
        Ok(_) => panic!("expected error"),
        Err(err) => {
            let report = Report::new(&err).pretty(true);
            eprintln!("{report:?}");
            assert_eq!(String::from_utf8_lossy(&err.buf[..]), r#"{"type":"500"}"#);
        }
    };
}
