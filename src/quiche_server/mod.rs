use quiche::{Config as QuicheConfig, ConnectionId};
use std::net::SocketAddr;
use tonic::{
    body::BoxBody, codegen::http::Request as CodegenRequest,
    codegen::http::Response as CodegenResponse, codegen::Never, codegen::Service, transport::Body,
    transport::NamedService,
};
use p2panda_replication::Request;

#[derive(Debug)]
pub struct QuicheServer<S> {
    service: S,
}

impl<S> QuicheServer<S>
where
    S: Service<CodegenRequest<Body>, Response = CodegenResponse<BoxBody>, Error = Never>
        + NamedService
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    pub fn new(service: S) -> Self {
        Self { service }
    }

    pub async fn serve(mut self, addr: SocketAddr) {
        let config = QuicheConfig::new(quiche::PROTOCOL_VERSION);
        let addr_bytes = match addr.ip() {
            std::net::IpAddr::V4(v4) => v4.octets(),
            std::net::IpAddr::V6(_) => {
                unimplemented!()
            }
        };
        let scid = ConnectionId::from_ref(&addr_bytes);

        let req = Request{
            path: "/path",
            data: &vec![1,2,3]
        };

        self.service.call(req.into());
    }
}
