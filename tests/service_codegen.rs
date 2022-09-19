#[micro_tower::codegen::service]
async fn hello_world(_: ()) -> &'static str {
    "Hello World"
}
