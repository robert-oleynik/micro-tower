use micro_tower::prelude::*;
use micro_tower::service::Service;
use micro_tower::util::BoxError;
use micro_tower::ServiceBuilder;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Inner service failed with: {0}")]
	Inner(#[from] BoxError),
}

/// Some documentation
#[micro_tower::codegen::service(buffer = 1)]
async fn hello_world(_: ()) -> &'static str {
	"Hello World"
}

#[micro_tower::codegen::service(buffer = 1)]
async fn inner_service(
	request: (),
	mut inner: Service<hello_world>,
) -> Result<&'static str, Error> {
	Ok(inner.ready().await?.call(request).await?)
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

#[tokio::test]
async fn build_inner_service() {
	let hw_service = hello_world::builder().build();
	let hw_service = ServiceBuilder::new()
		.boxed_future()
		.buffer(1)
		.service(hw_service);
	let hw_service = Service::from(Box::new(hw_service));
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
	let hw_service = ServiceBuilder::new()
		.boxed_future()
		.buffer(1)
		.service(hw_service);
	let hw_service = Service::from(Box::new(hw_service));
	let mut inner_service = inner_service::builder().inner(hw_service).build();

	let response = inner_service.ready().await.unwrap().call(()).await.unwrap();
	assert_eq!(response, "Hello World");
}

#[micro_tower::codegen::service(buffer = 1)]
fn sync_service(_: ()) -> &'static str {
	"Hello World"
}

#[tokio::test]
async fn syn_service() {
	let mut service = sync_service::builder().build();

	let res = service.ready().await.unwrap().call(()).await.unwrap();
	assert_eq!(res, "Hello World");
}

#[derive(Debug, thiserror::Error)]
#[error("placeholder")]
struct ErrorMockup;

#[allow(clippy::unnecessary_wraps)]
fn error_mockup() -> Result<(), ErrorMockup> {
	Ok(())
}

#[micro_tower::codegen::service(buffer = 1)]
async fn error_service(_: ()) -> Result<(), ErrorMockup> {
	error_mockup()?;
	Ok(())
}

#[tokio::test]
async fn call_error_service() {
	let mut service = error_service::builder().build();
	service.ready().await.unwrap().call(()).await.unwrap();
}

#[micro_tower::codegen::service(buffer = 1)]
async fn add_service(_: (), lhs: i64, rhs: i64) -> i64 {
	*lhs + *rhs
}

#[tokio::test]
async fn call_add_service() {
	let mut service = add_service::builder().lhs(8).rhs(16).build();
	let rep = service.ready().await.unwrap().call(()).await.unwrap();
	assert_eq!(rep, 24);
}
