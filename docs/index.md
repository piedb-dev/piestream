# piestream Developer Documentation

Welcome to an overview of the developer documentations of piestream!

## Design Docs

The [design docs](https://github.com/piestreamlabs/piestream/blob/main/docs/README.md) covers some high-level ideas of how we built piestream.

## Crate Docs

Here are the rustdocs of the core crates in piestream. You can read them to understand the implementation details of piestream.

<!-- Not all crates are listed here. For example, binary crates and test crates are not included. -->

### Core Components

- **Frontend**
  - [piestream_sqlparser](piestream_sqlparser/index.html)
  - [piestream_frontend](piestream_frontend/index.html)
- **Meta**
  - [piestream_meta](piestream_meta/index.html)
- **Compute**
  - [piestream_expr](piestream_expr/index.html)
  - [piestream_batch](piestream_batch/index.html)
  - [piestream_stream](piestream_stream/index.html)
- **Storage**
  - [piestream_storage](piestream_storage/index.html)
  - [piestream_hummock_sdk](piestream_hummock_sdk/index.html)
  - [piestream_object_store](piestream_object_store/index.html)

### Source and Connector

- [piestream_source](piestream_source/index.html)
- [piestream_connector](piestream_connector/index.html)

### Common

Common functionalities shared inside piestream.

- [piestream_common](piestream_common/index.html)
- [piestream_common_service](piestream_common_service/index.html)
- [piestream_rpc_client](piestream_rpc_client/index.html)
- [piestream_tracing](piestream_tracing/index.html)
- [piestream_pb](piestream_pb/index.html): protobuf definitions generated by prost from `src/proto`
- [piestream_rt](piestream_rt/index.html)

### Utils

The crates under `src/utils` are several independent util crates which helps to simplify development. We plan to publish them to [crates.io](https://crates.io/) in future when they are more mature.

- [async_stack_trace](async_stack_trace/index.html)
- [global_stats_alloc](global_stats_alloc/index.html)
- [local_stats_alloc](local_stats_alloc/index.html)
- [memcomparable](memcomparable/index.html)
- [pgwire](pgwire/index.html)
