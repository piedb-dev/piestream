//// AuthInfo is the information required to login to a server.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuthInfo {
    #[prost(enumeration="auth_info::EncryptionType", tag="1")]
    pub encryption_type: i32,
    #[prost(bytes="vec", tag="2")]
    pub encrypted_value: ::prost::alloc::vec::Vec<u8>,
}
/// Nested message and enum types in `AuthInfo`.
pub mod auth_info {
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum EncryptionType {
        Unknown = 0,
        Plaintext = 1,
        Sha256 = 2,
        Md5 = 3,
    }
}
//// User defines a user in the system.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserInfo {
    #[prost(uint32, tag="1")]
    pub id: u32,
    #[prost(string, tag="2")]
    pub name: ::prost::alloc::string::String,
    #[prost(bool, tag="3")]
    pub is_supper: bool,
    #[prost(bool, tag="4")]
    pub can_create_db: bool,
    #[prost(bool, tag="5")]
    pub can_login: bool,
    #[prost(message, optional, tag="6")]
    pub auth_info: ::core::option::Option<AuthInfo>,
    //// Granted privileges will be only updated through the command of GRANT/REVOKE.
    #[prost(message, repeated, tag="7")]
    pub privileges: ::prost::alloc::vec::Vec<GrantPrivilege>,
}
//// GrantPrivilege defines a privilege granted to a user.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GrantPrivilege {
    #[prost(enumeration="grant_privilege::Privilege", repeated, tag="5")]
    pub privileges: ::prost::alloc::vec::Vec<i32>,
    #[prost(bool, tag="6")]
    pub with_grant_option: bool,
    #[prost(oneof="grant_privilege::Target", tags="1, 2, 3, 4")]
    pub target: ::core::option::Option<grant_privilege::Target>,
}
/// Nested message and enum types in `GrantPrivilege`.
pub mod grant_privilege {
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GrantDatabase {
        #[prost(uint32, tag="1")]
        pub database_id: u32,
    }
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GrantSchema {
        #[prost(uint32, tag="1")]
        pub database_id: u32,
        #[prost(uint32, tag="2")]
        pub schema_id: u32,
    }
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GrantTable {
        #[prost(uint32, tag="1")]
        pub database_id: u32,
        #[prost(uint32, tag="2")]
        pub schema_id: u32,
        #[prost(uint32, tag="3")]
        pub table_id: u32,
    }
    //// To support grant privilege on ALL TABLES IN SCHEMA schema_name.
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GrantAllTables {
        #[prost(uint32, tag="1")]
        pub database_id: u32,
        #[prost(uint32, tag="2")]
        pub schema_id: u32,
    }
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Privilege {
        Unknown = 0,
        Select = 1,
        Insert = 2,
        Update = 3,
        Delete = 4,
        Create = 5,
        Connect = 6,
        All = 20,
    }
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Target {
        #[prost(message, tag="1")]
        GrantDatabase(GrantDatabase),
        #[prost(message, tag="2")]
        GrantSchema(GrantSchema),
        #[prost(message, tag="3")]
        GrantTable(GrantTable),
        #[prost(message, tag="4")]
        GrantAllTables(GrantAllTables),
    }
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateUserRequest {
    #[prost(message, optional, tag="1")]
    pub user: ::core::option::Option<UserInfo>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateUserResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(uint32, tag="2")]
    pub user_id: u32,
    #[prost(uint64, tag="3")]
    pub version: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropUserRequest {
    #[prost(uint32, tag="1")]
    pub user_id: u32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropUserResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(uint64, tag="2")]
    pub version: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GrantPrivilegeRequest {
    #[prost(uint32, tag="1")]
    pub user_id: u32,
    #[prost(message, optional, tag="2")]
    pub privilege: ::core::option::Option<GrantPrivilege>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GrantPrivilegeResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(uint64, tag="2")]
    pub version: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RevokePrivilegeRequest {
    #[prost(uint32, tag="1")]
    pub user_id: u32,
    #[prost(message, optional, tag="2")]
    pub privilege: ::core::option::Option<GrantPrivilege>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RevokePrivilegeResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(uint64, tag="2")]
    pub version: u64,
}
/// Generated client implementations.
pub mod user_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct UserServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl UserServiceClient<tonic::transport::Channel> {
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
    impl UserServiceClient<tonic::transport::Channel> {
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
        pub async fn create_user(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateUserRequest>,
        ) -> Result<tonic::Response<super::CreateUserResponse>, tonic::Status> {
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
                "/user.UserService/CreateUser",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn drop_user(
            &mut self,
            request: impl tonic::IntoRequest<super::DropUserRequest>,
        ) -> Result<tonic::Response<super::DropUserResponse>, tonic::Status> {
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
                "/user.UserService/DropUser",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn grant_privilege(
            &mut self,
            request: impl tonic::IntoRequest<super::GrantPrivilegeRequest>,
        ) -> Result<tonic::Response<super::GrantPrivilegeResponse>, tonic::Status> {
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
                "/user.UserService/GrantPrivilege",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn revoke_privilege(
            &mut self,
            request: impl tonic::IntoRequest<super::RevokePrivilegeRequest>,
        ) -> Result<tonic::Response<super::RevokePrivilegeResponse>, tonic::Status> {
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
                "/user.UserService/RevokePrivilege",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod user_service_server {
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
    pub trait UserService: Send + Sync + 'static {
        async fn create_user(
            &self,
            request: tonic::Request<super::CreateUserRequest>,
        ) -> Result<tonic::Response<super::CreateUserResponse>, tonic::Status>;
        async fn drop_user(
            &self,
            request: tonic::Request<super::DropUserRequest>,
        ) -> Result<tonic::Response<super::DropUserResponse>, tonic::Status>;
        async fn grant_privilege(
            &self,
            request: tonic::Request<super::GrantPrivilegeRequest>,
        ) -> Result<tonic::Response<super::GrantPrivilegeResponse>, tonic::Status>;
        async fn revoke_privilege(
            &self,
            request: tonic::Request<super::RevokePrivilegeRequest>,
        ) -> Result<tonic::Response<super::RevokePrivilegeResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct UserServiceServer<T: UserService> {
        inner: Arc<T>,
    }
    impl<T: UserService> UserServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self { inner }
        }
    }
    impl<T> tonic::codegen::Service<(PathAndQuery, BoxMessageStream)>
    for UserServiceServer<T>
    where
        T: UserService,
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
                "/user.UserService/CreateUser" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::CreateUserRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .create_user(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/user.UserService/DropUser" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::DropUserRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .drop_user(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/user.UserService/GrantPrivilege" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::GrantPrivilegeRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .grant_privilege(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/user.UserService/RevokePrivilege" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::RevokePrivilegeRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .revoke_privilege(request)
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
    impl<T: UserService> Clone for UserServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: UserService> tonic::transport::NamedService for UserServiceServer<T> {
        const NAME: &'static str = "user.UserService";
    }
}
