#[micro_tower::codegen::service]
async fn hello_world(_: ()) -> &'static str {
    "Hello, World!"
}

micro_tower::runtime::manifest! {
    Manifest: [
        hello_world
    ]
}

#[tokio::main]
async fn main() {
    let _manifest = Manifest::create();
}
