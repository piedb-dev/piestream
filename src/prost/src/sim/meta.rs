#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeartbeatRequest {
    #[prost(uint32, tag="1")]
    pub node_id: u32,
    #[prost(enumeration="super::common::WorkerType", tag="2")]
    pub worker_type: i32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeartbeatResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
}
/// Fragments of a Materialized View
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TableFragments {
    #[prost(uint32, tag="1")]
    pub table_id: u32,
    #[prost(map="uint32, message", tag="2")]
    pub fragments: ::std::collections::HashMap<u32, table_fragments::Fragment>,
    #[prost(map="uint32, message", tag="3")]
    pub actor_status: ::std::collections::HashMap<u32, table_fragments::ActorStatus>,
}
/// Nested message and enum types in `TableFragments`.
pub mod table_fragments {
    /// Runtime information of an actor
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ActorStatus {
        /// Current on which parallel unit
        #[prost(message, optional, tag="1")]
        pub parallel_unit: ::core::option::Option<super::super::common::ParallelUnit>,
        /// Current state
        #[prost(enumeration="ActorState", tag="2")]
        pub state: i32,
    }
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Fragment {
        #[prost(uint32, tag="1")]
        pub fragment_id: u32,
        #[prost(enumeration="super::super::stream_plan::FragmentType", tag="2")]
        pub fragment_type: i32,
        #[prost(enumeration="fragment::FragmentDistributionType", tag="3")]
        pub distribution_type: i32,
        #[prost(message, repeated, tag="4")]
        pub actors: ::prost::alloc::vec::Vec<super::super::stream_plan::StreamActor>,
        /// Vnode mapping (which should be set in upstream dispatcher) of the fragment.
        #[prost(message, optional, tag="5")]
        pub vnode_mapping: ::core::option::Option<super::super::common::ParallelUnitMapping>,
    }
    /// Nested message and enum types in `Fragment`.
    pub mod fragment {
        #[derive(prost_helpers::AnyPB)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
        #[repr(i32)]
        pub enum FragmentDistributionType {
            Single = 0,
            Hash = 1,
        }
    }
    /// Current state of actor
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum ActorState {
        /// Initial state after creation
        Inactive = 0,
        /// Running normally
        Running = 1,
    }
}
/// TODO: remove this when dashboard refactored.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActorLocation {
    #[prost(message, optional, tag="1")]
    pub node: ::core::option::Option<super::common::WorkerNode>,
    #[prost(message, repeated, tag="2")]
    pub actors: ::prost::alloc::vec::Vec<super::stream_plan::StreamActor>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlushRequest {
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlushResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
}
// Below for cluster service.

