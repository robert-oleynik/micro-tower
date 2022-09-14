use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use micro_tower::prelude::*;
use micro_tower::runtime::Runtime;
use micro_tower::{codegen::service, service::Service};
use serde::{Deserialize, Serialize};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(Deserialize, Serialize)]
pub struct Request {}

#[derive(Deserialize, Serialize)]
pub struct Response {
    message: String,
}

#[service(buffer = 64)]
async fn buffered_service(_req: Request) -> Response {
    tokio::time::sleep(Duration::from_secs(5)).await;
    Response {
        message: String::from("Hello, World!"),
    }
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed setting default logging subscriber.");

    let buffered_service = Service::<buffered_service>::builder().build().unwrap();

    let pool_service_maker = tower::ServiceBuilder::new().service_fn(
        |_: ()| -> Pin<Box<dyn Future<Output = Result<_, Infallible>> + Send>> {
            let service = Service::<buffered_service>::builder()
                .build()
                .unwrap()
                .into_inner();
            let service = tower::load::PendingRequests::new(
                service,
                tower::load::CompleteOnResponse::default(),
            );
            Box::pin(async move { Ok(service) })
        },
    );
    let pool = tower::balance::pool::Builder::new()
        .max_services(Some(16))
        .build(pool_service_maker, ());
    let pooled_service = tower::ServiceBuilder::new().buffer(64).service(pool);

    Runtime::default()
        .bind_service(8080, buffered_service)
        .bind_service(8000, pooled_service)
        .run()
        .await
}
