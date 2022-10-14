use bytes::buf::{Reader, Writer};
use bytes::BytesMut;

mod json;

pub use json::Json;

pub trait Decode<T> {
	type Error;

	/// Read bytes from byte buffer and decode them to the structure `T`.
	///
	/// # Errors
	///
	/// Will return `Err` if bytes of buffer cannot be decoded to structure `T`.
	fn decode(reader: &mut Reader<BytesMut>) -> Result<T, Self::Error>;
}

pub trait Encode<T> {
	type Error;

	/// Encode message structure `T` and write the encoded message to byte buffer.
	///
	/// # Errors
	///
	/// Will return `Err` if `message` cannot be encoded.
	fn encode(writer: &mut Writer<BytesMut>, message: T) -> Result<(), Self::Error>;
}
