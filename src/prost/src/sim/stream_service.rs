#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HangingChannel {
    #[prost(message, optional, tag="1")]
    pub upstream: ::core::option::Option<super::common::ActorInfo>,
    #[prost(message, optional, tag="2")]
    pub downstream: ::core::option::Option<super::common::ActorInfo>,
}
/// Describe the fragments which will be running on this node
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateActorsRequest {
    #[prost(string, tag="1")]
    pub request_id: ::prost::alloc::string::String,
    #[prost(message, repeated, tag="2")]
    pub actors: ::prost::alloc::vec::Vec<super::stream_plan::StreamActor>,
    #[prost(message, repeated, tag="3")]
    pub hanging_channels: ::prost::alloc::vec::Vec<HangingChannel>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateActorsResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BroadcastActorInfoTableRequest {
    #[prost(message, repeated, tag="1")]
    pub info: ::prost::alloc::vec::Vec<super::common::ActorInfo>,
}
/// Create channels and gRPC connections for a fragment
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BuildActorsRequest {
    #[prost(string, tag="1")]
    pub request_id: ::prost::alloc::string::String,
    #[prost(uint32, repeated, tag="2")]
    pub actor_id: ::prost::alloc::vec::Vec<u32>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BuildActorsResponse {
    #[prost(string, tag="1")]
    pub request_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropActorsRequest {
    #[prost(string, tag="1")]
    pub request_id: ::prost::alloc::string::String,
    #[prost(uint32, repeated, tag="2")]
    pub actor_ids: ::prost::alloc::vec::Vec<u32>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropActorsResponse {
    #[prost(string, tag="1")]
    pub request_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ForceStopActorsRequest {
    #[prost(string, tag="1")]
    pub request_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub epoch: ::core::option::Option<super::data::Epoch>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ForceStopActorsResponse {
    #[prost(string, tag="1")]
    pub request_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InjectBarrierRequest {
    #[prost(string, tag="1")]
    pub request_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub barrier: ::core::option::Option<super::data::Barrier>,
    #[prost(uint32, repeated, tag="3")]
    pub actor_ids_to_send: ::prost::alloc::vec::Vec<u32>,
    #[prost(uint32, repeated, tag="4")]
    pub actor_ids_to_collect: ::prost::alloc::vec::Vec<u32>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InjectBarrierResponse {
    #[prost(string, tag="1")]
    pub request_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, repeated, tag="3")]
    pub finished_create_mviews: ::prost::alloc::vec::Vec<inject_barrier_response::FinishedCreateMview>,
}
/// Nested message and enum types in `InjectBarrierResponse`.
pub mod inject_barrier_response {
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct FinishedCreateMview {
        #[prost(uint64, tag="1")]
        pub epoch: u64,
        #[prost(uint32, tag="2")]
        pub actor_id: u32,
    }
}
/// Before starting streaming, the leader node broadcast the actor-host table to needed workers.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BroadcastActorInfoTableResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateSourceRequest {
    #[prost(message, optional, tag="1")]
    pub source: ::core::option::Option<super::catalog::Source>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateSourceResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropSourceRequest {
    #[prost(uint32, tag="1")]
    pub source_id: u32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropSourceResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SyncSourcesRequest {
    #[prost(message, repeated, tag="1")]
    pub sources: ::prost::alloc::vec::Vec<super::catalog::Source>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SyncSourcesResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
}
/// Generated client implementations.
pub mod stream_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct StreamServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl StreamServiceClient<tonic::transport::Channel> {
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
    impl StreamServiceClient<tonic::transport::Channel> {
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
        pub async fn update_actors(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateActorsRequest>,
        ) -> Result<tonic::Response<super::UpdateActorsResponse>, tonic::Status> {
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
                "/stream_service.StreamService/UpdateActors",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn build_actors(
            &mut self,
            request: impl tonic::IntoRequest<super::BuildActorsRequest>,
        ) -> Result<tonic::Response<super::BuildActorsResponse>, tonic::Status> {
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
                "/stream_service.StreamService/BuildActors",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn broadcast_actor_info_table(
            &mut self,
            request: impl tonic::IntoRequest<super::BroadcastActorInfoTableRequest>,
        ) -> Result<
                tonic::Response<super::BroadcastActorInfoTableResponse>,
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
                "/stream_service.StreamService/BroadcastActorInfoTable",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn drop_actors(
            &mut self,
            request: impl tonic::IntoRequest<super::DropActorsRequest>,
        ) -> Result<tonic::Response<super::DropActorsResponse>, tonic::Status> {
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
                "/stream_service.StreamService/DropActors",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn force_stop_actors(
            &mut self,
            request: impl tonic::IntoRequest<super::ForceStopActorsRequest>,
        ) -> Result<tonic::Response<super::ForceStopActorsResponse>, tonic::Status> {
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
                "/stream_service.StreamService/ForceStopActors",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn inject_barrier(
            &mut self,
            request: impl tonic::IntoRequest<super::InjectBarrierRequest>,
        ) -> Result<tonic::Response<super::InjectBarrierResponse>, tonic::Status> {
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
                "/stream_service.StreamService/InjectBarrier",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_source(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateSourceRequest>,
        ) -> Result<tonic::Response<super::CreateSourceResponse>, tonic::Status> {
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
                "/stream_service.StreamService/CreateSource",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn sync_sources(
            &mut self,
            request: impl tonic::IntoRequest<super::SyncSourcesRequest>,
        ) -> Result<tonic::Response<super::SyncSourcesResponse>, tonic::Status> {
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
                "/stream_service.StreamService/SyncSources",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn drop_source(
            &mut self,
            request: impl tonic::IntoRequest<super::DropSourceRequest>,
        ) -> Result<tonic::Response<super::DropSourceResponse>, tonic::Status> {
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
                "/stream_service.StreamService/DropSource",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod stream_service_server {
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
    pub trait StreamService: Send + Sync + 'static {
        async fn update_actors(
            &self,
            request: tonic::Request<super::UpdateActorsRequest>,
        ) -> Result<tonic::Response<super::UpdateActorsResponse>, tonic::Status>;
        async fn build_actors(
            &self,
            request: tonic::Request<super::BuildActorsRequest>,
        ) -> Result<tonic::Response<super::BuildActorsResponse>, tonic::Status>;
        async fn broadcast_actor_info_table(
            &self,
            request: tonic::Request<super::BroadcastActorInfoTableRequest>,
        ) -> Result<
                tonic::Response<super::BroadcastActorInfoTableResponse>,
                tonic::Status,
            >;
        async fn drop_actors(
            &self,
            request: tonic::Request<super::DropActorsRequest>,
        ) -> Result<tonic::Response<super::DropActorsResponse>, tonic::Status>;
        async fn force_stop_actors(
            &self,
            request: tonic::Request<super::ForceStopActorsRequest>,
        ) -> Result<tonic::Response<super::ForceStopActorsResponse>, tonic::Status>;
        async fn inject_barrier(
            &self,
            request: tonic::Request<super::InjectBarrierRequest>,
        ) -> Result<tonic::Response<super::InjectBarrierResponse>, tonic::Status>;
        async fn create_source(
            &self,
            request: tonic::Request<super::CreateSourceRequest>,
        ) -> Result<tonic::Response<super::CreateSourceResponse>, tonic::Status>;
        async fn sync_sources(
            &self,
            request: tonic::Request<super::SyncSourcesRequest>,
        ) -> Result<tonic::Response<super::SyncSourcesResponse>, tonic::Status>;
        async fn drop_source(
            &self,
            request: tonic::Request<super::DropSourceRequest>,
        ) -> Result<tonic::Response<super::DropSourceResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct StreamServiceServer<T: StreamService> {
        inner: Arc<T>,
    }
    impl<T: StreamService> StreamServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self { inner }
        }
    }
    impl<T> tonic::codegen::Service<(PathAndQuery, BoxMessageStream)>
    for StreamServiceServer<T>
    where
        T: StreamService,
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
                "/stream_service.StreamService/UpdateActors" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::UpdateActorsRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .update_actors(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/stream_service.StreamService/BuildActors" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::BuildActorsRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .build_actors(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/stream_service.StreamService/BroadcastActorInfoTable" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<
                            tonic::Request<super::BroadcastActorInfoTableRequest>,
                        >()
                            .unwrap();
                        let res = (*inner)
                            .broadcast_actor_info_table(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/stream_service.StreamService/DropActors" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::DropActorsRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .drop_actors(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/stream_service.StreamService/ForceStopActors" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::ForceStopActorsRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .force_stop_actors(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/stream_service.StreamService/InjectBarrier" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::InjectBarrierRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .inject_barrier(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/stream_service.StreamService/CreateSource" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::CreateSourceRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .create_source(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/stream_service.StreamService/SyncSources" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::SyncSourcesRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .sync_sources(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/stream_service.StreamService/DropSource" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::DropSourceRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .drop_source(request)
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
    impl<T: StreamService> Clone for StreamServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: StreamService> tonic::transport::NamedService for StreamServiceServer<T> {
        const NAME: &'static str = "stream_service.StreamService";
    }
}
