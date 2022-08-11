// // // Copyright 2022 PieDb Data
// // //
// // // Licensed under the Apache License, Version 2.0 (the "License");
// // // you may not use this file except in compliance with the License.
// // // You may obtain a copy of the License at
// // //
// // // http://www.apache.org/licenses/LICENSE-2.0
// // //
// // // Unless required by applicable law or agreed to in writing, software
// // // distributed under the License is distributed on an "AS IS" BASIS,
// // // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// // // See the License for the specific language governing permissions and
// // // limitations under the License.

// // use crate::handler::handle;
// // use piestream_sqlparser::parser::Parser as SqlParser;
// // use pgwire::pg_server::{BoxedError, Session, SessionManager, UserAuthenticator};
// // use pgwire::pg_response::PgResponse;
// // use pgwire::pg_field_descriptor::PgFieldDescriptor;

// // use piestream_common::session_config::ConfigMap;
// // use parking_lot::{RwLock, RwLockReadGuard};
// // use crate::FrontendOpts;
// // use crate::SessionManagerImpl;
// // use crate::session::{
// //     FrontendEnv,
// //     AuthContext,
// //     SessionImpl
// // };
// // use std::sync::Arc;
// // use clap::Parser;


// // pub async fn run_statement(sql: &str) -> std::result::Result<PgResponse, BoxedError> {
// //     println!("mysql_session.rs run_statement  {:?} ",sql);
// //     let mut stmts = SqlParser::parse_sql(sql).map_err(|e| {
// //         tracing::error!("failed to parse sql:\n{}:\n{}", sql, e);
// //         e
// //     }).unwrap();
// //     println!("stmt ====== {:?}",&stmts);

// //     let stm = stmts.swap_remove(0);
// //     let env = FrontendEnv::mock();
// //     let auth_context = Arc::new(AuthContext::new(
// //         "dev".to_string(),
// //         "root".to_string(),
// //     ));
// //     let user_authenticator = UserAuthenticator::None;
// //     let session = SessionImpl::new(env, auth_context, user_authenticator);
// //     let rsp = handle(Arc::new(session),stm, sql).await.unwrap();
// //     println!("pgresponse ===== {:?}",rsp);
// //     Ok(rsp)

// // }




// // use pgwire::pg_response::PgResponse;
// // use pgwire::pg_server::{BoxedError, Session, SessionManager, UserAuthenticator};
// // use mysql_srv

// use std::fmt::Formatter;
// use std::io::{Error, ErrorKind};
// use std::marker::Sync;
// use std::path::PathBuf;
// use std::sync::atomic::{AtomicI32, Ordering};
// use std::sync::Arc;
// use std::time::Duration;
// use parking_lot::{RwLock, RwLockReadGuard};
// use pgwire::pg_field_descriptor::PgFieldDescriptor;
// use pgwire::pg_response::PgResponse;
// use rand::RngCore;

// #[cfg(test)]
// use piestream_common::catalog::{DEFAULT_DATABASE_NAME, DEFAULT_SUPPER_USER};
// use piestream_common::config::FrontendConfig;
// use piestream_common::error::Result;
// use piestream_common::session_config::ConfigMap;
// use piestream_common::util::addr::HostAddr;
// use piestream_pb::common::WorkerType;
// use piestream_pb::user::auth_info::EncryptionType;
// use piestream_rpc_client::{ComputeClientPool, MetaClient};
// use piestream_sqlparser::ast::Statement;
// use piestream_sqlparser::parser::Parser;
// use tokio::sync::oneshot::Sender;
// use tokio::sync::watch;
// use tokio::task::JoinHandle;

// use crate::binder::Binder;
// use crate::catalog::catalog_service::{CatalogReader, CatalogWriter, CatalogWriterImpl};
// use crate::catalog::root_catalog::Catalog;
// use crate::handler::handle;
// use crate::handler::util::to_pg_field;
// use crate::meta_client::{FrontendMetaClient, FrontendMetaClientImpl};
// use crate::observer::observer_manager::ObserverManager;
// use crate::optimizer::plan_node::PlanNodeId;
// use crate::planner::Planner;
// use crate::scheduler::worker_node_manager::{WorkerNodeManager, WorkerNodeManagerRef};
// use crate::scheduler::{HummockSnapshotManager, HummockSnapshotManagerRef, QueryManager};
// use crate::test_utils::MockUserInfoWriter;
// use crate::user::user_authentication::md5_hash_with_salt;
// use crate::user::user_manager::UserInfoManager;
// use crate::user::user_service::{UserInfoReader, UserInfoWriter, UserInfoWriterImpl};
// use crate::FrontendOpts;


