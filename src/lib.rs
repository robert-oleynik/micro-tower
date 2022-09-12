#![feature(error_reporter)]

pub mod api;
pub mod connection;
#[cfg(feature = "r2d2")]
pub mod r2d2;
pub mod runtime;
pub mod service;
pub mod shutdown;
pub mod tcp;
pub mod util;

pub mod prelude {
    pub use crate::util::Buildable;
    pub use tower::Service as TowerService;
    pub use tower::ServiceExt as TowerServiceExt;
}

pub use micro_tower_codegen as codegen;

pub mod export {
    pub use derive_builder;
    pub use tokio;
    pub use tower;
    pub use tracing;
}
