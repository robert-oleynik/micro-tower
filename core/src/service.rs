/// Used return a service of type `S` from a multi service container.
pub trait GetByName<Name> {
    type Service;

    /// Returns a reference to a service of type `S`.
    fn get(&self) -> &Self::Service;
}

pub trait Create {
    type Service;

    fn create() -> Self::Service;
}
