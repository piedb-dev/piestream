// Copyright 2022 Piedb Data
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

pub type BoxedError = Box<dyn std::error::Error + Send + Sync>;
pub type SessionId = (i32, i32);
/// The interface for a database system behind pgwire protocol.
/// We can mock it for testing purpose.
pub trait SessionManager<VS>: Send + Sync + 'static
where
    VS: Stream<Item = RowSetResult> + Unpin + Send,
{
    type Session: Session<VS>;

    fn connect(&self, database: &str, user_name: &str) -> Result<Arc<Self::Session>, BoxedError>;

    fn cancel_queries_in_session(&self, session_id: SessionId);
}




#[async_trait::async_trait]
pub trait Session<VS>: Send + Sync
where
    VS: Stream<Item = RowSetResult> + Unpin + Send,
{
    async fn run_statement(
        self: Arc<Self>,
        sql: &str,
        format: bool,
    ) -> Result<PgResponse<VS>, BoxedError>;
    async fn infer_return_type(
        self: Arc<Self>,
        sql: &str,
    ) -> Result<Vec<PgFieldDescriptor>, BoxedError>;
    fn user_authenticator(&self) -> &UserAuthenticator;

    fn id(&self) -> SessionId;
}

#[derive(Debug, Clone)]
pub enum UserAuthenticator {
    // No need to authenticate.
    None,
    // raw password in clear-text form.
    ClearText(Vec<u8>),
    // password encrypted with random salt.
    Md5WithSalt {
        encrypted_password: Vec<u8>,
        salt: [u8; 4],
    },
}


#[derive(Debug, Clone)]
pub enum UserAuthenticator {
    // No need to authenticate.
    None,
    // raw password in clear-text form.
    ClearText(Vec<u8>),
    // password encrypted with random salt.
    Md5WithSalt {
        encrypted_password: Vec<u8>,
        salt: [u8; 4],
    },
}

impl UserAuthenticator {
    pub fn authenticate(&self, password: &[u8]) -> bool {
        match self {
            UserAuthenticator::None => true,
            UserAuthenticator::ClearText(text) => password == text,
            UserAuthenticator::Md5WithSalt {
                encrypted_password, ..
            } => encrypted_password == password,
        }
    }
}





