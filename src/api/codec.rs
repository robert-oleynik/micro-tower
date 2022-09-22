use bytes::buf::{Reader, Writer};
use bytes::BytesMut;

mod json;

pub use json::Json;

pub trait Decode<T> {
    type Error;

    fn decode(reader: &mut Reader<BytesMut>) -> Result<T, Self::Error>;
}

pub trait Encode<T> {
    type Error;

    fn encode(writer: &mut Writer<BytesMut>, message: T) -> Result<(), Self::Error>;
}