// use piestream_mysql_srv::mysql_server::{BoxedError, MysqlSession, MysqlSessionManager, UserAuthenticator};


// pub struct OptimizerContext {
//     pub session_ctx: Arc<MysqlSessionImpl>,
//     // We use `AtomicI32` here because  `Arc<T>` implements `Send` only when `T: Send + Sync`.
//     pub next_id: AtomicI32,
//     /// For debugging purposes, store the SQL string in Context
//     pub sql: Arc<str>,
// }

// #[derive(Clone, Debug)]
// pub struct OptimizerContextRef {
//     inner: Arc<OptimizerContext>,
// }

// impl !Sync for OptimizerContextRef {}

// impl From<OptimizerContext> for OptimizerContextRef {
//     fn from(inner: OptimizerContext) -> Self {
//         Self {
//             inner: Arc::new(inner),
//         }
//     }
// }

// impl OptimizerContextRef {
//     pub fn inner(&self) -> &OptimizerContext {
//         &self.inner
//     }

//     pub fn next_plan_node_id(&self) -> PlanNodeId {
//         // It's safe to use `fetch_add` and `Relaxed` ordering since we have marked
//         // `QueryContextRef` not `Sync`.
//         let next_id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
//         PlanNodeId(next_id)
//     }
// }

// impl OptimizerContext {
//     pub fn new(session_ctx: Arc<MysqlSessionImpl>, sql: Arc<str>) -> Self {
//         Self {
//             session_ctx,
//             next_id: AtomicI32::new(0),
//             sql,
//         }
//     }

//     // TODO(TaoWu): Remove the async.
//     #[cfg(test)]
//     pub async fn mock() -> OptimizerContextRef {
//         Self {
//             session_ctx: Arc::new(MysqlSessionImpl::mock()),
//             next_id: AtomicI32::new(0),
//             sql: Arc::from(""),
//         }
//         .into()
//     }
// }

// impl std::fmt::Debug for OptimizerContext {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "QueryContext {{ current id = {} }}",
//             self.next_id.load(Ordering::Relaxed)
//         )
//     }
// }

// fn load_config(opts: &FrontendOpts) -> FrontendConfig {
//     if opts.config_path.is_empty() {
//         return FrontendConfig::default();
//     }

//     let config_path = PathBuf::from(opts.config_path.to_owned());
//     FrontendConfig::init(config_path).unwrap()
// }

// /// The global environment for the frontend server.
// #[derive(Clone)]
// pub struct FrontendEnv {
//     // Different session may access catalog at the same time and catalog is protected by a
//     // RwLock.
//     meta_client: Arc<dyn FrontendMetaClient>,
//     catalog_writer: Arc<dyn CatalogWriter>,
//     catalog_reader: CatalogReader,
//     user_info_writer: Arc<dyn UserInfoWriter>,
//     user_info_reader: UserInfoReader,
//     worker_node_manager: WorkerNodeManagerRef,
//     query_manager: QueryManager,
//     hummock_snapshot_manager: HummockSnapshotManagerRef,
//     server_addr: HostAddr,
// }

// impl FrontendEnv {
//     pub async fn init(
//         opts: &FrontendOpts,
//     ) -> Result<(Self, JoinHandle<()>, JoinHandle<()>, Sender<()>)> {
//         let meta_client = MetaClient::new(opts.meta_addr.clone().as_str()).await?;
//         Self::with_meta_client(meta_client, opts).await
//     }

//     pub fn mock() -> Self {
//         use crate::test_utils::{MockCatalogWriter, MockFrontendMetaClient};

