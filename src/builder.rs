//! Extensions to improve [`tower::ServiceBuilder`].

use crate::service::pool;
use crate::{api, layer};
use tower::layer::util::Stack;
use tower::ServiceBuilder;

pub trait ServiceBuilderExt<L> {
	/// Wrap service in [`api::Layer`]. Should be done to prepare service for sessions (e.g.
	/// [`crate::session::tcp::spawn`]).
	fn api<R, C>(self) -> ServiceBuilder<Stack<api::Layer<R, C>, L>>;

	/// Wrap service in [`layer::future::BoxLayer`].
	fn boxed_future(self) -> ServiceBuilder<Stack<layer::future::BoxLayer, L>>;
}

pub trait ServicePoolBuilderExt<L> {
	/// Wraps the make service to create a pool with `count` many services.
	fn pooled<T, Req>(
		self,
		count: usize,
		target: T,
	) -> ServiceBuilder<Stack<pool::Layer<T, Req>, L>>;
}

impl<L> ServiceBuilderExt<L> for ServiceBuilder<L> {
	fn api<R, C>(self) -> ServiceBuilder<Stack<api::Layer<R, C>, L>> {
		self.layer(api::Layer::default())
	}

	fn boxed_future(self) -> ServiceBuilder<Stack<layer::future::BoxLayer, L>> {
		self.layer(layer::future::BoxLayer::default())
	}
}

impl<L> ServicePoolBuilderExt<L> for ServiceBuilder<L> {
	fn pooled<T, Req>(
		self,
		count: usize,
		target: T,
	) -> ServiceBuilder<Stack<pool::Layer<T, Req>, L>> {
		self.layer(pool::Layer::with_size(count, target))
	}
}
