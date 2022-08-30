use derive_builder::Builder;
use derive_builder::UninitializedFieldError;

pub mod manifest;

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

    pub fn manifest<M: manifest::Create>(self) -> Runtime<M> {
        let manifest = self.runtime.block_on(async move { M::create() });
        Runtime {
            runtime: self.runtime,
            manifest,
        }
    }
}

impl<M: manifest::Create> Runtime<M> {
    pub fn run(self) {
        let manifest = self.manifest;
        self.runtime.block_on(async move {
            let _ = manifest;
        })
    }
}
