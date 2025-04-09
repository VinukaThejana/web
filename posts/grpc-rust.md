# Building a gRPC Application in Rust and Analyze

gRPC (gRPC Remote Procedure Calls) is a modern, high-performance RPC framework that can run in any environment. It uses HTTP/2 for transport, Protocol Buffers as the interface description language, and provides features such as authentication, load balancing, and more. In this blog post, we’ll walk through building a simple gRPC application in Rust and then capture and analyze the network traffic using `tcpdump`.

## Prerequisites

Before we begin, make sure you have the following installed:

- *Rust* - You can install Rust using [rustup](https://rustup.rs/).
- *tcpdump* - Install`tcpdump` using your package manager.
- [grpccurl](https://github.com/fullstorydev/grpcurl) or [postman](https://www.postman.com/) for making grpc requests.


## Setting Up the Rust Project

First, let’s create a new Rust project:

```bash
cargo new grpc-example
cd grpc-example
```

Next, add the necessary dependencies to your `Cargo.toml` file:

```toml
[package]
name = "grpc-example"
version = "0.1.0"
edition = "2021"

[dependencies]
prost = "0.13.5"
tonic = "0.12.3"
tokio = { version = "1.43.0", features = ["full"] }

[build-dependencies]
tonic-build = "0.12.3"
```

Here, we're using `tonic` for gRPC implementation for Rust, and `prost` for Protocol Buffers.

## Defining the gRPC Service

Create a `proto` directory in your project root and add a `hello.proto` file.

```proto
syntax = "proto3";

package hello;

service Greeter {
  rpc SayHello (HelloRequest) returns (HelloReply);
}

message HelloRequest {
  string name = 1;
}

message HelloReply {
  string message = 3;
}
```

This defines a simple gRPC service with a single `SayHello` method. Mainly, there are 4 different ways to define a method by adding or omitting the stream keyword.

This defines a simple gRPC service with a single `SayHello` method. Mainly, there are 4 different ways to define a method by adding or omitting the `stream` keyword.

- Unary (client sends a request and server sends the response)
- Server Streaming (client sends a request, but server sends a stream of messages back)
- Client Streaming (client sends a stream of messages, but server sends a single response)
- Bidirectional streaming (both client and server sends streams of messages)


In this post we only implement a unary method as our focus it to understand how protobuf serialization works.
