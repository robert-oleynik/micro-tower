use tower::layer::util::Stack;
use tower::ServiceBuilder;

use crate::api;

pub trait ServiceBuilderExt<L> {
    /// Wrap service in [`api::Layer`]. Should be done to prepare service for sessions (e.g.
    /// [`micro_tower::sessions::tcp::spawn`]).
    fn api<R, C>(self) -> ServiceBuilder<Stack<api::Layer<R, C>, L>>;
}

impl<L> ServiceBuilderExt<L> for ServiceBuilder<L> {
    fn api<R, C>(self) -> ServiceBuilder<Stack<api::Layer<R, C>, L>> {
        self.layer(api::Layer::default())
    }
}
