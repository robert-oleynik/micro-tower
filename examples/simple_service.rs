#![feature(error_reporter)]

use micro_tower::api::codec;
use micro_tower::runtime::Runtime;
use micro_tower::session::tcp;
use std::num::ParseIntError;

/// Service documentation
#[micro_tower::codegen::service(buffer = 24, pool = 4)]
pub async fn parse_str(request: String) -> Result<i32, ParseIntError> {
	request.parse()
}

#[micro_tower::codegen::main]
async fn tower() -> _ {
	let addr = "127.0.0.1:4000".parse().unwrap();
	let session = tcp::Session::<codec::Json, _>::with_addr(addr)
		.await
		.unwrap();

	Runtime::builder()
		.bind_service::<parse_str, _>(session)
		.build()
		.await
}
