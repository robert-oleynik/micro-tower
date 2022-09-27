use micro_tower::prelude::*;
use micro_tower::util::BoxError;

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
