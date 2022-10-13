#![feature(error_reporter)]

use std::num::ParseIntError;

use micro_tower::{runtime::Runtime, session::tcp};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

/// Other service
#[micro_tower::codegen::service]
pub async fn other(_: ()) -> &'static str {
    "Hello World"
}

/// Service documentation
#[micro_tower::codegen::service]
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
    let session = tcp::Session::with_addr(addr).await.unwrap();

    let rt = Runtime::builder()
        .service::<other>()
        .bind_service::<parse_str, _>(session)
        .build()
        .await;

    rt.run().await

    // let controller = Controller::default();

    // let service = ServiceBuilder::new().service_fn(|_| async move {
    //     let service = parse_str::builder().build();
    //     let service = ServiceBuilder::new()
    //         .api::<String, codec::Json>()
    //         .service(service);
    //     Ok(service)
    // });

    // let addr: SocketAddr = "127.0.0.1:8000".parse().unwrap();
    // let service_handle = micro_tower::session::tcp::spawn(addr, service, &controller);
    // let shutdown_handle = controller.spawn_handler().unwrap();

    // if let Err(err) = service_handle.await.unwrap() {
    //     let report = micro_tower::report!(err.as_ref());
    //     panic!("{report:?}")
    // }
    // shutdown_handle.await.unwrap();
}
