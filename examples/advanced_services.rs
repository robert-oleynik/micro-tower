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

    Runtime::default()
        .bind_service(8080, buffered_service)
        .run()
        .await
}
