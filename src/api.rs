//! Utilities to translate request and replies.

use serde::{Deserialize, Serialize};

pub mod codec;
pub mod layer;
pub mod service;

use crate::util::BoxError;
pub use layer::Layer;
pub use service::Service;

/// TODO
#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Message<T> {
    #[serde(rename = "ok")]
    Ok { data: T },
    #[serde(rename = "400")]
    BadRequest,
    #[serde(rename = "500")]
    InternalServerError,
}

/// TODO
#[derive(Debug)]
pub struct Error {
    pub buf: bytes::BytesMut,
    pub err: BoxError,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("service failed")
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.err.as_ref())
    }
}
