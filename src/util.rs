pub trait Buildable {
    type Builder;

    fn builder() -> Self::Builder;
}
