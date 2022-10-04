pub mod pool;

pub use tower::*;

#[derive(Debug, thiserror::Error)]
#[error("service `{0}` not ready")]
pub struct NotReady(pub &'static str);
