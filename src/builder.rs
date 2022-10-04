use tower::layer::util::Stack;
use tower::ServiceBuilder;

use crate::api;
use crate::service::pool;

pub trait ServiceBuilderExt<L> {
    /// Wrap service in [`api::Layer`]. Should be done to prepare service for sessions (e.g.
    /// [`micro_tower::sessions::tcp::spawn`]).
    fn api<R, C>(self) -> ServiceBuilder<Stack<api::Layer<R, C>, L>>;
}

pub trait ServicePoolBuilderExt<L> {
    /// Wraps the make service to create a pool with `count` many services.
    fn pooled<Req>(self, count: usize) -> ServiceBuilder<Stack<pool::Layer<Req>, L>>;
}

impl<L> ServiceBuilderExt<L> for ServiceBuilder<L> {
    fn api<R, C>(self) -> ServiceBuilder<Stack<api::Layer<R, C>, L>> {
        self.layer(api::Layer::default())
    }
}

impl<L> ServicePoolBuilderExt<L> for ServiceBuilder<L> {
    fn pooled<Req>(self, count: usize) -> ServiceBuilder<Stack<pool::Layer<Req>, L>> {
        self.layer(pool::Layer::with_size(count))
    }
}
