use std::any::TypeId;
use std::ops::{Deref, DerefMut};

use crate::utils::{Named, TypeRegistry};

/// Used return a service of type `S` from a multi service container.
pub trait GetByName<S: Create> {
    /// Returns a reference to a service of type `S`.
    fn get(&self) -> Option<Service<S>>;
}

/// Provides interface to create services. Used by `codegen`
pub trait Create: crate::utils::Named {
    type Service;

    fn deps() -> &'static [TypeId];

    fn create(registry: &TypeRegistry) -> Self::Service;
}

/// Wrapper around services.
///
/// # Usage
///
/// ```rust
/// #[micro_tower::codegen::service]
/// async fn hello_world(_: ()) -> &'static str {
///     "Hello, World!"
/// }
///
/// #[micro_tower::codegen::service]
/// async fn hello_world2(_: (), other: Service<hello_world>) -> &'static str {
///     let result = other.ready().await.call(()).await.unwrap();
///     result
/// }
/// ```
#[derive(Clone)]
pub struct Service<S: Create> {
    inner: S::Service,
}

impl<S: Create> Service<S> {
    pub fn from_service(inner: S::Service) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> S::Service {
        self.inner
    }
}

impl<S: Create> Deref for Service<S> {
    type Target = S::Service;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S: Create> DerefMut for Service<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<S: Create + 'static> Named for Service<S> {
    fn name() -> TypeId {
        TypeId::of::<S>()
    }
}
