pub mod builder;
pub mod registry;

/// Used to manage and maintain services.
pub struct Runtime {}

impl Runtime {
    /// Returns new runtime builder.
    pub fn builder() -> builder::Builder {
        builder::Builder::default()
    }

    /// Start runtime and wait for shutdown signal
    pub async fn run(self) {
        todo!()
    }
}
