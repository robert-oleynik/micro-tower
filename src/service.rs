//! Reexport [`tower`] utilities.

pub mod pool;

pub use tower::*;

#[derive(Debug, thiserror::Error)]
#[error("service `{0}` not ready")]

pub struct NotReady(pub &'static str);

/// Interface to obtain information about services. Used at compile and runtime time.
pub trait Info {
    /// Request type of service.
    type Request;

    /// Returns generic service name. Used to identify a service and should be unique other all
    /// services.
    fn name() -> &'static str;
}
