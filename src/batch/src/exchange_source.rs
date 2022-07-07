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

use std::fmt::Debug;
use std::future::Future;

use piestream_common::array::DataChunk;

use crate::execution::grpc_exchange::GrpcExchangeSource;
use crate::execution::local_exchange::LocalExchangeSource;
use crate::executor::test_utils::FakeExchangeSource;

/// Each `ExchangeSource` maps to one task, it takes the execution result from task chunk by chunk.
pub trait ExchangeSource: Send + Debug {
    type TakeDataFuture<'a>: Future<Output = piestream_common::error::Result<Option<DataChunk>>>
        + 'a
    where
        Self: 'a;
    fn take_data(&mut self) -> Self::TakeDataFuture<'_>;
}

#[derive(Debug)]
pub enum ExchangeSourceImpl {
    Grpc(GrpcExchangeSource),
    Local(LocalExchangeSource),
    Fake(FakeExchangeSource),
}

impl ExchangeSourceImpl {
    pub(crate) async fn take_data(
        &mut self,
    ) -> piestream_common::error::Result<Option<DataChunk>> {
        match self {
            ExchangeSourceImpl::Grpc(grpc) => grpc.take_data().await,
            ExchangeSourceImpl::Local(local) => local.take_data().await,
            ExchangeSourceImpl::Fake(fake) => fake.take_data().await,
        }
    }
}
