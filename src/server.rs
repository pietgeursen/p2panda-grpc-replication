use futures::Stream;
use std::net::SocketAddr;
use std::pin::Pin;
use tonic::{
    Request, Response, Status,
};
use std::net::Ipv4Addr;

use replication::replication_server::{Replication, ReplicationServer};
use replication::*;

pub mod replication {
    tonic::include_proto!("replication"); // The string specified here must match the proto package name
}

mod qp2p_server;
mod quiche_server;

use quiche_server::QuicheServer;
use qp2p_server::Qp2pServer;

type ResponseStream =
    Pin<Box<dyn Stream<Item = Result<NewLogHeightsStreamResponse, Status>> + Send>>;

#[derive(Debug, Default)]
pub struct MyReplication {}

#[tonic::async_trait]
impl Replication for MyReplication {
    async fn get_all_entries_by_authors(
        &self,
        request: Request<GetAllEntriesByAuthorsRequest>,
    ) -> Result<Response<GetEntriesResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = GetEntriesResponse { entries: vec![] };
        Ok(Response::new(reply))
    }
    async fn get_single_entry(
        &self,
        request: Request<GetSingleEntryRequest>,
    ) -> Result<Response<GetEntriesResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = GetEntriesResponse { entries: vec![] };
        Ok(Response::new(reply))
    }

    async fn get_entries_by_sequence_range(
        &self,
        request: Request<GetEntriesBySequenceRangeRequest>,
    ) -> Result<Response<GetEntriesResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = GetEntriesResponse { entries: vec![] };
        Ok(Response::new(reply))
    }

    async fn get_log_height_deltas(
        &self,
        request: Request<GetLogHeightDeltasRequest>,
    ) -> Result<Response<GetLogHeightDeltasResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = GetLogHeightDeltasResponse {
            author_known_log_heights: vec![],
        };
        Ok(Response::new(reply))
    }

    async fn new_log_heights_stream(
        &self,
        _request: Request<NewLogHeightsStreamRequest>,
    ) -> Result<Response<Self::NewLogHeightsStreamStream>, Status> {
        Err(Status::unimplemented(
            "New log heights streams are not supported",
        ))
    }

    type NewLogHeightsStreamStream = ResponseStream;

    async fn set_author_aliases(
        &self,
        request: Request<SetAuthorAliasesRequest>,
    ) -> Result<Response<SetAuthorAliasesResponse>, Status> {
        println!("Got a request: {:?}", request);

        Err(Status::unimplemented("Author aliases are not supported"))
    }

    async fn check_author_alias_uuid_is_valid(
        &self,
        request: Request<CheckAuthorAliasUuidIsValidRequest>,
    ) -> Result<Response<CheckAuthorAliasUuidIsValidResponse>, Status> {
        println!("Got a request: {:?}", request);

        Err(Status::unimplemented("Author aliases are not supported"))
    }
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();



    let addr: SocketAddr = "[::1]:50051".parse()?;
    let greeter = MyReplication::default();
    let service = ReplicationServer::new(greeter);

    //Server::builder().add_service(service).serve(addr).await?;
    //QuicheServer::new(service).serve(addr).await;
    
    let peers = vec![SocketAddr::from((Ipv4Addr::LOCALHOST, 8099))];
    Qp2pServer::new(service, peers).await.unwrap().serve().await.unwrap();

    Ok(())
}
