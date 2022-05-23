/// Task is a running instance of Stage.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TaskId {
    #[prost(string, tag="1")]
    pub query_id: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub stage_id: u32,
    #[prost(uint32, tag="3")]
    pub task_id: u32,
}
/// Every task will create N buffers (channels) for parent operators to fetch results from,
/// where N is the parallelism of parent stage.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TaskOutputId {
    #[prost(message, optional, tag="1")]
    pub task_id: ::core::option::Option<TaskId>,
    /// The id of output channel to fetch from
    #[prost(uint32, tag="2")]
    pub output_id: u32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TaskInfo {
    #[prost(message, optional, tag="1")]
    pub task_id: ::core::option::Option<super::batch_plan::TaskId>,
    #[prost(enumeration="task_info::TaskStatus", tag="2")]
    pub task_status: i32,
}
/// Nested message and enum types in `TaskInfo`.
pub mod task_info {
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum TaskStatus {
        NotFound = 0,
        Pending = 1,
        Running = 2,
        Failing = 3,
        Cancelling = 4,
        Finished = 5,
        Failed = 6,
    }
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateTaskRequest {
    #[prost(message, optional, tag="1")]
    pub task_id: ::core::option::Option<super::batch_plan::TaskId>,
    #[prost(message, optional, tag="2")]
    pub plan: ::core::option::Option<super::batch_plan::PlanFragment>,
    #[prost(uint64, tag="3")]
    pub epoch: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateTaskResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AbortTaskRequest {
    #[prost(message, optional, tag="1")]
    pub task_id: ::core::option::Option<super::batch_plan::TaskId>,
    #[prost(bool, tag="2")]
    pub force: bool,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AbortTaskResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetTaskInfoRequest {
    #[prost(message, optional, tag="1")]
    pub task_id: ::core::option::Option<super::batch_plan::TaskId>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetTaskInfoResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, optional, tag="2")]
    pub task_info: ::core::option::Option<TaskInfo>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetDataResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, optional, tag="2")]
    pub record_batch: ::core::option::Option<super::data::DataChunk>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetStreamRequest {
    #[prost(uint32, tag="1")]
    pub up_fragment_id: u32,
    #[prost(uint32, tag="2")]
    pub down_fragment_id: u32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetDataRequest {
    #[prost(message, optional, tag="1")]
    pub task_output_id: ::core::option::Option<super::batch_plan::TaskOutputId>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetStreamResponse {
    #[prost(message, optional, tag="1")]
    pub message: ::core::option::Option<super::data::StreamMessage>,
}
/// Generated client implementations.
pub mod task_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct TaskServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl TaskServiceClient<tonic::transport::Channel> {
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
    impl TaskServiceClient<tonic::transport::Channel> {
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
        pub async fn create_task(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateTaskRequest>,
        ) -> Result<tonic::Response<super::CreateTaskResponse>, tonic::Status> {
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
                "/task_service.TaskService/CreateTask",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_task_info(
            &mut self,
            request: impl tonic::IntoRequest<super::GetTaskInfoRequest>,
        ) -> Result<tonic::Response<super::GetTaskInfoResponse>, tonic::Status> {
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
                "/task_service.TaskService/GetTaskInfo",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn abort_task(
            &mut self,
            request: impl tonic::IntoRequest<super::AbortTaskRequest>,
        ) -> Result<tonic::Response<super::AbortTaskResponse>, tonic::Status> {
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
                "/task_service.TaskService/AbortTask",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated client implementations.
pub mod exchange_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct ExchangeServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ExchangeServiceClient<tonic::transport::Channel> {
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
    impl ExchangeServiceClient<tonic::transport::Channel> {
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
        pub async fn get_data(
            &mut self,
            request: impl tonic::IntoRequest<super::GetDataRequest>,
        ) -> Result<
                tonic::Response<tonic::codec::Streaming<super::GetDataResponse>>,
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
                "/task_service.ExchangeService/GetData",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        pub async fn get_stream(
            &mut self,
            request: impl tonic::IntoRequest<super::GetStreamRequest>,
        ) -> Result<
                tonic::Response<tonic::codec::Streaming<super::GetStreamResponse>>,
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
                "/task_service.ExchangeService/GetStream",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod task_service_server {
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
    pub trait TaskService: Send + Sync + 'static {
        async fn create_task(
            &self,
            request: tonic::Request<super::CreateTaskRequest>,
        ) -> Result<tonic::Response<super::CreateTaskResponse>, tonic::Status>;
        async fn get_task_info(
            &self,
            request: tonic::Request<super::GetTaskInfoRequest>,
        ) -> Result<tonic::Response<super::GetTaskInfoResponse>, tonic::Status>;
        async fn abort_task(
            &self,
            request: tonic::Request<super::AbortTaskRequest>,
        ) -> Result<tonic::Response<super::AbortTaskResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct TaskServiceServer<T: TaskService> {
        inner: Arc<T>,
    }
    impl<T: TaskService> TaskServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self { inner }
        }
    }
    impl<T> tonic::codegen::Service<(PathAndQuery, BoxMessageStream)>
    for TaskServiceServer<T>
    where
        T: TaskService,
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
                "/task_service.TaskService/CreateTask" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::CreateTaskRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .create_task(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/task_service.TaskService/GetTaskInfo" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::GetTaskInfoRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .get_task_info(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/task_service.TaskService/AbortTask" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::AbortTaskRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .abort_task(request)
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
    impl<T: TaskService> Clone for TaskServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: TaskService> tonic::transport::NamedService for TaskServiceServer<T> {
        const NAME: &'static str = "task_service.TaskService";
    }
}
/// Generated server implementations.
pub mod exchange_service_server {
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
    pub trait ExchangeService: Send + Sync + 'static {
        type GetDataStream: futures_core::Stream<
                Item = Result<super::GetDataResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn get_data(
            &self,
            request: tonic::Request<super::GetDataRequest>,
        ) -> Result<tonic::Response<Self::GetDataStream>, tonic::Status>;
        type GetStreamStream: futures_core::Stream<
                Item = Result<super::GetStreamResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn get_stream(
            &self,
            request: tonic::Request<super::GetStreamRequest>,
        ) -> Result<tonic::Response<Self::GetStreamStream>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct ExchangeServiceServer<T: ExchangeService> {
        inner: Arc<T>,
    }
    impl<T: ExchangeService> ExchangeServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self { inner }
        }
    }
    impl<T> tonic::codegen::Service<(PathAndQuery, BoxMessageStream)>
    for ExchangeServiceServer<T>
    where
        T: ExchangeService,
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
                "/task_service.ExchangeService/GetData" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::GetDataRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .get_data(request)
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
                "/task_service.ExchangeService/GetStream" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::GetStreamRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .get_stream(request)
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
    impl<T: ExchangeService> Clone for ExchangeServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: ExchangeService> tonic::transport::NamedService
    for ExchangeServiceServer<T> {
        const NAME: &'static str = "task_service.ExchangeService";
    }
}
