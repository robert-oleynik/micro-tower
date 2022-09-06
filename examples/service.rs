use micro_tower::prelude::*;
use micro_tower::{runtime::Runtime, service::Service};
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

micro_tower::codegen::manifest! {
    Manifest: [
        hello_args: 8080,
        hello_world,
        hello_world2
    ]
}

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed setting default logging subscriber.");

    let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();
    Runtime::builder()
        .runtime(rt)
        .build()
        .unwrap()
        .manifest(Manifest::create)
        .run(Manifest::run);
}
