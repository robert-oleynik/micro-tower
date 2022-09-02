use std::ops::{Deref, DerefMut};

/// Used return a service of type `S` from a multi service container.
pub trait GetByName<Name> {
    type Service;

    /// Returns a reference to a service of type `S`.
    fn get(&self) -> &Self::Service;
}

/// Provides interface to create services. Used by `codegen`
pub trait Create {
    type Service;

    fn create() -> Self::Service;
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
pub struct Service<S: Create> {
    service: S::Service,
}

impl<S: Create> Service<S> {
    /// Create a new wrapped service from an existing one.
    pub fn from_service(service: S::Service) -> Self {
        Self { service }
    }
}

impl<S: Create> Deref for Service<S> {
    type Target = S::Service;

    fn deref(&self) -> &Self::Target {
        &self.service
    }
}

impl<S: Create> DerefMut for Service<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.service
    }
}
