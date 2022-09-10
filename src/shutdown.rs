use tokio::sync::watch::{Receiver, Sender};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Message channel closed")]
    Closed,
}

/// Used to control a coordinated system shutdown.
///
/// # Usage
///
/// ```rust
/// let controller = Controller::default();
/// let watcher = controller.watcher();
///
/// tokio::spawn(async move {
///     watcher.wait().await.unwrap()
/// });
///
/// // ...
///
/// controller.send().unwrap();
/// ```
pub struct Controller {
    recv: Receiver<bool>,
    send: Sender<bool>,
}

pub struct Watcher {
    recv: Receiver<bool>,
}

impl Default for Controller {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::watch::channel(false);
        Self { recv: rx, send: tx }
    }
}

impl Controller {
    /// Create a new watcher instance which is used to await a shutdown event.
    ///
    /// # Usage
    ///
    /// ```rust
    /// let controller = Controller::default();
    /// let watcher = controller.watcher();
    ///
    /// tokio::spawn(async move {
    ///     watcher.wait().await.unwrap()
    /// });
    /// ```
    pub fn watcher(&self) -> Watcher {
        Watcher {
            recv: self.recv.clone(),
        }
    }

    /// Send a shutdown signal to a watcher.
    pub fn send(&self) -> Result<(), Error> {
        self.send.send(true).map_err(|_| Error::Closed)
    }
}

impl Watcher {
    /// Wait until shutdown signal is received.
    ///
    /// # Usage
    ///
    /// ```rust
    /// let controller = Controller::default();
    /// let watcher = controller.watcher();
    ///
    /// tokio::spawn(async move {
    ///     watcher.wait().await.unwrap()
    /// });
    ///
    /// // ...
    ///
    /// controller.send().unwrap();
    /// ```
    pub async fn wait(&mut self) -> Result<(), Error> {
        self.recv.changed().await.map_err(|_| Error::Closed)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Controller;

    #[test]
    fn default() {
        let _ = Controller::default();
    }

    #[tokio::test]
    async fn wait_for_signal() {
        let controller = Controller::default();
        let mut watcher = controller.watcher();

        let handle = tokio::spawn(async move { watcher.wait().await.unwrap() });

        tokio::time::sleep(Duration::from_secs(1)).await;
        controller.send().unwrap();
        handle.await.unwrap()
    }
}
