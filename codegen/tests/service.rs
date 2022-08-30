#[micro_tower_codegen::service]
async fn hello_world(_: ()) -> String {
    String::from("Hello, World!")
}
