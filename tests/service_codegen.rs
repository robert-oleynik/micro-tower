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
