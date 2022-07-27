// Copyright 2022 PieDb Data
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::io;
use std::io::ErrorKind;
use std::result::Result;
use std::sync::Arc;

use tokio::net::{TcpListener, TcpStream};

use crate::pg_field_descriptor::PgFieldDescriptor;
use crate::pg_protocol::PgProtocol;
use crate::pg_response::PgResponse;

pub type BoxedError = Box<dyn std::error::Error + Send + Sync>;

/// The interface for a database system behind pgwire protocol.
/// We can mock it for testing purpose.
pub trait SessionManager: Send + Sync + 'static {
    type Session: Session;

    fn connect(&self, database: &str, user_name: &str) -> Result<Arc<Self::Session>, BoxedError>;
}

/// A psql connection. Each connection binds with a database. Switching database will need to
/// recreate another connection.
#[async_trait::async_trait]
pub trait Session: Send + Sync {
    async fn run_statement(self: Arc<Self>, sql: &str) -> Result<PgResponse, BoxedError>;
    async fn infer_return_type(
        self: Arc<Self>,
        sql: &str,
    ) -> Result<Vec<PgFieldDescriptor>, BoxedError>;
    fn user_authenticator(&self) -> &UserAuthenticator;
}

#[derive(Debug, Clone)]
pub enum UserAuthenticator {
    // No need to authenticate.
    None,
    // raw password in clear-text form.
    ClearText(Vec<u8>),
    // password encrypted with random salt.
    MD5WithSalt {
        encrypted_password: Vec<u8>,
        salt: [u8; 4],
    },
}

impl UserAuthenticator {
    pub fn authenticate(&self, password: &[u8]) -> bool {
        match self {
            UserAuthenticator::None => true,
            UserAuthenticator::ClearText(text) => password == text,
            UserAuthenticator::MD5WithSalt {
                encrypted_password, ..
            } => encrypted_password == password,
        }
    }
}

/// Binds a Tcp listener at `addr`. Spawn a coroutine to serve every new connection.
/// 绑定Tcp侦听器地址为addr。生成一个协同程序来服务于每一个新的连接
pub async fn pg_serve(addr: &str, session_mgr: Arc<impl SessionManager>) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await.unwrap();
    // accept connections and process them, spawning a new thread for each one
    // 接收连接并处理它们，为每个连接生成一个新线程
    tracing::info!("Server Listening at {}", addr);
    loop {
        let session_mgr = session_mgr.clone();
        let conn_ret = listener.accept().await;
        match conn_ret {
            Ok((stream, peer_addr)) => {
                tracing::info!("New connection: {}", peer_addr);
                //tokio启动新任务的方法tokio::spawn(),参数是pg_serve_conn，可以不断的启动一个服务程序
                tokio::spawn(async move {
                    // connection succeeded
                    pg_serve_conn(stream, session_mgr).await;
                    tracing::info!("Connection {} closed", peer_addr);
                });
            }

            Err(e) => {
                tracing::error!("Connection failure: {}", e);
            }
        }
    }
}

