use futures::Stream;
use replication::server::Replication;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::pin::Pin;

use doggo_build::async_trait;
use log::trace;
use replication::*;

pub mod replication {
    doggo_build::include_proto!("replication");
    //tonic::include_proto!("replication"); // The string specified here must match the proto package name
}

mod qp2p_server;

use qp2p_server::Qp2pServer;

type ResponseStream =
    Pin<Box<dyn Stream<Item = Result<NewLogHeightsStreamResponse, Status>> + Send>>;

#[derive(Debug, Default)]
pub struct MyReplication {}

use futures::channel::mpsc::Receiver;
impl MyReplication {
    async fn with_streaming_request(&mut self, messages: Receiver<u8>) {}
}

impl MyReplication {
    async fn get_all_entries_by_authors(
        &mut self,
        request: GetAllEntriesByAuthorsRequest,
    ) -> Result<GetEntriesResponse, Status> {
        println!("Got a request: {:?}", request);
        let reply = GetEntriesResponse { entries: vec![] };
        Ok(reply)
    }
}

#[doggo_build::async_trait]
impl Replication for MyReplication {
    async fn get_all_entries_by_authors(
        &mut self,
        request: GetAllEntriesByAuthorsRequest,
    ) -> Result<GetEntriesResponse, Status> {
        println!("here");
        self.get_all_entries_by_authors(request).await
    }
    async fn get_single_entry(
        &mut self,
        request: GetSingleEntryRequest,
    ) -> Result<GetEntriesResponse, Status> {
        println!("Got a request: {:?}", request);
        let reply = GetEntriesResponse { entries: vec![] };
        Ok(reply)
    }
    async fn get_entries_by_sequence_range(
        &mut self,
        request: GetEntriesBySequenceRangeRequest,
    ) -> Result<GetEntriesResponse, Status> {
        println!("Got a request: {:?}", request);
        let reply = GetEntriesResponse { entries: vec![] };
        Ok(reply)
    }
    async fn get_log_height_deltas(
        &mut self,
        request: GetLogHeightDeltasRequest,
    ) -> Result<GetLogHeightDeltasResponse, Status> {
        todo!()
    }
    async fn new_log_heights_stream(
        &mut self,
        request: NewLogHeightsStreamRequest,
    ) -> Result<futures::channel::mpsc::Receiver<NewLogHeightsStreamResponse>, Status> {
        todo!()
    }

    async fn set_author_aliases(
        &mut self,
        request: SetAuthorAliasesRequest,
    ) -> Result<SetAuthorAliasesResponse, Status> {
        todo!()
    }
    async fn check_author_alias_uuid_is_valid(
        &mut self,
        request: CheckAuthorAliasUuidIsValidRequest,
    ) -> Result<CheckAuthorAliasUuidIsValidResponse, Status> {
        todo!()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    trace!("startingggggggggggg !!!!!!!!");

    let addr: SocketAddr = "[::1]:50051".parse()?;
    //let greeter = MyReplication::default();
    //let service = ReplicationService::new(greeter);
    let service = MyReplication {};

    //Server::builder().add_service(service).serve(addr).await?;
    //QuicheServer::new(service).serve(addr).await;

    let peers = vec![SocketAddr::from((Ipv4Addr::LOCALHOST, 8099))];
    let (server, mut endpoint) = Qp2pServer::new(service, peers).await.unwrap();

    // TODO: just for debugging.
    tokio::spawn(async move {
        let (conn, mut incoming) = endpoint
            .connect_to(&SocketAddr::from((Ipv4Addr::LOCALHOST, 8099)))
            .await
            .unwrap();

        let msg = GetSingleEntryRequest {
            author: None,
            log_id: 23,
            sequence: 3434,
            aliases_uuid: "biglonguuid".to_owned(),
        };

        //conn.send(serde_json::to_vec(&req).unwrap().into()).await;
        //let response = incoming.next().await;
        //trace!("response: {:?}", response);
    });

    server.serve().await.unwrap();

    Ok(())
}
