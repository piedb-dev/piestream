// Copyright 2022 Singularity Data
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

use crate::pg_protocol::PgProtocol;
use crate::pg_response::PgResponse;

pub type BoxedError = Box<dyn std::error::Error + Send + Sync>;

/// The interface for a database system behind pgwire protocol.
/// We can mock it for testing purpose.
pub trait SessionManager: Send + Sync + 'static {
    type Session: Session;

    fn connect(&self, database: &str) -> Result<Arc<Self::Session>, BoxedError>;
}

/// A psql connection. Each connection binds with a database. Switching database will need to
/// recreate another connection.
#[async_trait::async_trait]
pub trait Session: Send + Sync {
    async fn run_statement(self: Arc<Self>, sql: &str) -> Result<PgResponse, BoxedError>;
}

/// Binds a Tcp listener at `addr`. Spawn a coroutine to serve every new connection.
pub async fn pg_serve(addr: &str, session_mgr: Arc<impl SessionManager>) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await.unwrap();
    // accept connections and process them, spawning a new thread for each one
    tracing::info!("Server Listening at {}", addr);
    loop {
        let session_mgr = session_mgr.clone();
        let conn_ret = listener.accept().await;
        match conn_ret {
            Ok((stream, peer_addr)) => {
                tracing::info!("New connection: {}", peer_addr);
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
    let mut pg_proto = PgProtocol::new(socket, session_mgr);
    loop {
        let terminate = pg_proto.process().await;
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
