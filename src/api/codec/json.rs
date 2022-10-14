use super::{Decode, Encode};
use bytes::buf::{Reader, Writer};
use bytes::BytesMut;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct Json;

impl<T: DeserializeOwned> Decode<T> for Json {
	type Error = serde_json::Error;

	fn decode(reader: &mut Reader<BytesMut>) -> Result<T, Self::Error> {
		serde_json::from_reader(reader)
	}
}

impl<T: Serialize> Encode<T> for Json {
	type Error = serde_json::Error;

	fn encode(writer: &mut Writer<BytesMut>, message: T) -> Result<(), Self::Error> {
		serde_json::to_writer(writer, &message)
	}
}
