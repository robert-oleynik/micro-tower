use micro_tower::prelude::*;
use micro_tower::runtime::Runtime;
use micro_tower::{codegen::service, service::Service};
use serde::{Deserialize, Serialize};

// #[derive(Deserialize, Serialize)]
// pub struct Request {}

// #[derive(Deserialize, Serialize)]
// pub struct Response {
//     message: String,
// }

// #[service(buffer = 64)]
// async fn buffered_service(req: Request) -> Response {
//     Response {
//         message: String::from("Hello, World!"),
//     }
// }

#[service(buffer = 64)]
async fn buffered_service(req: i32) -> i32 {
    req
}

#[tokio::main]
async fn main() {
    let buffered_service = Service::<buffered_service>::builder().build().unwrap();

    let rt = Runtime::default()
        .bind_service(8080, buffered_service)
        .run()
        .await;
}
