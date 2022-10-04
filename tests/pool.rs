use std::convert::Infallible;

use micro_tower::prelude::*;
use micro_tower::ServiceBuilder;
use tower::load::CompleteOnResponse;
use tower::load::PendingRequests;

#[micro_tower::codegen::service]
async fn service_moc(request: usize, num: usize) -> usize {
    request + *num
}

#[tokio::test]
async fn call_pool() {
    let mut pool = ServiceBuilder::new()
        .pooled(4, ())
        .service_fn(|_| async move {
            let service = service_moc::builder().num(42).build();
            let service = PendingRequests::new(service, CompleteOnResponse::default());
            Ok::<_, Infallible>(service)
        });

    pool.ready().await.unwrap().call(22).await.unwrap();
}

#[tokio::test]
async fn multi_call_pool() {
    let mut pool = ServiceBuilder::new()
        .pooled(4, ())
        .service_fn(|_| async move {
            let service = service_moc::builder().num(42).build();
            let service = PendingRequests::new(service, CompleteOnResponse::default());
            Ok::<_, Infallible>(service)
        });

    for _ in 0..128 {
        let rep = pool.ready().await.unwrap().call(22).await.unwrap();
        assert_eq!(rep, 64);
    }
}
