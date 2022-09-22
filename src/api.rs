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
pub struct Error {
    buf: bytes::BytesMut,
    err: BoxError,
}
