use proc_macro2::{Ident, TokenStream};
use prost_build::{Config, Method, ServiceGenerator};
use quote::{format_ident, quote};
use std::io;
use std::path::Path;

pub struct Builder {}
impl Builder {
    /// Compile the .proto files and execute code generation.
    pub fn compile(
        self,
        protos: &[impl AsRef<Path>],
        includes: &[impl AsRef<Path>],
    ) -> io::Result<()> {
        let mut config = Config::new();
        config.service_generator(Box::new(ServiceGen {}));
        self.compile_with_config(config, protos, includes)
    }

    /// Compile the .proto files and execute code generation using a
    /// custom `prost_build::Config`.
    pub fn compile_with_config(
        self,
        mut config: Config,
        protos: &[impl AsRef<Path>],
        includes: &[impl AsRef<Path>],
    ) -> io::Result<()> {
        config.compile_protos(protos, includes)?;
        Ok(())
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {}
    }
}

pub struct ServiceGen {}

impl ServiceGenerator for ServiceGen {
    fn generate(&mut self, service: prost_build::Service, buf: &mut String) {
        let trait_name = format_ident!("{}", service.name);
        println!("{:?}", service);

        let requests = create_requests(&service);
        let server_service_trait = create_server_service_trait(&service, &trait_name);
        let panda_server = create_panda_server(&service, &trait_name);

        let tokens = quote! {
            #[derive(Debug, serde::Serialize, serde::Deserialize)]
            /// Our equivalent of the tonic::Status error type.
            pub enum Status {
                RequestDecodeError
            }

            // An enum that represents all the request types
            #requests

            pub mod server {
                use super::*;
                use prost::Message;

                // A type that represents the Server trait
                #server_service_trait

                #panda_server
            }

            // Create a type that represents the Client trait
            pub mod client {
                pub trait #trait_name{

                }
            }
        };

        let ast: syn::File = syn::parse2(tokens).expect("not a valid tokenstream");
        let code = prettyplease::unparse(&ast);
        buf.push_str(&code);
    }
}

fn create_panda_server(
    service: &prost_build::Service,
    trait_name: &Ident,
) -> proc_macro2::TokenStream {
    let match_arms = service
        .methods
        .iter()
        .map(|method| create_req_match_arm(method, &trait_name))
        .collect::<TokenStream>();

    let server_name = format_ident!("{}Server", service.name);

    quote!(

        pub type PinnedStream<T> = std::pin::Pin<std::boxed::Box<dyn futures::Stream<Item=T>>>;

        // If we have a type that implements #trait_name then we can pass it a request and
        // get a reponse from it.
        pub struct #server_name<S: #trait_name>{
            service: S,
        }

        impl<S: #trait_name> #server_name<S>{

            pub fn new(service: S) -> Self {
                Self{
                    service
                }
            }

            // TODO this might have to take incoming + outgoing connections and drive them itself.
            // Streaming is going to need framing and shit. probably asynchronous_codec
            pub async fn handle_request(&mut self, incoming: futures::channel::mpsc::Receiver<Vec<u8>>, outgoing: futures::channel::mpsc::Sender<Vec<u8>>) -> Vec<u8>
            {
                use futures::prelude::*;
                use futures::StreamExt;
                use futures::FutureExt;
                use futures::TryStreamExt;
                // The first packet will be a `Requests`.

                let request = match incoming.next().await{
                    Some(bytes) => {
                        serde_json::from_slice(&bytes)
                    }
                    None => {
                        return Self::encode_status_to_vec(&Status::RequestDecodeError);
                    }
                };

                if request.is_err() {
                    return Self::encode_status_to_vec(&Status::RequestDecodeError);
                }

                match request.unwrap() {
                    #match_arms
                }
            }

            fn encode_status_to_vec(status: &Status) -> Vec<u8>{
                serde_json::to_vec(&Result::<Vec<u8>, Status>::Err(Status::RequestDecodeError)).unwrap()
            }

        }
    )
}

