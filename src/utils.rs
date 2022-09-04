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
    type Target = Service<S>;

    fn get(&self) -> Option<Service<S>> {
        self.get(&S::name())
            .map(|service| service.downcast_ref::<S::Service>().unwrap())
            .map(|service| (*service).clone())
            .map(Service::from_service)
    }
}

impl<S: Create> GetByName<Service<S>> for TypeRegistry
where
    S::Service: Clone,
{
    type Target = Service<S>;

    fn get(&self) -> Option<Service<S>> {
        <TypeRegistry as GetByName<S>>::get(self)
    }
}
