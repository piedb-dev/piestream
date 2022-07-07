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

pub use anyhow::anyhow;
use piestream_common::error::{ErrorCode, RwError};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, RpcError>;

#[derive(Error, Debug)]
pub enum RpcError {
    #[error("Transport error: {0}")]
    TrasnportError(#[from] tonic::transport::Error),

    #[error("gRPC status: {0}")]
    GrpcStatus(#[from] tonic::Status),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<RpcError> for RwError {
    fn from(r: RpcError) -> Self {
        ErrorCode::RpcError(r.into()).into()
    }
}
