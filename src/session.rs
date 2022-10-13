//! Structs and methods to manage channels (tcp, streams, etc.)

use tower::BoxError;

use crate::shutdown::Controller;
use crate::util::BoxFuture;

pub mod stream;
pub mod tcp;

pub trait Session<SB> {
    /// # Parameter
    ///
    /// - `service_builder` Used to build new services on demand
    /// - `controller` Used to manage graceful shutdown
    fn run(self, service_builder: SB, controller: Controller) -> BoxFuture<Result<(), BoxError>>;
}
