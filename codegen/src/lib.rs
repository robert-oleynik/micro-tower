#![feature(proc_macro_diagnostic)]

use darling::FromMeta;
use proc_macro::TokenStream;
use syn::parse_macro_input;

mod service;

use service::Args;

/// Generates service implementation from specified function signature.
///
/// **Important:** The generated service is not a *function* but a *struct*
///
/// # Usage
///
/// ```rust
/// use micro_tower::service::Service;
///
/// #[micro_tower::codegen::service]
/// async fn service_name(request: Request) -> Response {
///     todo!()
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let service = Service::<service_name>::builder().build().unwrap();
/// }
/// ```
///
/// or with [`Result`]
///
/// ```rust
/// #[micro_tower::codegen::service]
/// async fn service_name(request: Request) -> Result<Response, Error> {
///     todo!()
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let service = Service::<service_name>::builder().build().unwrap();
/// }
/// ```
///
/// **Note:** `::std::result::Result` is **required** as return type to determine `Response` and
/// `Error` type
///
/// # Use other services
///
/// ```rust
/// use micro_tower::prelude::*;
/// use micro_tower::service::Service;
///
/// #[micro_tower::codegen::service]
/// async fn service_name(request: Request, other: Service<other_service>) -> Response {
///     other.ready().await.unwrap().call(request).await.unwrap()
/// }
///
/// #[micro_tower::codegen::service]
/// async fn other_service(request: Request) -> Response {
///     todo!()
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let other = Service::<other_service>::builder().build().unwrap();
///     let service = Service::<service_name>::builder()
///         .other(other)
///         .build()
///         .unwrap();
/// }
/// ```
///
/// It is important to call `Service::ready` before calling the service to ensure that the service
/// is ready to handle a new request.
///
/// # Options
///
/// Service `options` can be specified by
///
/// ```rust
/// #[micro_tower::codegen::service(<put your options here>)]
/// ```
///
/// ## Buffer requests
///
/// Option: `buffer = <len>`
///
/// Example:
///
/// ```rust
/// #[micro_tower::codegen::service(buffer = 64)]
/// async fn service_name(request: Request) -> Response {
///     todo!()
/// }
/// ```
///
/// ## Concurrency limit
///
/// Option: `concurrency = <limit>`
///
/// This option will limit the number of concurrent requests to `<limit>` and is set to `1` if
/// option `buffer` is set.
///
/// ```rust
/// #[micro_tower::codegen::service(concurrency = 16)]
/// async fn service_name(request: Request) -> Response {
///     todo!()
/// }
/// ````
///
/// ## Changing module path
///
/// Option: `crate = <path>`
///
/// This will set the module path of `::micro_tower` to `<path>`.
///
/// Example:
///
/// ```rust
/// use micro_tower as mt;
///
/// #[mt::codegen::service(crate = "mt")]
/// async fn service_name(request: Request) -> Response {
///     todo!()
/// }
/// ```
#[proc_macro_attribute]
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as syn::AttributeArgs);
    let items = parse_macro_input!(item as service::Items);
    let args = match Args::from_list(&args) {
        Ok(v) => {
            if let Err(err) = v.verify() {
                return err.to_compile_error().into();
            }
            v
        }
        Err(err) => return err.write_errors().into(),
    };

    service::generate(args, items).into()
}
