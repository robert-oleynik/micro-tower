use proc_macro::{Diagnostic, Level};
use syn::parse::Parse;

use crate::utils;

use self::signature::Signature;

mod args;
mod signature;

/// Parse proc attribute items.
pub struct Items {
    signature: Signature,
    block: Box<syn::Block>,
}

pub struct Service {
    args: args::Args,
    request: syn::Type,
    request_arg: syn::FnArg,
    response: syn::Type,
    service_dependencies: Vec<syn::FnArg>,
    code_block: Box<syn::Block>,
}

impl Service {
    pub fn new(args: syn::AttributeArgs, items: Items) -> syn::Result<Self> {
        let args = args::Args::try_from(args)?;
        todo!()
    }
}

// impl Items {
//     pub fn generate(self) -> TokenStream {
//         let pub_token = self.signature.pub_token();
//         let ident = self.signature.ident();
//         let block = self.block;
//         let request_arg = self.signature.request_arg();
//         let request_type = self.signature.request_type();
//         let response_type = self.signature.response_type();
//         let ret = match self.signature.ret_result() {
//             true => quote::quote!(Ok(result?)),
//             false => quote::quote!(Ok(result)),
//         };
//         let output = self.signature.output();
//         quote::quote!(
//             #[allow(non_camel_case_types)]
//             #[derive(::std::clone::Clone)]
//             #pub_token struct #ident;

//             impl #ident {
//                 async fn handle(#request_arg) -> #output #block
//             }

//             impl ::micro_tower::service::Create for #ident {
//                 type Service = ::micro_tower::tower::util::BoxCloneService<#request_type, #response_type, ::micro_tower::tower::BoxError>;

//                 fn create() -> Self::Service {
//                     ::micro_tower::tower::ServiceBuilder::new()
//                         .boxed_clone()
//                         .service(#ident)
//                 }
//             }

//             impl ::micro_tower::tower::Service< #request_type > for #ident {
//                 type Response = #response_type;
//                 type Error = ::micro_tower::tower::BoxError;
//                 type Future = ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

//                 fn poll_ready(&mut self, cx: &mut ::std::task::Context<'_>) -> ::std::task::Poll<Result<(), Self::Error>> {
//                     ::std::task::Poll::Ready(Ok(()))
//                 }

//                 fn call(&mut self, request: #request_type) -> Self::Future {
//                     ::std::boxed::Box::pin(async move {
//                         let result = Self::handle(request).await;
//                         #ret
//                     })
//                 }
//             }
//         )
//         .into()
//     }
// }

impl Parse for Items {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            signature: input.parse()?,
            block: input.parse()?,
        })
    }
}
