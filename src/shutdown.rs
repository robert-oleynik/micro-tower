use tokio::signal::unix;
use tokio::signal::unix::SignalKind;
use tokio::task::JoinHandle;
use tokio_util::sync::{CancellationToken, WaitForCancellationFuture};

#[derive(Default)]
pub struct Controller {
    token: CancellationToken,
}

impl Clone for Controller {
    fn clone(&self) -> Self {
        Self {
            token: self.token.child_token(),
        }
    }
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

    /// Spawns a new handler which waits for shutdown signals.
    pub fn spawn_handler(self) -> std::io::Result<JoinHandle<()>> {
        let mut qt = unix::signal(SignalKind::quit())?;
        let mut tm = unix::signal(SignalKind::terminate())?;

        let handle = tokio::spawn(async move {
            tokio::select! {
                _ = qt.recv() => {
                    tracing::debug!("received SIGQUIT signal");
                },
                _ = tm.recv() => {
                    tracing::debug!("received SIGTERM signal");
                }
                res = tokio::signal::ctrl_c() => {
                    match res {
                        Ok(_) => tracing::debug!("received ctrl-c shutdown request"),
                        Err(err) => {
                            let report = crate::report!(err);
                            tracing::error!("{report:?}");
                        }
                    }
                }
            }
            tracing::trace!("sending shutdown signal");
            self.shutdown();
            tracing::trace!("shutdown complete");
        });
        Ok(handle)
    }
}
