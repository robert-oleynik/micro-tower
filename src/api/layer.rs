use std::marker::PhantomData;

use super::Service;

/// Creates a layer which wraps a given service inside an api translation layer (see [`Service`]).
/// The api layer will translate requests of type `R` with codec `C`.
pub struct Layer<R, C = super::codec::Json> {
    _p: PhantomData<(R, C)>,
}

impl<R, C> Default for Layer<R, C> {
    fn default() -> Self {
        Self { _p: PhantomData }
    }
}

impl<R, C, S> tower::Layer<S> for Layer<R, C> {
    type Service = Service<R, C, S>;

    fn layer(&self, inner: S) -> Self::Service {
        Service::from_service(inner)
    }
}
