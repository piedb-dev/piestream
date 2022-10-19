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

use std::backtrace::Backtrace;

use either::Either;
use piestream_common::array::ArrayError;
use piestream_common::error::{BoxedError, Error, TrackingIssue};
use piestream_common::util::value_encoding::error::ValueEncodingError;
use piestream_connector::error::ConnectorError;
use piestream_connector::sink::SinkError;
use piestream_expr::ExprError;
use piestream_rpc_client::error::RpcError;
use piestream_storage::error::StorageError;

use super::Barrier;

#[derive(thiserror::Error, Debug)]
enum Inner {
    #[error("Storage error: {0}")]
    Storage(
        #[backtrace]
        #[source]
        StorageError,
    ),

    #[error("Chunk operation error: {0}")]
    EvalError(Either<ArrayError, ExprError>),

    // TODO: remove this after state table is fully used
    #[error("Serialize/deserialize error: {0}")]
    SerdeError(BoxedError),

    #[error("Sink error: {0}")]
    SinkError(SinkError),

    #[error("RPC error: {0}")]
    RpcError(RpcError),

    #[error("Channel `{0}` closed")]
    ChannelClosed(String),

    #[error("Failed to align barrier: expected {0:?} but got {1:?}")]
    AlignBarrier(Box<Barrier>, Box<Barrier>),

    #[error("Connector error: {0}")]
    ConnectorError(BoxedError),

    #[error("Feature is not yet implemented: {0}, {1}")]
    NotImplemented(String, TrackingIssue),

    #[error(transparent)]
    Internal(anyhow::Error),
}

impl StreamExecutorError {
    fn serde_error(error: impl Error) -> Self {
        Inner::SerdeError(error.into()).into()
    }

    pub fn channel_closed(name: impl Into<String>) -> Self {
        Inner::ChannelClosed(name.into()).into()
    }

    pub fn align_barrier(expected: Barrier, received: Barrier) -> Self {
        Inner::AlignBarrier(expected.into(), received.into()).into()
    }

    pub fn connector_error(error: impl Error) -> Self {
        Inner::ConnectorError(error.into()).into()
    }

    pub fn not_implemented(error: impl Into<String>, issue: impl Into<TrackingIssue>) -> Self {
        Inner::NotImplemented(error.into(), issue.into()).into()
    }
}

/// Error type for streaming executors.
#[derive(thiserror::Error)]
#[error("{inner}")]
pub struct StreamExecutorError {
    #[from]
    inner: Inner,
    backtrace: Backtrace,
}

impl std::fmt::Debug for StreamExecutorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::error::Error;

        write!(f, "{}", self.inner)?;
        writeln!(f)?;
        if let Some(backtrace) = self.inner.backtrace() {
            write!(f, "  backtrace of inner error:\n{}", backtrace)?;
        } else {
            write!(
                f,
                "  backtrace of `StreamExecutorError`:\n{}",
                self.backtrace
            )?;
        }
        Ok(())
    }
}

/// Storage error.
impl From<StorageError> for StreamExecutorError {
    fn from(s: StorageError) -> Self {
        Inner::Storage(s).into()
    }
}

/// Chunk operation error.
impl From<ArrayError> for StreamExecutorError {
    fn from(e: ArrayError) -> Self {
        Inner::EvalError(Either::Left(e)).into()
    }
}

impl From<ExprError> for StreamExecutorError {
    fn from(e: ExprError) -> Self {
        Inner::EvalError(Either::Right(e)).into()
    }
}

/// Internal error.
impl From<anyhow::Error> for StreamExecutorError {
    fn from(a: anyhow::Error) -> Self {
        Inner::Internal(a).into()
    }
}

/// Serialize/deserialize error.
impl From<memcomparable::Error> for StreamExecutorError {
    fn from(m: memcomparable::Error) -> Self {
        Self::serde_error(m)
    }
}
impl From<ValueEncodingError> for StreamExecutorError {
    fn from(e: ValueEncodingError) -> Self {
        Self::serde_error(e)
    }
}

impl From<RpcError> for StreamExecutorError {
    fn from(e: RpcError) -> Self {
        Inner::RpcError(e).into()
    }
}

/// Connector error.
impl From<ConnectorError> for StreamExecutorError {
    fn from(s: ConnectorError) -> Self {
        Self::connector_error(s)
    }
}

impl From<SinkError> for StreamExecutorError {
    fn from(e: SinkError) -> Self {
        Inner::SinkError(e).into()
    }
}

pub type StreamExecutorResult<T> = std::result::Result<T, StreamExecutorError>;

#[cfg(test)]
mod tests {
    use piestream_common::bail;

    use super::*;

    fn func_return_error() -> StreamExecutorResult<()> {
        bail!("test_error")
    }

    #[test]
    #[should_panic]
    #[ignore]
    fn executor_error_ui_test_1() {
        // For this test, ensure that we have only one backtrace from error when panic.
        func_return_error().unwrap();
    }

    #[test]
    #[ignore]
    fn executor_error_ui_test_2() {
        // For this test, ensure that we have only one backtrace from error when panic.
        func_return_error().map_err(|e| println!("{:?}", e)).ok();
    }
}
