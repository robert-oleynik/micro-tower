use micro_tower::prelude::*;
use micro_tower::service::Service;
use micro_tower::util::Buildable;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[micro_tower::codegen::service]
async fn hello_world(_: ()) -> &'static str {
    "Hello, World!"
}

#[micro_tower::codegen::service]
async fn hello_world2(_: ()) -> Result<String, std::convert::Infallible> {
    Ok(String::from("Hello, World!"))
}

#[micro_tower::codegen::service(crate = "micro_tower")]
async fn hello_args(_: (), mut service: Service<hello_world>) -> &'static str {
    service.ready().await.unwrap().call(()).await.unwrap()
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed setting default logging subscriber.");

    let hw_service = Service::<hello_world>::builder().build().unwrap();
    let hw2_service = Service::<hello_world2>::builder().build().unwrap();
    let ha_service = Service::<hello_args>::builder()
        .service(hw_service)
        .build()
        .unwrap();
}
