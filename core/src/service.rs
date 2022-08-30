/// Used return a service of type `S` from a multi service container.
pub trait GetService<S> {
    /// Returns a reference to a service of type `S`.
    fn get(&self) -> &S;
}

pub trait CreateService {
    type Service;

    fn create() -> Self::Service;
}
