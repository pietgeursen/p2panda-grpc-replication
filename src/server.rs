use tonic::{transport::Server, Request, Response, Status};

use replication::replication_server::{Replication, ReplicationServer};
use replication::*;

pub mod replication {
    tonic::include_proto!("replication"); // The string specified here must match the proto package name
}

#[derive(Debug, Default)]
pub struct MyReplication {}

#[tonic::async_trait]
impl Replication for MyReplication {

    async fn get_author_aliases(
        &self,
        request: Request<GetAuthorAliasesRequest>,
    ) -> Result<Response<GetAuthorAliasesResponse>, Status> {
        println!("Got a request: {:?}", request);

        let error = AuthorAliasesNotSupportedError{};
        let error_type = Some(get_author_aliases_response::errors::ErrorType::AuthorAliasesNotSupported(error));

        let reply = GetAuthorAliasesResponse { response: Some(get_author_aliases_response::Response::Error(get_author_aliases_response::Errors{error_type})) };
        Ok(Response::new(reply))
    }
    async fn get_all_entries_by_authors(
        &self,
        request: Request<GetAllEntriesByAuthorsRequest>,
    ) -> Result<Response<GetEntriesResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = GetEntriesResponse { response: None };
        Ok(Response::new(reply))
    }

    async fn get_single_entry(
        &self,
        request: Request<GetSingleEntryRequest>,
    ) -> Result<Response<GetEntriesResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = GetEntriesResponse { response: None };
        Ok(Response::new(reply))
    }

    async fn get_entries_by_sequence_range(
        &self,
        request: Request<GetEntriesBySequenceRangeRequest>,
    ) -> Result<Response<GetEntriesResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = GetEntriesResponse { response: None };
        Ok(Response::new(reply))
    }

    async fn get_entries_delta(
        &self,
        request: Request<GetEntriesDeltaRequest>,
    ) -> Result<Response<GetEntriesDeltaResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = GetEntriesDeltaResponse { response: None };
        Ok(Response::new(reply))
    }




//    async fn get_entries(
//        &self,
//        request: Request<GetEntriesRequest>,
//    ) -> Result<Response<GetEntriesResponse>, Status> {
//        println!("Got a request: {:?}", request);
//
//        let entries = vec![];
//
//        let reply = GetEntriesResponse { entries };
//        Ok(Response::new(reply))
//    }
//
//    async fn get_entries_delta(
//        &self,
//        request: Request<GetEntriesDeltaRequest>,
//    ) -> Result<Response<GetEntriesDeltaResponse>, Status> {
//        println!("Got a request: {:?}", request);
//        println!("author: {:?}", request.into_inner().author_known_log_heights);
//
//
//        let author_known_log_heights = vec![];
//        let reply = GetEntriesDeltaResponse {
//            author_known_log_heights,
//        };
//        Ok(Response::new(reply))
//    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = MyReplication::default();
    let service = ReplicationServer::new(greeter);


    Server::builder()
        .add_service(service)
        .serve(addr)
        .await?;

    Ok(())
}