//         let catalog = Arc::new(RwLock::new(Catalog::default()));
//         let catalog_writer = Arc::new(MockCatalogWriter::new(catalog.clone()));
//         let catalog_reader = CatalogReader::new(catalog);
//         let user_info_manager = Arc::new(RwLock::new(UserInfoManager::default()));
//         let user_info_writer = Arc::new(MockUserInfoWriter::new(user_info_manager.clone()));
//         let user_info_reader = UserInfoReader::new(user_info_manager);
//         let worker_node_manager = Arc::new(WorkerNodeManager::mock(vec![]));
//         let meta_client = Arc::new(MockFrontendMetaClient {});
//         let hummock_snapshot_manager = Arc::new(HummockSnapshotManager::new(meta_client.clone()));
//         let compute_client_pool = Arc::new(ComputeClientPool::new(u64::MAX));
//         let query_manager = QueryManager::new(
//             worker_node_manager.clone(),
//             hummock_snapshot_manager.clone(),
//             compute_client_pool,
//         );
//         let server_addr = HostAddr::try_from("127.0.0.1:4565").unwrap();
//         Self {
//             meta_client,
//             catalog_writer,
//             catalog_reader,
//             user_info_writer,
//             user_info_reader,
//             worker_node_manager,
//             query_manager,
//             hummock_snapshot_manager,
//             server_addr,
//         }
//     }

//     pub async fn with_meta_client(
//         mut meta_client: MetaClient,
//         opts: &FrontendOpts,
//     ) -> Result<(Self, JoinHandle<()>, JoinHandle<()>, Sender<()>)> {
//         let config = load_config(opts);
//         tracing::info!("Starting frontend node with config {:?}", config);

//         let frontend_address: HostAddr = opts
//             .client_address
//             .as_ref()
//             .unwrap_or(&opts.host)
//             .parse()
//             .unwrap();
//         // Register in meta by calling `AddWorkerNode` RPC.
//         meta_client
//             .register(&frontend_address, WorkerType::Frontend)
//             .await?;

//         let (heartbeat_join_handle, heartbeat_shutdown_sender) = MetaClient::start_heartbeat_loop(
//             meta_client.clone(),
//             Duration::from_millis(config.server.heartbeat_interval_ms as u64),
//         );

//         let (catalog_updated_tx, catalog_updated_rx) = watch::channel(0);
//         let catalog = Arc::new(RwLock::new(Catalog::default()));
//         let catalog_writer = Arc::new(CatalogWriterImpl::new(
//             meta_client.clone(),
//             catalog_updated_rx,
//         ));
//         let catalog_reader = CatalogReader::new(catalog.clone());

//         let worker_node_manager = Arc::new(WorkerNodeManager::new());

//         let frontend_meta_client = Arc::new(FrontendMetaClientImpl(meta_client.clone()));
//         let hummock_snapshot_manager =
//             Arc::new(HummockSnapshotManager::new(frontend_meta_client.clone()));
//         let compute_client_pool = Arc::new(ComputeClientPool::new(u64::MAX));
//         let query_manager = QueryManager::new(
//             worker_node_manager.clone(),
//             hummock_snapshot_manager.clone(),
//             compute_client_pool,
//         );

//         let user_info_manager = Arc::new(RwLock::new(UserInfoManager::default()));
//         let (user_info_updated_tx, user_info_updated_rx) = watch::channel(0);
//         let user_info_reader = UserInfoReader::new(user_info_manager.clone());
//         let user_info_writer = Arc::new(UserInfoWriterImpl::new(
//             meta_client.clone(),
//             user_info_updated_rx,
//         ));

//         let observer_manager = ObserverManager::new(
//             meta_client.clone(),
//             frontend_address.clone(),
//             worker_node_manager.clone(),
//             catalog,
//             catalog_updated_tx,
//             user_info_manager,
//             user_info_updated_tx,
//             hummock_snapshot_manager.clone(),
//         )
//         .await;
//         let observer_join_handle = observer_manager.start().await?;

//         meta_client.activate(&frontend_address).await?;

//         Ok((
//             Self {
//                 catalog_reader,
//                 catalog_writer,
//                 user_info_reader,
//                 user_info_writer,
//                 worker_node_manager,
//                 meta_client: frontend_meta_client,
//                 query_manager,
//                 hummock_snapshot_manager,
//                 server_addr: frontend_address,
//             },
//             observer_join_handle,
//             heartbeat_join_handle,
//             heartbeat_shutdown_sender,
//         ))
//     }

//     /// Get a reference to the frontend env's catalog writer.
//     pub fn catalog_writer(&self) -> &dyn CatalogWriter {
//         &*self.catalog_writer
//     }

