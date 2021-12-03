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

## Design intentions

- Allow resource constrained devices to participate. We do this by defining a simple core api based on polling with optional extensions that improve bandwidth and allow for live updates.
- Prefer off the shelf solutions to lower barrier to adoption. We think gRPC is the best choice for this.
- Let the client stay in control by asking for what it needs.
  - Streaming updates from the server should only broadcast lightweight messages that notify that something has happened, not what has happened. Clients decide when or if they want to request the actual change. This makes it harder to overwhelm the client if a huge number of large messages would otherwise be pushed on the client.
- Prefer apis that work with collections of items rather than one by one. This enables batching / throttling of workloads.


## Replication RPC


