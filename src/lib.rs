#![feature(error_reporter)]

pub mod api;
pub mod builder;
pub mod layer;
pub mod runtime;
pub mod session;
pub mod shutdown;
pub mod util;

pub mod prelude {
	pub use crate::builder::{ServiceBuilderExt, ServicePoolBuilderExt};
	pub use tower::{Layer, Service as TowerService, ServiceExt};
	pub use tracing::Instrument;
}

pub mod export {
	pub use {derive_builder, tokio, tracing, tracing_subscriber};
}

pub mod service;

pub use micro_tower_codegen as codegen;
pub use tower::{Service, ServiceBuilder};
