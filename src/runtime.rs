use derive_builder::Builder;
use derive_builder::UninitializedFieldError;
use tokio::task::JoinHandle;

#[derive(Builder)]
#[builder(build_fn(skip), pattern = "owned")]
pub struct Runtime<M> {
    runtime: tokio::runtime::Runtime,
    #[builder(setter(skip))]
    manifest: M,
}

impl RuntimeBuilder<()> {
    pub fn build(self) -> Result<Runtime<()>, UninitializedFieldError> {
        let rt = self
            .runtime
            .ok_or(UninitializedFieldError::new("runtime"))?;
        Ok(Runtime {
            runtime: rt,
            manifest: (),
        })
    }
}

impl Runtime<()> {
    pub fn builder() -> RuntimeBuilder<()> {
        RuntimeBuilder::default()
    }

    pub fn manifest<M>(self, f: impl FnOnce() -> M) -> Runtime<M> {
        let manifest = self.runtime.block_on(async move { f() });
        Runtime {
            runtime: self.runtime,
            manifest,
        }
    }
}

impl<M> Runtime<M> {
    pub fn run(self, f: impl FnOnce(M) -> Vec<JoinHandle<()>>) {
        let manifest = self.manifest;
        self.runtime.block_on(async move {
            tracing::info!("start runtime");
            for (i, j) in f(manifest).into_iter().enumerate() {
                if let Err(err) = j.await {
                    tracing::error!(
                        message = "Failed to wait for job to finish",
                        error = format!("{err}")
                    )
                }
                tracing::trace!("job {i} stopped")
            }
            tracing::info!("exiting runtime")
        })
    }
}
