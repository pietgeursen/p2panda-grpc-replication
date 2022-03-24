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
        let trait_name = format_ident!("{}Service", service.name);
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

    quote!(

        // If we have a type that implements #trait_name then we can pass it a request and
        // get a reponse from it.
        pub struct P2pandaServer<S: #trait_name>{
            service: S,
        }

        impl<S: #trait_name> P2pandaServer<S>{

            pub fn new(service: S) -> Self {
                Self{
                    service
                }
            }

            pub async fn handle_request(&mut self, request: &[u8]) -> Vec<u8>{
                let request = serde_json::from_slice(request);

                if request.is_err() {
                    return serde_json::to_vec(&Result::<Vec<u8>, Status>::Err(Status::RequestDecodeError)).unwrap();
                }

                match request.unwrap() {
                    #match_arms
                }
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

            fn serialize_message<S, M>(message: &M, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::ser::Serializer,
                M: prost::Message,
            {
                serializer.serialize_bytes(&message.encode_to_vec())
            }

            fn deserialize_message<'de, D, M>(deserialize: D) -> Result<M, D::Error>
            where
                D: serde::de::Deserializer<'de>,
                M: prost::Message + Default,
            {
                struct MessageVisitor<M> {
                    msg: std::marker::PhantomData<M>,
                }

                impl<M> Default for MessageVisitor<M> {
                    fn default() -> Self {
                        Self {
                            msg: Default::default(),
                        }
                    }
                }
                impl<'de, M> serde::de::Visitor<'de> for MessageVisitor<M>
                where
                    M: prost::Message + Default,
                {
                    type Value = M;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("a proto buf message encoded as bytes")
                    }

                    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        M::decode(v).map_err(|err| {
                            serde::de::Error::custom(format!("Protobuf decode failed: {:?}", err))
                        })
                    }
                }

                deserialize.deserialize_any(MessageVisitor::<M>::default())
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
    quote!( async fn #method_name(&mut self, request: #req_type) -> Result<#res_type, Status>; )
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
    let method = format_ident!("{}", &method.input_type);
    quote!(
    #method(
        #[serde(
            serialize_with = "serialize_message",
            deserialize_with = "deserialize_message"
        )]
        #method
        ),
    )
}

fn create_req_match_arm(method: &Method, service_trait: &Ident) -> proc_macro2::TokenStream {
    let input = format_ident!("{}", &method.input_type);
    let method = create_server_method_name(method);
    quote!( Requests::#input(req) => {
        let result = <S as server::#service_trait>::#method(&mut self.service, req).await
            .map(|response| response.encode_to_vec());

        serde_json::to_vec(&result).expect("expected to be able to encode to vec")
    },)
}
