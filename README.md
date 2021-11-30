# P2panda gRPC replication

> Quick spike into grpc in Rust using Tonic

## Try it out

- Get [bloomrpc](https://github.com/bloomrpc/bloomrpc)
- Start the server: `$ cargo run`
- Hit the api using bloomrpc
  - load the `replication.proto` file in this project using the `+` button on the left.
  - select one of the rpc methods
    - lol the default template for bytes is broken in bloomrpc: https://github.com/bloomrpc/bloomrpc/issues/101
  - change the address / port to `localhost:50051`
  - press play