#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddWorkerNodeRequest {
    #[prost(enumeration="super::common::WorkerType", tag="1")]
    pub worker_type: i32,
    #[prost(message, optional, tag="2")]
    pub host: ::core::option::Option<super::common::HostAddress>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddWorkerNodeResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, optional, tag="2")]
    pub node: ::core::option::Option<super::common::WorkerNode>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActivateWorkerNodeRequest {
    #[prost(message, optional, tag="1")]
    pub host: ::core::option::Option<super::common::HostAddress>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActivateWorkerNodeResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteWorkerNodeRequest {
    #[prost(message, optional, tag="1")]
    pub host: ::core::option::Option<super::common::HostAddress>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteWorkerNodeResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListAllNodesRequest {
    #[prost(enumeration="super::common::WorkerType", tag="1")]
    pub worker_type: i32,
    /// Whether to include nodes still starting
    #[prost(bool, tag="2")]
    pub include_starting_nodes: bool,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListAllNodesResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, repeated, tag="2")]
    pub nodes: ::prost::alloc::vec::Vec<super::common::WorkerNode>,
}
/// Below for notification service.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeRequest {
    #[prost(enumeration="super::common::WorkerType", tag="1")]
    pub worker_type: i32,
    #[prost(message, optional, tag="2")]
    pub host: ::core::option::Option<super::common::HostAddress>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MetaSnapshot {
    #[prost(message, repeated, tag="1")]
    pub nodes: ::prost::alloc::vec::Vec<super::common::WorkerNode>,
    #[prost(message, repeated, tag="2")]
    pub database: ::prost::alloc::vec::Vec<super::catalog::Database>,
    #[prost(message, repeated, tag="3")]
    pub schema: ::prost::alloc::vec::Vec<super::catalog::Schema>,
    #[prost(message, repeated, tag="4")]
    pub source: ::prost::alloc::vec::Vec<super::catalog::Source>,
    #[prost(message, repeated, tag="5")]
    pub table: ::prost::alloc::vec::Vec<super::catalog::Table>,
    #[prost(message, repeated, tag="6")]
    pub view: ::prost::alloc::vec::Vec<super::catalog::VirtualTable>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(enumeration="subscribe_response::Operation", tag="2")]
    pub operation: i32,
    #[prost(uint64, tag="3")]
    pub version: u64,
    #[prost(oneof="subscribe_response::Info", tags="4, 5, 6, 7, 8, 9, 10")]
    pub info: ::core::option::Option<subscribe_response::Info>,
}
/// Nested message and enum types in `SubscribeResponse`.
pub mod subscribe_response {
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Operation {
        Invalid = 0,
        Add = 1,
        Delete = 2,
        Update = 3,
        Snapshot = 4,
    }
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Info {
        #[prost(message, tag="4")]
        Node(super::super::common::WorkerNode),
        #[prost(message, tag="5")]
        Database(super::super::catalog::Database),
        #[prost(message, tag="6")]
        Schema(super::super::catalog::Schema),
        #[prost(message, tag="7")]
        Table(super::super::catalog::Table),
        #[prost(message, tag="8")]
        Source(super::super::catalog::Source),
        #[prost(message, tag="9")]
        Snapshot(super::MetaSnapshot),
        #[prost(message, tag="10")]
        HummockSnapshot(super::super::hummock::HummockSnapshot),
    }
}
/// Generated client implementations.
pub mod heartbeat_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct HeartbeatServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl HeartbeatServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl HeartbeatServiceClient<tonic::transport::Channel> {
        pub fn new(inner: tonic::transport::Channel) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(self) -> Self {
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(self) -> Self {
            self
        }
        pub async fn heartbeat(
            &mut self,
            request: impl tonic::IntoRequest<super::HeartbeatRequest>,
        ) -> Result<tonic::Response<super::HeartbeatResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e),
                    )
                })?;
            let codec = ();
            let path = http::uri::PathAndQuery::from_static(
                "/meta.HeartbeatService/Heartbeat",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated client implementations.
pub mod stream_manager_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct StreamManagerServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl StreamManagerServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl StreamManagerServiceClient<tonic::transport::Channel> {
        pub fn new(inner: tonic::transport::Channel) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(self) -> Self {
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(self) -> Self {
            self
        }
        pub async fn flush(
            &mut self,
            request: impl tonic::IntoRequest<super::FlushRequest>,
        ) -> Result<tonic::Response<super::FlushResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e),
                    )
                })?;
            let codec = ();
            let path = http::uri::PathAndQuery::from_static(
                "/meta.StreamManagerService/Flush",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated client implementations.
pub mod cluster_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct ClusterServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ClusterServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl ClusterServiceClient<tonic::transport::Channel> {
        pub fn new(inner: tonic::transport::Channel) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(self) -> Self {
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(self) -> Self {
            self
        }
        pub async fn add_worker_node(
            &mut self,
            request: impl tonic::IntoRequest<super::AddWorkerNodeRequest>,
        ) -> Result<tonic::Response<super::AddWorkerNodeResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e),
                    )
                })?;
            let codec = ();
            let path = http::uri::PathAndQuery::from_static(
                "/meta.ClusterService/AddWorkerNode",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn activate_worker_node(
            &mut self,
            request: impl tonic::IntoRequest<super::ActivateWorkerNodeRequest>,
        ) -> Result<tonic::Response<super::ActivateWorkerNodeResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e),
                    )
                })?;
            let codec = ();
            let path = http::uri::PathAndQuery::from_static(
                "/meta.ClusterService/ActivateWorkerNode",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_worker_node(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteWorkerNodeRequest>,
        ) -> Result<tonic::Response<super::DeleteWorkerNodeResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e),
                    )
                })?;
            let codec = ();
            let path = http::uri::PathAndQuery::from_static(
                "/meta.ClusterService/DeleteWorkerNode",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_all_nodes(
            &mut self,
            request: impl tonic::IntoRequest<super::ListAllNodesRequest>,
        ) -> Result<tonic::Response<super::ListAllNodesResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e),
                    )
                })?;
            let codec = ();
            let path = http::uri::PathAndQuery::from_static(
                "/meta.ClusterService/ListAllNodes",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated client implementations.
pub mod notification_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct NotificationServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl NotificationServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl NotificationServiceClient<tonic::transport::Channel> {
        pub fn new(inner: tonic::transport::Channel) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(self) -> Self {
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(self) -> Self {
            self
        }
        pub async fn subscribe(
            &mut self,
            request: impl tonic::IntoRequest<super::SubscribeRequest>,
        ) -> Result<
                tonic::Response<tonic::codec::Streaming<super::SubscribeResponse>>,
                tonic::Status,
            > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e),
                    )
                })?;
            let codec = ();
            let path = http::uri::PathAndQuery::from_static(
                "/meta.NotificationService/Subscribe",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod heartbeat_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        unused_mut,
        clippy::let_unit_value,
    )]
    use tonic::codegen::{
        http::uri::PathAndQuery, futures::stream::{self, StreamExt},
        *,
    };
    #[async_trait]
    pub trait HeartbeatService: Send + Sync + 'static {
        async fn heartbeat(
            &self,
            request: tonic::Request<super::HeartbeatRequest>,
        ) -> Result<tonic::Response<super::HeartbeatResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct HeartbeatServiceServer<T: HeartbeatService> {
        inner: Arc<T>,
    }
    impl<T: HeartbeatService> HeartbeatServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self { inner }
        }
    }
    impl<T> tonic::codegen::Service<(PathAndQuery, BoxMessageStream)>
    for HeartbeatServiceServer<T>
    where
        T: HeartbeatService,
    {
        type Response = BoxMessageStream;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(
            &mut self,
            (path, mut req): (PathAndQuery, BoxMessageStream),
        ) -> Self::Future {
            let inner = self.inner.clone();
            match path.path() {
                "/meta.HeartbeatService/Heartbeat" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::HeartbeatRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .heartbeat(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                _ => Box::pin(async move { Ok(stream::empty().boxed()) }),
            }
        }
    }
    impl<T: HeartbeatService> Clone for HeartbeatServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: HeartbeatService> tonic::transport::NamedService
    for HeartbeatServiceServer<T> {
        const NAME: &'static str = "meta.HeartbeatService";
    }
}
/// Generated server implementations.
pub mod stream_manager_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        unused_mut,
        clippy::let_unit_value,
    )]
    use tonic::codegen::{
        http::uri::PathAndQuery, futures::stream::{self, StreamExt},
        *,
    };
    #[async_trait]
    pub trait StreamManagerService: Send + Sync + 'static {
        async fn flush(
            &self,
            request: tonic::Request<super::FlushRequest>,
        ) -> Result<tonic::Response<super::FlushResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct StreamManagerServiceServer<T: StreamManagerService> {
        inner: Arc<T>,
    }
    impl<T: StreamManagerService> StreamManagerServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self { inner }
        }
    }
    impl<T> tonic::codegen::Service<(PathAndQuery, BoxMessageStream)>
    for StreamManagerServiceServer<T>
    where
        T: StreamManagerService,
    {
        type Response = BoxMessageStream;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(
            &mut self,
            (path, mut req): (PathAndQuery, BoxMessageStream),
        ) -> Self::Future {
            let inner = self.inner.clone();
            match path.path() {
                "/meta.StreamManagerService/Flush" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::FlushRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .flush(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                _ => Box::pin(async move { Ok(stream::empty().boxed()) }),
            }
        }
    }
    impl<T: StreamManagerService> Clone for StreamManagerServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: StreamManagerService> tonic::transport::NamedService
    for StreamManagerServiceServer<T> {
        const NAME: &'static str = "meta.StreamManagerService";
    }
}
/// Generated server implementations.
pub mod cluster_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        unused_mut,
        clippy::let_unit_value,
    )]
    use tonic::codegen::{
        http::uri::PathAndQuery, futures::stream::{self, StreamExt},
        *,
    };
    #[async_trait]
    pub trait ClusterService: Send + Sync + 'static {
        async fn add_worker_node(
            &self,
            request: tonic::Request<super::AddWorkerNodeRequest>,
        ) -> Result<tonic::Response<super::AddWorkerNodeResponse>, tonic::Status>;
        async fn activate_worker_node(
            &self,
            request: tonic::Request<super::ActivateWorkerNodeRequest>,
        ) -> Result<tonic::Response<super::ActivateWorkerNodeResponse>, tonic::Status>;
        async fn delete_worker_node(
            &self,
            request: tonic::Request<super::DeleteWorkerNodeRequest>,
        ) -> Result<tonic::Response<super::DeleteWorkerNodeResponse>, tonic::Status>;
        async fn list_all_nodes(
            &self,
            request: tonic::Request<super::ListAllNodesRequest>,
        ) -> Result<tonic::Response<super::ListAllNodesResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct ClusterServiceServer<T: ClusterService> {
        inner: Arc<T>,
    }
    impl<T: ClusterService> ClusterServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self { inner }
        }
    }
    impl<T> tonic::codegen::Service<(PathAndQuery, BoxMessageStream)>
    for ClusterServiceServer<T>
    where
        T: ClusterService,
    {
        type Response = BoxMessageStream;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(
            &mut self,
            (path, mut req): (PathAndQuery, BoxMessageStream),
        ) -> Self::Future {
            let inner = self.inner.clone();
            match path.path() {
                "/meta.ClusterService/AddWorkerNode" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::AddWorkerNodeRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .add_worker_node(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/meta.ClusterService/ActivateWorkerNode" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<
                            tonic::Request<super::ActivateWorkerNodeRequest>,
                        >()
                            .unwrap();
                        let res = (*inner)
                            .activate_worker_node(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/meta.ClusterService/DeleteWorkerNode" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::DeleteWorkerNodeRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .delete_worker_node(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/meta.ClusterService/ListAllNodes" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::ListAllNodesRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .list_all_nodes(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                _ => Box::pin(async move { Ok(stream::empty().boxed()) }),
            }
        }
    }
    impl<T: ClusterService> Clone for ClusterServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: ClusterService> tonic::transport::NamedService for ClusterServiceServer<T> {
        const NAME: &'static str = "meta.ClusterService";
    }
}
/// Generated server implementations.
pub mod notification_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        unused_mut,
        clippy::let_unit_value,
    )]
    use tonic::codegen::{
        http::uri::PathAndQuery, futures::stream::{self, StreamExt},
        *,
    };
    #[async_trait]
    pub trait NotificationService: Send + Sync + 'static {
        type SubscribeStream: futures_core::Stream<
                Item = Result<super::SubscribeResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn subscribe(
            &self,
            request: tonic::Request<super::SubscribeRequest>,
        ) -> Result<tonic::Response<Self::SubscribeStream>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct NotificationServiceServer<T: NotificationService> {
        inner: Arc<T>,
    }
    impl<T: NotificationService> NotificationServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self { inner }
        }
    }
    impl<T> tonic::codegen::Service<(PathAndQuery, BoxMessageStream)>
    for NotificationServiceServer<T>
    where
        T: NotificationService,
    {
        type Response = BoxMessageStream;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(
            &mut self,
            (path, mut req): (PathAndQuery, BoxMessageStream),
        ) -> Self::Future {
            let inner = self.inner.clone();
            match path.path() {
                "/meta.NotificationService/Subscribe" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::SubscribeRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .subscribe(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            res
                                .into_inner()
                                .map(|res| res.map(|rsp| Box::new(rsp) as BoxMessage))
                                .boxed(),
                        )
                    })
                }
                _ => Box::pin(async move { Ok(stream::empty().boxed()) }),
            }
        }
    }
    impl<T: NotificationService> Clone for NotificationServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: NotificationService> tonic::transport::NamedService
    for NotificationServiceServer<T> {
        const NAME: &'static str = "meta.NotificationService";
    }
}