//     /// Get a reference to the frontend env's catalog reader.
//     pub fn catalog_reader(&self) -> &CatalogReader {
//         &self.catalog_reader
//     }

//     /// Get a reference to the frontend env's user info writer.
//     pub fn user_info_writer(&self) -> &dyn UserInfoWriter {
//         &*self.user_info_writer
//     }

//     /// Get a reference to the frontend env's user info reader.
//     pub fn user_info_reader(&self) -> &UserInfoReader {
//         &self.user_info_reader
//     }

//     pub fn worker_node_manager(&self) -> &WorkerNodeManager {
//         &*self.worker_node_manager
//     }

//     pub fn worker_node_manager_ref(&self) -> WorkerNodeManagerRef {
//         self.worker_node_manager.clone()
//     }

//     pub fn meta_client(&self) -> &dyn FrontendMetaClient {
//         &*self.meta_client
//     }

//     pub fn meta_client_ref(&self) -> Arc<dyn FrontendMetaClient> {
//         self.meta_client.clone()
//     }

//     pub fn query_manager(&self) -> &QueryManager {
//         &self.query_manager
//     }

//     pub fn hummock_snapshot_manager(&self) -> &HummockSnapshotManagerRef {
//         &self.hummock_snapshot_manager
//     }

//     pub fn server_address(&self) -> &HostAddr {
//         &self.server_addr
//     }
// }
// #[derive(Clone, Debug)]
// pub struct AuthContext {
//     pub database: String,
//     pub user_name: String,
// }

// impl AuthContext {
//     pub fn new(database: String, user_name: String) -> Self {
//         Self {
//             database,
//             user_name,
//         }
//     }
// }








// pub struct MysqlSessionImpl {
//     env: FrontendEnv,
//     auth_context: Arc<AuthContext>,
//     // Used for user authentication.
//     user_authenticator: UserAuthenticator,
//     /// Stores the value of configurations.
//     config_map: RwLock<ConfigMap>,
// }

// impl MysqlSessionImpl {
//     pub fn new(
//         env: FrontendEnv,
//         auth_context: Arc<AuthContext>,
//         user_authenticator: UserAuthenticator,
//     ) -> Self {
//         Self {
//             env,
//             auth_context,
//             user_authenticator,
//             config_map: RwLock::new(Default::default()),
//         }
//     }

//     #[cfg(test)]
//     pub fn mock() -> Self {
//         Self {
//             env: FrontendEnv::mock(),
//             auth_context: Arc::new(AuthContext::new(
//                 DEFAULT_DATABASE_NAME.to_string(),
//                 DEFAULT_SUPPER_USER.to_string(),
//             )),
//             user_authenticator: UserAuthenticator::None,
//             config_map: Default::default(),
//         }
//     }

//     pub fn env(&self) -> &FrontendEnv {
//         &self.env
//     }

//     pub fn auth_context(&self) -> Arc<AuthContext> {
//         self.auth_context.clone()
//     }

//     pub fn database(&self) -> &str {
//         &self.auth_context.database
//     }

//     pub fn user_name(&self) -> &str {
//         &self.auth_context.user_name
//     }

//     pub fn config(&self) -> RwLockReadGuard<ConfigMap> {
//         self.config_map.read()
//     }

//     pub fn set_config(&self, key: &str, value: &str) -> Result<()> {
//         self.config_map.write().set(key, value)
//     }
// }



// pub struct MysqlSessionManagerImpl {
//     env: FrontendEnv,
//     observer_join_handle: JoinHandle<()>,
//     heartbeat_join_handle: JoinHandle<()>,
//     _heartbeat_shutdown_sender: Sender<()>,
// }


// impl MysqlSessionManager for MysqlSessionManagerImpl {
//     type MysqlSession = MysqlSessionImpl;

