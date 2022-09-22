#![feature(error_reporter)]

pub mod api;
pub mod builder;
pub mod session;
pub mod util;

pub mod prelude {
    pub use crate::builder::ServiceBuilderExt;
    pub use tower::{Layer, Service, ServiceExt};
    pub use tracing::Instrument;
}

pub mod export {
    pub use derive_builder;
    pub use tracing;
}

pub mod service {
    #[derive(Debug, thiserror::Error)]
    #[error("service `{0}` not ready")]
    pub struct NotReady(pub &'static str);
}

pub use micro_tower_codegen as codegen;
pub use tower::Service;
