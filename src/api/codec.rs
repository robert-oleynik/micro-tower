use bytes::buf::{Reader, Writer};
use bytes::BytesMut;

pub trait Decode<T> {
    type Error;

    fn decode(stream: &mut Reader<BytesMut>) -> Result<T, Self::Error>;
}

pub trait Encode<T> {
    type Error;

    fn encode(stream: &mut Writer<BytesMut>, message: T) -> Result<(), Self::Error>;
}
