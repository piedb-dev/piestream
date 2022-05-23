#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateDatabaseRequest {
    #[prost(message, optional, tag="1")]
    pub db: ::core::option::Option<super::catalog::Database>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateDatabaseResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(uint32, tag="2")]
    pub database_id: u32,
    #[prost(uint64, tag="3")]
    pub version: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropDatabaseRequest {
    #[prost(uint32, tag="1")]
    pub database_id: u32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropDatabaseResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(uint64, tag="2")]
    pub version: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateSchemaRequest {
    #[prost(message, optional, tag="1")]
    pub schema: ::core::option::Option<super::catalog::Schema>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateSchemaResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(uint32, tag="2")]
    pub schema_id: u32,
    #[prost(uint64, tag="3")]
    pub version: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropSchemaRequest {
    #[prost(uint32, tag="1")]
    pub schema_id: u32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropSchemaResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(uint64, tag="2")]
    pub version: u64,
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
    #[prost(uint32, tag="2")]
    pub source_id: u32,
    #[prost(uint64, tag="3")]
    pub version: u64,
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
    #[prost(uint64, tag="2")]
    pub version: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateMaterializedViewRequest {
    #[prost(message, optional, tag="1")]
    pub materialized_view: ::core::option::Option<super::catalog::Table>,
    #[prost(message, optional, tag="2")]
    pub fragment_graph: ::core::option::Option<super::stream_plan::StreamFragmentGraph>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateMaterializedViewResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(uint32, tag="2")]
    pub table_id: u32,
    #[prost(uint64, tag="3")]
    pub version: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropMaterializedViewRequest {
    #[prost(uint32, tag="1")]
    pub table_id: u32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropMaterializedViewResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(uint64, tag="2")]
    pub version: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateMaterializedSourceRequest {
    #[prost(message, optional, tag="1")]
    pub source: ::core::option::Option<super::catalog::Source>,
    #[prost(message, optional, tag="2")]
    pub materialized_view: ::core::option::Option<super::catalog::Table>,
    #[prost(message, optional, tag="3")]
    pub fragment_graph: ::core::option::Option<super::stream_plan::StreamFragmentGraph>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateMaterializedSourceResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(uint32, tag="2")]
    pub source_id: u32,
    #[prost(uint32, tag="3")]
    pub table_id: u32,
    #[prost(uint64, tag="4")]
    pub version: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropMaterializedSourceRequest {
    #[prost(uint32, tag="1")]
    pub source_id: u32,
    #[prost(uint32, tag="2")]
    pub table_id: u32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropMaterializedSourceResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(uint64, tag="2")]
    pub version: u64,
}
/// Generated client implementations.
pub mod ddl_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct DdlServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl DdlServiceClient<tonic::transport::Channel> {
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
    impl DdlServiceClient<tonic::transport::Channel> {
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
        pub async fn create_database(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateDatabaseRequest>,
        ) -> Result<tonic::Response<super::CreateDatabaseResponse>, tonic::Status> {
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
                "/ddl_service.DdlService/CreateDatabase",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn drop_database(
            &mut self,
            request: impl tonic::IntoRequest<super::DropDatabaseRequest>,
        ) -> Result<tonic::Response<super::DropDatabaseResponse>, tonic::Status> {
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
                "/ddl_service.DdlService/DropDatabase",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateSchemaRequest>,
        ) -> Result<tonic::Response<super::CreateSchemaResponse>, tonic::Status> {
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
                "/ddl_service.DdlService/CreateSchema",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn drop_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::DropSchemaRequest>,
        ) -> Result<tonic::Response<super::DropSchemaResponse>, tonic::Status> {
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
                "/ddl_service.DdlService/DropSchema",
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
                "/ddl_service.DdlService/CreateSource",
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
                "/ddl_service.DdlService/DropSource",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_materialized_view(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateMaterializedViewRequest>,
        ) -> Result<
                tonic::Response<super::CreateMaterializedViewResponse>,
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
                "/ddl_service.DdlService/CreateMaterializedView",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn drop_materialized_view(
            &mut self,
            request: impl tonic::IntoRequest<super::DropMaterializedViewRequest>,
        ) -> Result<
                tonic::Response<super::DropMaterializedViewResponse>,
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
                "/ddl_service.DdlService/DropMaterializedView",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_materialized_source(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateMaterializedSourceRequest>,
        ) -> Result<
                tonic::Response<super::CreateMaterializedSourceResponse>,
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
                "/ddl_service.DdlService/CreateMaterializedSource",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn drop_materialized_source(
            &mut self,
            request: impl tonic::IntoRequest<super::DropMaterializedSourceRequest>,
        ) -> Result<
                tonic::Response<super::DropMaterializedSourceResponse>,
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
                "/ddl_service.DdlService/DropMaterializedSource",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod ddl_service_server {
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
    pub trait DdlService: Send + Sync + 'static {
        async fn create_database(
            &self,
            request: tonic::Request<super::CreateDatabaseRequest>,
        ) -> Result<tonic::Response<super::CreateDatabaseResponse>, tonic::Status>;
        async fn drop_database(
            &self,
            request: tonic::Request<super::DropDatabaseRequest>,
        ) -> Result<tonic::Response<super::DropDatabaseResponse>, tonic::Status>;
        async fn create_schema(
            &self,
            request: tonic::Request<super::CreateSchemaRequest>,
        ) -> Result<tonic::Response<super::CreateSchemaResponse>, tonic::Status>;
        async fn drop_schema(
            &self,
            request: tonic::Request<super::DropSchemaRequest>,
        ) -> Result<tonic::Response<super::DropSchemaResponse>, tonic::Status>;
        async fn create_source(
            &self,
            request: tonic::Request<super::CreateSourceRequest>,
        ) -> Result<tonic::Response<super::CreateSourceResponse>, tonic::Status>;
        async fn drop_source(
            &self,
            request: tonic::Request<super::DropSourceRequest>,
        ) -> Result<tonic::Response<super::DropSourceResponse>, tonic::Status>;
        async fn create_materialized_view(
            &self,
            request: tonic::Request<super::CreateMaterializedViewRequest>,
        ) -> Result<
                tonic::Response<super::CreateMaterializedViewResponse>,
                tonic::Status,
            >;
        async fn drop_materialized_view(
            &self,
            request: tonic::Request<super::DropMaterializedViewRequest>,
        ) -> Result<tonic::Response<super::DropMaterializedViewResponse>, tonic::Status>;
        async fn create_materialized_source(
            &self,
            request: tonic::Request<super::CreateMaterializedSourceRequest>,
        ) -> Result<
                tonic::Response<super::CreateMaterializedSourceResponse>,
                tonic::Status,
            >;
        async fn drop_materialized_source(
            &self,
            request: tonic::Request<super::DropMaterializedSourceRequest>,
        ) -> Result<
                tonic::Response<super::DropMaterializedSourceResponse>,
                tonic::Status,
            >;
    }
    #[derive(Debug)]
    pub struct DdlServiceServer<T: DdlService> {
        inner: Arc<T>,
    }
    impl<T: DdlService> DdlServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self { inner }
        }
    }
    impl<T> tonic::codegen::Service<(PathAndQuery, BoxMessageStream)>
    for DdlServiceServer<T>
    where
        T: DdlService,
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
                "/ddl_service.DdlService/CreateDatabase" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::CreateDatabaseRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .create_database(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/ddl_service.DdlService/DropDatabase" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::DropDatabaseRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .drop_database(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/ddl_service.DdlService/CreateSchema" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::CreateSchemaRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .create_schema(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/ddl_service.DdlService/DropSchema" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::DropSchemaRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .drop_schema(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/ddl_service.DdlService/CreateSource" => {
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
                "/ddl_service.DdlService/DropSource" => {
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
                "/ddl_service.DdlService/CreateMaterializedView" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<
                            tonic::Request<super::CreateMaterializedViewRequest>,
                        >()
                            .unwrap();
                        let res = (*inner)
                            .create_materialized_view(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/ddl_service.DdlService/DropMaterializedView" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<
                            tonic::Request<super::DropMaterializedViewRequest>,
                        >()
                            .unwrap();
                        let res = (*inner)
                            .drop_materialized_view(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/ddl_service.DdlService/CreateMaterializedSource" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<
                            tonic::Request<super::CreateMaterializedSourceRequest>,
                        >()
                            .unwrap();
                        let res = (*inner)
                            .create_materialized_source(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/ddl_service.DdlService/DropMaterializedSource" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<
                            tonic::Request<super::DropMaterializedSourceRequest>,
                        >()
                            .unwrap();
                        let res = (*inner)
                            .drop_materialized_source(request)
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
    impl<T: DdlService> Clone for DdlServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: DdlService> tonic::transport::NamedService for DdlServiceServer<T> {
        const NAME: &'static str = "ddl_service.DdlService";
    }
}
