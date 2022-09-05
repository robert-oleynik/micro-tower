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
    use tower::Service;
    use tower::ServiceExt;
    service.ready().await.unwrap().call(()).await.unwrap()
}

micro_tower::manifest! {
    Manifest: [
        hello_args,
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

    tracing::trace!("Hello, WOrld!");

    let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();
    Runtime::builder()
        .runtime(rt)
        .build()
        .unwrap()
        .manifest::<Manifest>()
        .run();
}
