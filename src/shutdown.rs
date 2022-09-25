use tokio_util::sync::{CancellationToken, WaitForCancellationFuture};

#[derive(Clone, Default)]
pub struct Controller {
    token: CancellationToken,
}

impl Controller {
    /// Create a new controller. Same as [`Controller::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Emit shutdown signal.
    pub fn shutdown(&self) {
        self.token.cancel();
    }

    /// Returns future to await shutdown signal.
    pub fn wait_for_shutdown(&self) -> WaitForCancellationFuture<'_> {
        self.token.cancelled()
    }
}
