use log::{error, trace};
use p2panda_replication::{Request, Response};
use qp2p::{Config, ConnectionError, Endpoint, EndpointError, IncomingConnections};
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use tonic::codegen::Body as CodegenBody;
use tonic::{
    body::BoxBody, codegen::http::Request as CodegenRequest,
    codegen::http::Response as CodegenResponse, codegen::Never, codegen::Service, transport::Body,
    transport::NamedService,
};

// qp2p needs to know about its peers to initiate connections
pub struct Qp2pServer<S> {
    endpoint: Endpoint,
    peers: Vec<SocketAddr>,
    incoming_conns: IncomingConnections,
    service: S,
}

impl<S> Qp2pServer<S>
where
    S: Service<CodegenRequest<Body>, Response = CodegenResponse<BoxBody>, Error = Never>
        + NamedService
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    pub async fn new(service: S, peers: Vec<SocketAddr>) -> Result<Self, EndpointError> {
        let (endpoint, incoming_conns, _contact) = Endpoint::new_peer(
            SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            &peers,
            Config {
                idle_timeout: Duration::from_secs(60 * 60).into(), // 1 hour idle timeout.
                ..Default::default()
            },
        )
        .await?;

        let local_addr = endpoint.local_addr();
        let public_addr = endpoint.public_addr();

        trace!("started qp2p endpoint with local_addr: {:?}, public_addr: {:?}", local_addr, public_addr);

        Ok(Qp2pServer {
            endpoint,
            peers,
            incoming_conns,
            service,
        })
    }

    pub async fn serve(mut self) -> Result<(), ConnectionError> {
        loop {
            let connection = match self.incoming_conns.next().await {
                Some((connection, _)) => {
                    trace!("opened connection!");
                    connection
                }
                None => {
                    error!("connection open failed");
                    break;
                }
            };

            // We expect _them_ to initiate requests and we'll reply
            let (mut send, mut receive) = connection.open_bi().await?;

            loop {
                match receive.next().await {
                    Ok(bytes) => {
                        trace!("received bytes: {:?}", bytes);
                        let json_req: Request = serde_json::from_slice(&bytes).unwrap();
                        trace!("json_req: {:?}", json_req);
                        let mut response = self.service.call(json_req.into()).await;
                        trace!("called gprc method, response was: {:?}", response);

                        let encoded_response = match response.as_mut() {
                            Ok(res) => {

                                let mut body_data = Vec::<u8>::new();

                                loop {
                                    if res.body().is_end_stream(){
                                        break;
                                    }
                                    let data = res.body_mut().data().await;

                                    match data{
                                        Some(Ok(data)) => {
                                            body_data.extend(&data.to_vec());
                                        },
                                        _ => break
                                    }
                                }

                                let res = Response {
                                    body: Some(&body_data),
                                    status: res.status().as_u16(),
                                    grpc_status: res
                                        .headers()
                                        .get("grpc-status")
                                        .and_then(|val| val.to_str().ok()),
                                };
                                serde_json::to_vec(&res).unwrap()
                            }
                            Err(err) => {
                                error!("error from calling grpc handler {:?}", err);
                                break;
                            }
                        };

                        match send.send_user_msg(encoded_response.into()).await {
                            Ok(_) => trace!("sent response ok"),
                            Err(err) => error!("error sending response to peer: {:?}", err),
                        };
                    }
                    Err(err) => {
                        error!("Error receiving from stream: {:?}", err);
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}

//#[tokio::main]
//async fn main() -> Result<(), EndpointError> {
//    println!("Hello, world!");
//    let peers = vec![SocketAddr::from((Ipv4Addr::LOCALHOST, 0))];
//    let mut server = Server::connect_to_peers(peers).await?;
//
//    Ok(())
//}
