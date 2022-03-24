use log::{error, trace, warn};
use qp2p::{Config, ConnectionError, Endpoint, EndpointError, IncomingConnections};
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use crate::server::{ReplicationService, P2pandaServer};
// TODO remove me
// qp2p needs to know about its peers to initiate connections
pub struct Qp2pServer<S> 
where
    S: ReplicationService,
{
    pub peers: Vec<SocketAddr>,
    pub incoming_conns: IncomingConnections,
    pub p2panda_server: P2pandaServer<S>,
}


impl<S> Qp2pServer<S>
where
    S: ReplicationService + 'static,
{
    // TODO: peers? remove?
    pub async fn new(service: S, peers: Vec<SocketAddr>) -> Result<(Self,Endpoint) , EndpointError> {
        trace!("new qp2p server");
        let (endpoint, incoming_conns, _contact) = Endpoint::new_peer(
            SocketAddr::from((Ipv4Addr::LOCALHOST, 8099)),
            &[],
            Config {
                idle_timeout: Duration::from_secs(60 * 60).into(), // 1 hour idle timeout.
                ..Default::default()
            },
        )
        .await?;

        let local_addr = endpoint.local_addr();
        let public_addr = endpoint.public_addr();

        trace!(
            "started qp2p endpoint with local_addr: {:?}, public_addr: {:?}",
            local_addr,
            public_addr
        );

        let p2panda_server = P2pandaServer::new(service);

        Ok((Qp2pServer {
            peers,
            incoming_conns,
            p2panda_server,
        }, endpoint ))
    }

    pub async fn serve(mut self) -> Result<(), ConnectionError> {
        loop {
            let (connection, mut incoming_messages) = match self.incoming_conns.next().await {
                Some((connection, incoming_messages)) => {
                    trace!("opened connection!");
                    (connection, incoming_messages)
                }
                None => {
                    error!("connection open failed");
                    break;
                }
            };

            loop {
                match incoming_messages.next().await {
                    Ok(Some(bytes)) => {
                        trace!("received bytes: {:?}", bytes);
                        let response = self.p2panda_server.handle_request(&bytes).await;
                        trace!("called gprc method, response was: {:?}", response);

                        match connection.send(response.into()).await {
                            Ok(_) => trace!("sent response ok"),
                            Err(err) => error!("error sending response to peer: {:?}", err),
                        };
                    }
                    Ok(None) => {
                        trace!("no more messages from remote peer");
                        break;
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
