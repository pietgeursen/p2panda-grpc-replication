use tonic::transport::Body;
use http::{Uri, StatusCode};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request<'a, 'b>{
    pub path: &'a str,
    pub data: &'b [u8] 
}

impl<'a, 'b> From<Request<'a, 'b>> for http::Request<Body>{
    fn from(req: Request) -> Self {
        http::Request::builder()
            .uri(req.path)
            .body(req.data.to_vec().into())
            .unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<'a, 'b>{
    pub body: Option<&'a [u8]>,
    pub status: u16,
    pub grpc_status: Option<&'b str>,
}


