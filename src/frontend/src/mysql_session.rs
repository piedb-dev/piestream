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

use std::sync::Arc;
use pgwire::pg_response::PgResponse;
use pgwire::pg_server::Session;
use pgwire::pg_server::UserAuthenticator;
use tokio::net::TcpListener;
use std::io::{ Result,Error};
use msql_srv::*;
use msql_srv::OkResponse;
// use msql_srv::ErrorKind;
use pgwire::pg_server::SessionManager;
use pgwire::pg_response::StatementType;
use pgwire::pg_server::BoxedError;
use crate::session::{SessionImpl, AuthContext, FrontendEnv,SessionManagerImpl};
use async_trait::async_trait;



// const CURRENT_DB: &str = "select database()";
const SHOW_DB: &str = "show databases";
const SHOW_TABLES: &str = "show tables";
const VERSION_COMMENT:&str = "select @@version_comment limit 1";
// const SHOW_SCHEMAS: &str = "show schemas";
// const SHOW_VERSION: &str = "select version()";



pub async fn mysql_server(addr: &str,session_mgr: Arc<SessionManagerImpl>) -> () {
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Server Listening at {}", &addr);
    loop {
        let session_mgr = session_mgr.clone();
        let (socket, _) = listener.accept().await.unwrap();
        let api = MySQLApi::new(session_mgr).await.unwrap();
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
    session: Arc<SessionImpl>,
    salt: [u8; 20],
    id: u32,
}

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
    pub async fn new(session_mgr: Arc<SessionManagerImpl>) -> Result<Self> {
        let session = session_mgr.clone().connect("dev", "root").unwrap();
        Ok(
        Self {
            session: session,
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
        results: QueryResultWriter<'a, W>,
        pg_results: PgResponse
    ) -> Result<()> {
        let stmt_type = pg_results.get_stmt_type();
        // todo alter more StatementType 
        match stmt_type {
            StatementType::CREATE_TABLE => {
                return results.completed(OkResponse::default());
            },
            StatementType::INSERT => {
                return results.completed(OkResponse::default());
            },
            // todo support more query output
            StatementType::SELECT => {
                let values = pg_results.values();
                let row_desc = pg_results.get_row_desc();
                let mut cols = vec![];
                for pg_field in row_desc {
                    let pg_field_name = pg_field.get_name();
                    // let type_oid = pg_field.get_type_oid();
                    let col = Column {
                        table: "table".to_string(),
                        column: pg_field_name.to_string(),
                        coltype: ColumnType::MYSQL_TYPE_LONGLONG,
                        colflags: ColumnFlags::empty(),
                    };
                    cols.push(col);
                }
                let mut rw = results.start(&cols)?;
                for value in values {
                    let v1 = value[0].as_ref().unwrap();
                    let v2 = value[1].as_ref().unwrap();
                    rw.write_col(v1)?;
                    rw.write_col(v2)?;
                    rw.end_row()?;
                };
                rw.finish()
            },
            _ => {
                return results.completed(OkResponse::default());
            }
        }
    }

    pub fn pgresponse_to_sqlresponse() {
        println!("pgresponse_to_sql");
    }

    pub fn show_dbs<'a, W: std::io::Write + Send>(
        &'a self,
        results: QueryResultWriter<'a, W>,
        pg_results: PgResponse
    ) -> Result<()> {
        let values = pg_results.values();
        let str_col = [Column {
            table: "".to_string(),
            column: "Databases".to_string(),
            coltype: ColumnType::MYSQL_TYPE_STRING,
            colflags: ColumnFlags::empty(),
        }];
        let mut rw = results.start(&str_col)?;
        rw.write_col("information_schema")?;
        rw.end_row()?;
        rw.write_col("mysql")?;
        rw.end_row()?;
        rw.write_col("performance_schema")?;
        rw.end_row()?;
        for db in values {
            let db_name = db[0].as_ref().unwrap();
            rw.write_col(db_name)?;
            rw.end_row()?;
        }
        rw.finish()
    }

    pub fn show_tables<'a, W: std::io::Write + Send>(
        &'a self,
        results: QueryResultWriter<'a, W>,
        _pg_results: PgResponse
    ) -> Result<()> {
        results.completed(OkResponse::default())
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
        _id: u32,
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
        let lower_case_sql = sql.trim().to_lowercase();
        tracing::info!("input sql: {}", lower_case_sql);
        let session = self.session.clone();
        if lower_case_sql == VERSION_COMMENT {
            return results.completed(OkResponse::default());
        } else {
            let rsp = session.run_statement(sql).await;
            match rsp {
                Ok(res) => {
                    match lower_case_sql.as_str() {
                        // todo add more ddl command function
                        SHOW_DB => self.show_dbs(results,res),
                        SHOW_TABLES => self.show_tables(results,res),
                        _ => {
                            self.write_output(results,res)
                        }
                    }
                },
                Err(e) => {
                    // todo support more ErrorKind
                    return results.error(ErrorKind::ER_ABORTING_CONNECTION, e.to_string().as_bytes())
                }
            }            
        }
    }
    async fn on_init<'a>(
        &'a mut self,
        _database_name: &'a str,
        writer: InitWriter<'a, W>,
    ) -> Result<()> {
        tracing::info!("enter db");
        writer.ok()
    }
}


#[test]
fn test_mysql() {

}