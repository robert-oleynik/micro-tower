#![feature(error_reporter)]

use std::num::ParseIntError;

use micro_tower::api::codec;
use micro_tower::runtime::Runtime;
use micro_tower::session::tcp;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

/// Service documentation
#[micro_tower::codegen::service(buffer = 1)]
pub async fn parse_str(request: String) -> Result<i32, ParseIntError> {
    request.parse()
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let addr = "127.0.0.1:4000".parse().unwrap();
    let session = tcp::Session::<codec::Json, _>::with_addr(addr)
        .await
        .unwrap();

    let rt = Runtime::builder()
        .bind_service::<parse_str, _>(session)
        .build()
        .await;

    rt.run().await;
}