//     fn connect(
//         &self,
//         database: &str,
//         user_name: &str,
//     ) -> std::result::Result<Arc<Self::MysqlSession>, BoxedError> {
//         let catalog_reader = self.env.catalog_reader();
//         let reader = catalog_reader.read_guard();
//         if reader.get_database_by_name(database).is_err() {
//             return Err(Box::new(Error::new(
//                 ErrorKind::InvalidInput,
//                 format!("Not found database name: {}", database),
//             )));
//         }
//         let user_reader = self.env.user_info_reader();
//         let reader = user_reader.read_guard();
//         if let Some(user) = reader.get_user_by_name(user_name) {
//             if !user.can_login {
//                 return Err(Box::new(Error::new(
//                     ErrorKind::InvalidInput,
//                     format!("User {} is not allowed to login", user_name),
//                 )));
//             }
//             let user_authenticator = match &user.auth_info {
//                 None => UserAuthenticator::None,
//                 Some(auth_info) => {
//                     if auth_info.encryption_type == EncryptionType::Plaintext as i32 {
//                         UserAuthenticator::ClearText(auth_info.encrypted_value.clone())
//                     } else if auth_info.encryption_type == EncryptionType::Md5 as i32 {
//                         let mut salt = [0; 4];
//                         let mut rng = rand::thread_rng();
//                         rng.fill_bytes(&mut salt);
//                         UserAuthenticator::MD5WithSalt {
//                             encrypted_password: md5_hash_with_salt(
//                                 &auth_info.encrypted_value,
//                                 &salt,
//                             ),
//                             salt,
//                         }
//                     } else {
//                         return Err(Box::new(Error::new(
//                             ErrorKind::Unsupported,
//                             format!("Unsupported auth type: {}", auth_info.encryption_type),
//                         )));
//                     }
//                 }
//             };

//             Ok(MysqlSessionImpl::new(
//                 self.env.clone(),
//                 Arc::new(AuthContext::new(
//                     database.to_string(),
//                     user_name.to_string(),
//                 )),
//                 user_authenticator,
//             )
//             .into())
//         } else {
//             Err(Box::new(Error::new(
//                 ErrorKind::InvalidInput,
//                 format!("Role {} does not exist", user_name),
//             )))
//         }
//     }
// }


// impl MysqlSessionManagerImpl {
//     pub async fn new(opts: &FrontendOpts) -> Result<Self> {
//         let (env, join_handle, heartbeat_join_handle, heartbeat_shutdown_sender) =
//             FrontendEnv::init(opts).await?;
//         Ok(Self {
//             env,
//             observer_join_handle: join_handle,
//             heartbeat_join_handle,
//             _heartbeat_shutdown_sender: heartbeat_shutdown_sender,
//         })
//     }

//     /// Used in unit test. Called before `LocalMeta::stop`.
//     pub fn terminate(&self) {
//         self.observer_join_handle.abort();
//         self.heartbeat_join_handle.abort();
//     }
// }



// use crate::session::*;
// #[async_trait::async_trait]
// impl MysqlSession for MysqlSessionImpl {
//     async fn run_statement(
//         self: Arc<Self>,
//         sql: &str,
//     ) -> std::result::Result<PgResponse, BoxedError> {
//         // Parse sql.
//         println!("sql ========= {:?}",&sql);
//         let mut stmts = Parser::parse_sql(sql).map_err(|e| {
//             tracing::error!("failed to parse sql:\n{}:\n{}", sql, e);
//             e
//         })?;
//         if stmts.is_empty() {
//             return Ok(PgResponse::empty_result(
//                 pgwire::pg_response::StatementType::EMPTY,
//             ));
//         }
//         if stmts.len() > 1 {
//             return Ok(PgResponse::empty_result_with_notice(
//                 pgwire::pg_response::StatementType::EMPTY,
//                 "cannot insert multiple commands into statement".to_string(),
//             ));
//         }
//         let stmt = stmts.swap_remove(0);
//         println!("frontend::session stmt  =========  {:?}",&stmt);
//         let rsp = handle(self, stmt, sql).await.map_err(|e| {
//             tracing::error!("failed to handle sql:\n{}:\n{}", sql, e);
//             e
//         })?;
//         Ok(rsp)
//     }

//     async fn infer_return_type(
//         self: Arc<Self>,
//         sql: &str,
//     ) -> std::result::Result<Vec<PgFieldDescriptor>, BoxedError> {
//         // Parse sql.
//         let mut stmts = Parser::parse_sql(sql).map_err(|e| {
//             tracing::error!("failed to parse sql:\n{}:\n{}", sql, e);
//             e
//         })?;
//         if stmts.is_empty() {
//             return Ok(vec![]);
//         }
//         if stmts.len() > 1 {
//             return Err(Box::new(Error::new(
//                 ErrorKind::InvalidInput,
//                 "cannot insert multiple commands into statement",
//             )));
//         }
//         let stmt = stmts.swap_remove(0);
//         let rsp = infer(self, stmt, sql).map_err(|e| {
//             tracing::error!("failed to handle sql:\n{}:\n{}", sql, e);
//             e
//         })?;
//         Ok(rsp)
//     }

