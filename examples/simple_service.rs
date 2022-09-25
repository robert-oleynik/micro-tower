#![feature(error_reporter)]

use std::error::Report;
use std::net::SocketAddr;
use std::num::ParseIntError;

use micro_tower::api::codec;
use micro_tower::prelude::*;
use micro_tower::shutdown::Controller;
use micro_tower::ServiceBuilder;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[micro_tower::codegen::service]
async fn parse_str(request: String) -> Result<i32, ParseIntError> {
    request.parse()
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let controller = Controller::default();

    let service = ServiceBuilder::new().service_fn(|_| async move {
        let service = parse_str::builder().build();
        let service = ServiceBuilder::new()
            .api::<String, codec::Json>()
            .service(service);
        Ok(service)
    });

    let addr: SocketAddr = "127.0.0.1:8000".parse().unwrap();
    if let Err(err) = micro_tower::session::tcp::spawn(addr, service, &controller)
        .await
        .unwrap()
    {
        let report = Report::new(err.as_ref()).pretty(true);
        panic!("{report:?}")
    }
}
