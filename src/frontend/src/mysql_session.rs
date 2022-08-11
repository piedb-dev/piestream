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

