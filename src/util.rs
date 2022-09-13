pub trait Buildable {
    type Target;
    type Builder;

    fn builder() -> Self::Builder;
}
