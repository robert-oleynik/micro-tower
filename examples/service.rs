use micro_tower::{runtime::Runtime, service::Service};

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
        hello_world,
        hello_world2,
        hello_args
    ]
}

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();
    Runtime::builder()
        .runtime(rt)
        .build()
        .unwrap()
        .manifest::<Manifest>()
        .run();
}
