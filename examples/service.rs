use micro_tower::runtime::Runtime;

#[micro_tower::codegen::service]
async fn hello_world(_: ()) -> &'static str {
    "Hello, World!"
}

#[micro_tower::codegen::service]
async fn hello_world2(_: ()) -> Result<String, std::convert::Infallible> {
    Ok(String::from("Hello, World!"))
}

#[micro_tower::codegen::service(crate = "micro_tower")]
async fn hello_args(_: ()) -> &'static str {
    "Hello, World!"
}

micro_tower::manifest! {
    Manifest: [
        hello_world
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
