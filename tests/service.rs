#[micro_tower::codegen::service]
async fn hello_world(_: ()) -> String {
    String::from("Hello, World!")
}