//     fn user_authenticator(&self) -> &UserAuthenticator {
//         &self.user_authenticator
//     }
// }


// fn infer(session: Arc<SessionImpl>, stmt: Statement, sql: &str) -> Result<Vec<PgFieldDescriptor>> {
//     let context = OptimizerContext::new(session, Arc::from(sql));
//     let session = context.session_ctx.clone();

//     let bound = {
//         let mut binder = Binder::new(
//             session.env().catalog_reader().read_guard(),
//             session.database().to_string(),
//         );
//         binder.bind(stmt)?
//     };

//     let root = Planner::new(context.into()).plan(bound)?;

//     let pg_descs = root
//         .schema()
//         .fields()
//         .iter()
//         .map(to_pg_field)
//         .collect::<Vec<PgFieldDescriptor>>();

//     Ok(pg_descs)
// }
// use crate::mysql_api::MySQLApi;
use std::io;
use std::sync::Arc;
use pgwire::pg_response::PgResponse;
use pgwire::pg_field_descriptor::PgFieldDescriptor;
use pgwire::pg_server::Session;
use pgwire::pg_server::UserAuthenticator;
use tokio::net::TcpListener;
use std::io::Result;
// use std::result::Result;
use msql_srv::*;
use msql_srv::OkResponse;

use std::io::Error;
use pgwire::pg_server::SessionManager;
use pgwire::pg_server::BoxedError;
use piestream_sqlparser::ast::Statement;
use piestream_sqlparser::parser::Parser;
use crate::session::{SessionImpl, AuthContext, FrontendEnv,SessionManagerImpl};
use crate::handler::handle;
use async_trait::async_trait;
use crate::FrontendOpts;
use clap::Parser as ClapParser;


pub async fn mysql_server(addr: &str) -> () {
    let addr = "127.0.0.1:4566".to_string();
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Server Listening at {}", &addr);
    // let session_mgr = session_mgr.clone();

    // let api = MySQLApi::new().await.unwrap();
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // let clone_api = api.clone();
        let api = MySQLApi::new().await.unwrap();

        tokio::spawn(async move {
            let result = AsyncMysqlIntermediary::run_on(api,socket).await;
            match result {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("fail to process incoming connection with e {}", e);
                }
            }
        });
    }
}


pub struct MySQLApi
{
    session_mgr: Arc<SessionManagerImpl>,
    salt: [u8; 20],
    id: u32,
}


// impl Clone for MySQLApi {
//     fn clone(&self) -> Self {
//         let mut bs = vec![0u8; 20];
//         let mut scramble: [u8; 20] = [0; 20];
//         for i in 0..20 {
//             scramble[i] = bs[i];
//             if scramble[i] == b'\0' || scramble[i] == b'$' {
//                 scramble[i] += 1;
//             }
//         }
//         Self {
//             // engine: self.engine.clone(),
//             // session_mgr: self.session_mgr,
//             salt: scramble,
//             id: self.id + 1,
//         }
//     }
// }

impl SessionManager for MySQLApi {
    type Session = SessionImpl;


    fn connect(
        &self,
        _database: &str,
        _user_name: &str,
    ) -> std::result::Result<Arc<Self::Session>, BoxedError> {
        Ok(self.session_ref())
    }
}


impl MySQLApi
{
    pub async fn new() -> Result<Self> {
        // let env = FrontendEnv::mock();
        let opts = FrontendOpts::parse();
        let (env, join_handle, heartbeat_join_handle, heartbeat_shutdown_sender) = 
        FrontendEnv::init(&opts).await.unwrap();
        println!("MySQLApi======================MySQLApi env new");
        let session_mgr = Arc::new(SessionManagerImpl::new(&opts).await.unwrap());
        Ok(
        Self {
            session_mgr: session_mgr,
            salt: [0; 20],
            id: 8,
        })
    }



