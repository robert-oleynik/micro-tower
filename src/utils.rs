use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::service::{Create, GetByName, Service};

pub trait Named: 'static {
    fn name() -> TypeId {
        TypeId::of::<Self>()
    }
}

pub type TypeRegistry = HashMap<TypeId, Box<dyn Any>>;

impl<S: Create> GetByName<S> for TypeRegistry
where
    S::Service: Clone,
{
    fn get(&self) -> Option<Service<S>> {
        self.get(&S::name())
            .map(|service| service.downcast_ref::<S::Service>().unwrap())
            .map(|service| (*service).clone())
            .map(Service::from_service)
    }
}