fn create_requests(service: &prost_build::Service) -> proc_macro2::TokenStream {
    let request_variants = service
        .methods
        .iter()
        .map(create_req_enum_variant)
        .collect::<TokenStream>();

    quote! {
            // Create an enum that represents all the request types
            // This type will be serialized and sent over the wire by the client
            #[derive(Debug)]
            #[derive(serde::Serialize, serde::Deserialize)]
            pub enum Requests{
                #request_variants
            }
    }
}

fn create_server_service_trait(
    service: &prost_build::Service,
    trait_name: &Ident,
) -> proc_macro2::TokenStream {
    let server_methods = service
        .methods
        .iter()
        .map(create_server_method)
        .collect::<TokenStream>();

    quote!(
        #[doggo_build::async_trait]
        pub trait #trait_name{
            #server_methods
        }
    )
}

fn create_server_method(method: &Method) -> proc_macro2::TokenStream {
    let method_name = create_server_method_name(method);
    let req_type = create_server_method_request_type(method);
    let res_type = create_server_method_response_type(method);
    let req_stream_type = create_stream_type(&req_type);
    let res_stream_type = create_stream_type(&res_type);

    match (method.client_streaming, method.server_streaming) {
        (false, false) => {
            quote!( async fn #method_name(&mut self, request: #req_type) -> Result<#res_type, Status>; )
        }
        (true, false) => {
            quote!( async fn #method_name<REQ: #req_stream_type>(&mut self, request: REQ) -> Result<#res_type, Status>; )
        }
        (false, true) => {
            quote!( async fn #method_name<RES: #res_stream_type>(&mut self, request: #req_type) -> Result<RES, Status>; )
        }
        (true, true) => {
            quote!( async fn #method_name<REQ: #req_stream_type, RES: #res_stream_type>(&mut self, request: REQ) -> Result<RES, Status>; )
        }
    }
}

fn create_stream_type(item_type: &proc_macro2::TokenStream) -> proc_macro2::TokenStream{
    quote!(futures::Stream<Item=#item_type> + std::marker::Unpin)
}

fn create_server_method_name(method: &Method) -> proc_macro2::TokenStream {
    let method = format_ident!("{}", &method.name);
    quote!( #method )
}

fn create_server_method_request_type(method: &Method) -> proc_macro2::TokenStream {
    let typ = format_ident!("{}", &method.input_type);
    quote!( #typ )
}

fn create_server_method_response_type(method: &Method) -> proc_macro2::TokenStream {
    let typ = format_ident!("{}", &method.output_type);
    quote!( #typ )
}

fn create_req_enum_variant(method: &Method) -> proc_macro2::TokenStream {
    // TODO streaming request variants don't take the request as an argument, they'll be sent
    // later.
    let method_name = format_ident!("{}", &method.proto_name);

    quote!(
        #method_name,
    )
}

fn create_req_match_arm(method: &Method, service_trait: &Ident) -> proc_macro2::TokenStream {
    let input = format_ident!("{}", &method.input_type);
    let method_name = create_server_method_name(method);
    //let enum_variant = create_req_enum_variant(method);
    let enum_variant = format_ident!("{}", &method.proto_name);
    let req_type = create_server_method_request_type(method);

    match (method.client_streaming, method.server_streaming) {
        (false, false) => {
            quote!( Requests::#enum_variant => {
                let stream = incoming
                    .map(|bytes: Vec<u8>| -> Result<#req_type, Status> {
                        #req_type::decode(bytes.into())
                            .map_err(|_| Status::RequestDecodeError)
                    })
                    .and_then(|req|{
                        <S as server::#service_trait>::#method_name(&mut self.service, req)
                    })
                    .map_ok(|res|{
                        res.encode_to_vec()
                    });
                    
                let result = Box::pin(stream)
                    .next()
                    .await;

                serde_json::to_vec(&result).expect("expected to be able to encode to vec")
            },)
        }
        (true, false) => {
            quote!( Requests::#enum_variant => {
                let stream = incoming
                    .map(|bytes: Vec<u8>| -> Result<#req_type, Status> {
                        #req_type::decode(bytes.into())
                            .map_err(|_| Status::RequestDecodeError)
                    });

                let stream = Box::pin(stream);

                <S as server::#service_trait>::#method_name(&mut self.service, stream);
            },)
        }
        (false, true) => {
            quote!()
        }
        (true, true) => {
            quote!()
        }
    }
}