async fn pg_serve_conn(socket: TcpStream, session_mgr: Arc<impl SessionManager>) {
    //初始化一个psql连接的状态机，从tcp流读取pg消息并写回结果。
    let mut pg_proto = PgProtocol::new(socket, session_mgr);

    let mut unnamed_statement = Default::default();
    let mut unnamed_portal = Default::default();
    let mut named_statements = Default::default();
    let mut named_portals = Default::default();
    //loop循环接收
    loop {
        //调用g_proto.process()函数，进行消息处理
        let terminate = pg_proto
            .process(
                &mut unnamed_statement,
                &mut unnamed_portal,
                &mut named_statements,
                &mut named_portals,
            )
            .await;
        //match 结果的枚举类型
        match terminate {
            Ok(is_ter) => {
                if is_ter {
                    break;
                }
            }
            Err(e) => {
                if e.kind() == ErrorKind::UnexpectedEof {
                    break;
                }
                // Execution error should not break current connection.
                // For unexpected eof, just break and not print to log.
                tracing::error!("Error {:?}!", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::sync::Arc;

    use tokio_postgres::types::*;
    use tokio_postgres::NoTls;

    use crate::pg_field_descriptor::{PgFieldDescriptor, TypeOid};
    use crate::pg_response::{PgResponse, StatementType};
    use crate::pg_server::{pg_serve, Session, SessionManager, UserAuthenticator};
    use crate::types::Row;

    struct MockSessionManager {}

    impl SessionManager for MockSessionManager {
        type Session = MockSession;

        fn connect(
            &self,
            _database: &str,
            _user_name: &str,
        ) -> Result<Arc<Self::Session>, Box<dyn Error + Send + Sync>> {
            Ok(Arc::new(MockSession {}))
        }
    }

    struct MockSession {}

    #[async_trait::async_trait]
    impl Session for MockSession {
        async fn run_statement(
            self: Arc<Self>,
            sql: &str,
        ) -> Result<PgResponse, Box<dyn Error + Send + Sync>> {
            // split a statement and trim \' around the input param to construct result.
            // Ex:
            //    SELECT 'a','b' -> result: a , b
            let res: Vec<Option<String>> = sql
                .split(&[' ', ',', ';'])
                .skip(1)
                .map(|x| {
                    Some(
                        x.trim_start_matches('\'')
                            .trim_end_matches('\'')
                            .to_string(),
                    )
                })
                .collect();

            Ok(PgResponse::new(
                StatementType::SELECT,
                1,
                vec![Row::new(res)],
                // NOTE: Extended mode don't need.
                vec![],
                true,
            ))
        }

        fn user_authenticator(&self) -> &UserAuthenticator {
            &UserAuthenticator::None
        }

        async fn infer_return_type(
            self: Arc<Self>,
            sql: &str,
        ) -> Result<Vec<PgFieldDescriptor>, super::BoxedError> {
            let count = sql.split(&[' ', ',', ';']).skip(1).count();
            Ok(vec![
                PgFieldDescriptor::new("".to_string(), TypeOid::Varchar,);
                count
            ])
        }
    }

    // test_psql_extended_mode_explicit_simple
    // constrain:
    // - Only support simple SELECT statement.
    // - Must provide all type description of the generic types.
    // - Input description(params description) should include all the generic params description we
    //   need.
    #[tokio::test]
    async fn test_psql_extended_mode_explicit_simple() {
        let session_mgr = Arc::new(MockSessionManager {});
        tokio::spawn(async move { pg_serve("127.0.0.1:10000", session_mgr).await });

        // Connect to the database.
        let (mut client, connection) = tokio_postgres::connect("host=localhost port=10000", NoTls)
            .await
            .unwrap();

        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        // explicit parameter (test pre_statement)
        {
            let statement = client
                .prepare_typed("SELECT $1;", &[Type::VARCHAR])
                .await
                .unwrap();

            let rows = client.query(&statement, &[&"AA"]).await.unwrap();
            let value: &str = rows[0].get(0);
            assert_eq!(value, "AA");

            let rows = client.query(&statement, &[&"BB"]).await.unwrap();
            let value: &str = rows[0].get(0);
            assert_eq!(value, "BB");
        }
        // explicit parameter (test portal)
        {
            let transaction = client.transaction().await.unwrap();
            let statement = transaction
                .prepare_typed("SELECT $1;", &[Type::VARCHAR])
                .await
                .unwrap();
            let portal1 = transaction.bind(&statement, &[&"AA"]).await.unwrap();
            let portal2 = transaction.bind(&statement, &[&"BB"]).await.unwrap();
            let rows = transaction.query_portal(&portal1, 0).await.unwrap();
            let value: &str = rows[0].get(0);
            assert_eq!(value, "AA");
            let rows = transaction.query_portal(&portal2, 0).await.unwrap();
            let value: &str = rows[0].get(0);
            assert_eq!(value, "BB");
            transaction.rollback().await.unwrap();
        }
        // mix parameter
        {
            let statement = client
                .prepare_typed("SELECT $1,$2;", &[Type::VARCHAR, Type::VARCHAR])
                .await
                .unwrap();
            let rows = client.query(&statement, &[&"AA", &"BB"]).await.unwrap();
            let value: &str = rows[0].get(0);
            assert_eq!(value, "AA");
            let value: &str = rows[0].get(1);
            assert_eq!(value, "BB");

            let statement = client
                .prepare_typed("SELECT $1,$1;", &[Type::VARCHAR])
                .await
                .unwrap();
            let rows = client.query(&statement, &[&"AA"]).await.unwrap();
            let value: &str = rows[0].get(0);
            assert_eq!(value, "AA");
            let value: &str = rows[0].get(1);
            assert_eq!(value, "AA");

            let statement = client
                .prepare_typed(
                    "SELECT $2,$3,$1,$3,$2;",
                    &[Type::VARCHAR, Type::VARCHAR, Type::VARCHAR],
                )
                .await
                .unwrap();
            let rows = client
                .query(&statement, &[&"AA", &"BB", &"CC"])
                .await
                .unwrap();
            let value: &str = rows[0].get(0);
            assert_eq!(value, "BB");
            let value: &str = rows[0].get(1);
            assert_eq!(value, "CC");
            let value: &str = rows[0].get(2);
            assert_eq!(value, "AA");
            let value: &str = rows[0].get(3);
            assert_eq!(value, "CC");
            let value: &str = rows[0].get(4);
            assert_eq!(value, "BB");

            let statement = client
                .prepare_typed(
                    "SELECT $3,$1;",
                    &[Type::VARCHAR, Type::VARCHAR, Type::VARCHAR],
                )
                .await
                .unwrap();
            let rows = client
                .query(&statement, &[&"AA", &"BB", &"CC"])
                .await
                .unwrap();
            let value: &str = rows[0].get(0);
            assert_eq!(value, "CC");
            let value: &str = rows[0].get(1);
            assert_eq!(value, "AA");

            let statement = client
                .prepare_typed(
                    "SELECT $2,$1;",
                    &[Type::VARCHAR, Type::VARCHAR, Type::VARCHAR],
                )
                .await
                .unwrap();
            let rows = client
                .query(&statement, &[&"AA", &"BB", &"CC"])
                .await
                .unwrap();
            let value: &str = rows[0].get(0);
            assert_eq!(value, "BB");
            let value: &str = rows[0].get(1);
            assert_eq!(value, "AA");
        }
        // no params
        {
            let rows = client.query("SELECT 'AA','BB';", &[]).await.unwrap();
            let value: &str = rows[0].get(0);
            assert_eq!(value, "AA");
            let value: &str = rows[0].get(1);
            assert_eq!(value, "BB");
        }
    }
}