    pub fn session_ref(&self) -> Arc<SessionImpl> {
        let env = FrontendEnv::mock();
        Arc::new(SessionImpl::new(
            env.clone(),
            Arc::new(AuthContext::new(
                "dev".to_string(),
                "root".to_string(),
            )),
            UserAuthenticator::None,
        ))
    }


    pub fn write_output<'a, W: std::io::Write + Send>(
        &self,
        // sql_result: &sql_engine::SQLResult,
        results: QueryResultWriter<'a, W>,
    ) -> Result<()> {
        return results.completed(OkResponse::default());
    }

    pub fn pgresponse_to_sqlresponse() {
        println!();
    }



}

#[async_trait]
impl<W: std::io::Write + Send> AsyncMysqlShim<W> for MySQLApi {
    type Error = Error;

    // fn version(&self) -> &str {
    //     self.version.as_str()
    // }

    fn connect_id(&self) -> u32 {
        self.id
    }

    fn default_auth_plugin(&self) -> &str {
        "mysql_native_password"
    }

    fn auth_plugin_for_username(&self, _user: &[u8]) -> &str {
        "mysql_native_password"
    }
    fn salt(&self) -> [u8; 20] {
        self.salt
    }

    async fn authenticate(
        &self,
        _auth_plugin: &str,
        _username: &[u8],
        _salt: &[u8],
        _auth_data: &[u8],
    ) -> bool {
        true
    }

    async fn authenticate_with_db(
        &self,
        _auth_plugin: &str,
        _username: &[u8],
        _salt: &[u8],
        _auth_data: &[u8],
        _db: &[u8],
    ) -> bool {
        true
    }

    async fn on_prepare<'a>(
        &'a mut self,
        query: &'a str,
        _writer: StatementMetaWriter<'a, W>,
    ) -> Result<()> {
        tracing::info!("on prepare query {}", query);
        Ok(())
    }

    async fn on_execute<'a>(
        &'a mut self,
        id: u32,
        _param: ParamParser<'a>,
        results: QueryResultWriter<'a, W>,
    ) -> Result<()> {
        // tracing::info!("on exec id {}", id);
        results.completed(OkResponse::default())

        // Ok(())
    }

    async fn on_close<'a>(&'a mut self, id: u32)
    where
        W: 'async_trait,
    {
        tracing::info!("on close id {}", id);
    }

    async fn on_query<'a>(
        &'a mut self,
        sql: &'a str,
        results: QueryResultWriter<'a, W>,
    ) -> Result<()> {
        println!("sql = {:?}",&sql);
        let lower_case_sql = sql.trim().to_lowercase();
        tracing::info!("input sql: {}", lower_case_sql);
        let db_name = "dev";
        let user_name = "root";
        let mut stmts = Parser::parse_sql(sql).map_err(|e| {
            tracing::error!("failed to parse sql:\n{}:\n{}", sql, e);
            e
        }).unwrap();
        let stmt = stmts.swap_remove(0);
        let env = FrontendEnv::mock();
        // let rsp = self.session_mgr.connect("dev", "root").run_statement(sql).await.unwrap();
        // let rsp = self.session_ref().run_statement(sql).await.unwrap();
        let rsp = self.session_mgr.connect("dev", "root").unwrap().run_statement(sql).await.unwrap();
        // results.completed(OkResponse::default());
        // self.write_output(results);
        println!("rsp ======== {:?}",&rsp);
        // self.write_output(results);
        let cols = [
            Column {
                table: "table".to_string(),
                column: "v1".to_string(),
                coltype: ColumnType::MYSQL_TYPE_LONGLONG,
                colflags: ColumnFlags::empty(),
            },
            Column {
                table: "table".to_string(),
                column: "v2".to_string(),
                coltype: ColumnType::MYSQL_TYPE_STRING,
                colflags: ColumnFlags::empty(),
            },
        ];
        let mut rw = results.start(&cols)?;
        rw.write_col(42)?;
        rw.write_col("b's value")?;
        // rw.end_row()?;
        // rw.write_col(22)?;
        // rw.write_col("c's value")?;
        // rw.finish();
        Ok(())
    }
    async fn on_init<'a>(
        &'a mut self,
        database_name: &'a str,
        writer: InitWriter<'a, W>,
    ) -> Result<()> {
        tracing::info!("enter db");
        writer.ok()
    }
}

