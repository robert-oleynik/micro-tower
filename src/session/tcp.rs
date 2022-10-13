use std::marker::PhantomData;
use std::net::SocketAddr;

use tokio::net::TcpListener;
use tower::{BoxError, Layer, Service, ServiceExt};

use crate::api::codec::{Decode, Encode};
use crate::api::Message;
use crate::shutdown::Controller;
use crate::util::BoxFuture;

pub struct Session<ED, Req> {
    addr: SocketAddr,
    listener: TcpListener,
    _p: PhantomData<(Req, ED)>,
}

impl<ED, Req> Session<ED, Req> {
    /// Create tcp session that binds to address `addr`.
    ///
    /// # Errors
    ///
    /// Will return `Err` if failed to create tcp listener.
    pub async fn with_addr(addr: SocketAddr) -> std::io::Result<Self> {
        let listener = TcpListener::bind(&addr).await?;
        Ok(Self {
            addr,
            listener,
            _p: PhantomData,
        })
    }
}

impl<SB, ED, Req> super::Session<SB> for Session<ED, Req>
where
    Req: Send + 'static,
    SB: Service<SocketAddr, Error = BoxError> + Send + 'static,
    SB::Future: Send,
    SB::Response: Service<Req, Error = BoxError> + Send,
    <SB::Response as Service<Req>>::Future: Send,
    ED: Encode<Message<<SB::Response as tower::Service<Req>>::Response>>
        + Decode<Req>
        + Send
        + 'static,
    <ED as Encode<Message<<SB::Response as tower::Service<Req>>::Response>>>::Error:
        std::error::Error + Send + Sync + 'static,
    <ED as Decode<Req>>::Error: std::error::Error + Send + Sync + Unpin + 'static,
{
    fn run(self, mut builder: SB, controller: Controller) -> BoxFuture<Result<(), BoxError>> {
        Box::pin(async move {
            let addr = self.addr;
            let listener = self.listener;
            tracing::info!(message = "listening on", port = addr.port());

            loop {
                tracing::trace!(message = "wait for new connections", port = addr.port());

                let (stream, addr) = tokio::select! {
                    result = listener.accept() => result?,
                    _ = controller.wait_for_shutdown() => return Ok(())
                };

                let service = match builder.ready().await {
                    Ok(service) => service,
                    Err(err) => {
                        let report = crate::report!(err.as_ref());
                        tracing::error!("{report:?}");
                        continue;
                    }
                };
                let service = match service.call(addr).await {
                    Ok(service) => service,
                    Err(err) => {
                        let report = crate::report!(err.as_ref());
                        tracing::error!("{report:?}");
                        continue;
                    }
                };
                let layer = crate::api::Layer::<Req, ED>::default();
                let service = layer.layer(service);

                tracing::info!(message = "new connection", addr = format!("{addr}"));

                let controller = controller.clone();
                tokio::spawn(async move {
                    if let Err(err) = super::stream::spawn_fut(stream, service, controller).await {
                        let report = crate::report!(err.as_ref());
                        tracing::error!("{report:?}");
                    }
                });
            }
        })
    }
}
