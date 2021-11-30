use tonic::{transport::Server, Request, Response, Status};

use replication::replication_server::{Replication, ReplicationServer};
use replication::{
    GetEntriesDeltaRequest, GetEntriesDeltaResponse, GetEntriesRequest, GetEntriesResponse,
};

pub mod replication {
    tonic::include_proto!("replication"); // The string specified here must match the proto package name
}

#[derive(Debug, Default)]
pub struct MyReplication {}

#[tonic::async_trait]
impl Replication for MyReplication {
    async fn get_entries(
        &self,
        request: Request<GetEntriesRequest>,
    ) -> Result<Response<GetEntriesResponse>, Status> {
        println!("Got a request: {:?}", request);

        let entries = vec![];

        let reply = GetEntriesResponse { entries };
        Ok(Response::new(reply))
    }

    async fn get_entries_delta(
        &self,
        request: Request<GetEntriesDeltaRequest>,
    ) -> Result<Response<GetEntriesDeltaResponse>, Status> {
        println!("Got a request: {:?}", request);

        let author_known_log_heights = vec![];
        let reply = GetEntriesDeltaResponse{
            author_known_log_heights
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = MyReplication::default();

    Server::builder()
        .add_service(ReplicationServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
