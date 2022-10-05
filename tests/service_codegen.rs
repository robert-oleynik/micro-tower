use micro_tower::prelude::*;
use micro_tower::util::{BoxError, BoxService};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Inner service failed with: {0}")]
    Inner(#[from] BoxError),
}

/// Some documentation
#[micro_tower::codegen::service]
async fn hello_world(_: ()) -> &'static str {
    "Hello World"
}

#[micro_tower::codegen::service]
async fn inner_service(request: (), mut inner: hello_world) -> Result<&'static str, Error> {
    Ok(inner.call(request).await?)
}

#[test]
fn build_hw_service() {
    let _service = hello_world::builder().build();
}

#[tokio::test]
async fn call_hw_service() {
    let mut service = hello_world::builder().build();

    let response = service.ready().await.unwrap().call(()).await.unwrap();
    assert_eq!(response, "Hello World");
}

#[test]
fn build_inner_service() {
    let hw_service = hello_world::builder().build();
    let _inner_service = inner_service::builder().inner(hw_service).build();
}

#[test]
#[should_panic]
fn build_inner_service_missing() {
    let _inner_service = inner_service::builder().build();
}

#[tokio::test]
async fn call_inner_service() {
    let hw_service = hello_world::builder().build();
    let mut inner_service = inner_service::builder().inner(hw_service).build();

    let response = inner_service.ready().await.unwrap().call(()).await.unwrap();
    assert_eq!(response, "Hello World");
}

#[micro_tower::codegen::service]
fn sync_service(_: ()) -> &'static str {
    "Hello World"
}

#[tokio::test]
async fn syn_service() {
    let mut service = sync_service::builder().build();

    let res = service.ready().await.unwrap().call(()).await.unwrap();
    assert_eq!(res, "Hello World");
}

#[micro_tower::codegen::service]
async fn extended_service(_: ()) {}

#[micro_tower::codegen::service(extend)]
async fn extended_service(req: String) -> String {
    req
}

#[micro_tower::codegen::service]
async fn inner_extended(req: String, mut inner: BoxService<String, extended_service>) -> String {
    inner.ready().await.unwrap().call(req).await.unwrap()
}

#[test]
fn build_extended_service() {
    let _service = extended_service::builder().build();
}

#[tokio::test]
async fn call_extended_service() {
    let service = extended_service::builder().build();
    let mut service: BoxService<String, extended_service> = Box::new(service);

    let response = service
        .ready()
        .await
        .unwrap()
        .call(String::from("Hello World"))
        .await
        .unwrap();
    assert_eq!(response, "Hello World");
}

#[test]
fn build_ext_inner_service() {
    let ext_service = extended_service::builder().build();
    let _inner_service = inner_extended::builder()
        .inner(Box::new(ext_service))
        .build();
}

#[tokio::test]
async fn call_inner_ext_service() {
    let ext_service = extended_service::builder().build();
    let mut inner_service = inner_extended::builder()
        .inner(Box::new(ext_service))
        .build();

    let response = inner_service
        .ready()
        .await
        .unwrap()
        .call(String::from("Hello World"))
        .await
        .unwrap();
    assert_eq!(response, "Hello World");
}

#[derive(Debug, thiserror::Error)]
#[error("placeholder")]
struct ErrorMockup;

#[allow(clippy::unnecessary_wraps)]
fn error_mockup() -> Result<(), ErrorMockup> {
    Ok(())
}

#[micro_tower::codegen::service]
async fn error_service(_: ()) -> Result<(), ErrorMockup> {
    error_mockup()?;
    Ok(())
}

#[tokio::test]
async fn call_error_service() {
    let mut service = error_service::builder().build();
    service.ready().await.unwrap().call(()).await.unwrap();
}

#[micro_tower::codegen::service]
async fn add_service(_: (), lhs: i64, rhs: i64) -> i64 {
    *lhs + *rhs
}

#[tokio::test]
async fn call_add_service() {
    let mut service = add_service::builder().lhs(8).rhs(16).build();
    let rep = service.ready().await.unwrap().call(()).await.unwrap();
    assert_eq!(rep, 24);
}
