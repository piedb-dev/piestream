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

use std::fmt::{Debug, Formatter};
use std::future::Future;

use futures::StreamExt;
use piestream_common::array::DataChunk;
use piestream_common::error::Result;
use piestream_pb::batch_plan::exchange_source::LocalExecutePlan::Plan;
use piestream_pb::batch_plan::{ExchangeSource as ProstExchangeSource, TaskOutputId};
use piestream_pb::task_service::{ExecuteRequest, GetDataResponse};
use piestream_rpc_client::ComputeClient;
use tonic::Streaming;

use crate::exchange_source::ExchangeSource;

/// Use grpc client as the source.
pub struct GrpcExchangeSource {
    stream: Streaming<GetDataResponse>,

    task_output_id: TaskOutputId,
}

impl GrpcExchangeSource {
    pub async fn create(exchange_source: ProstExchangeSource) -> Result<Self> {
        let addr = exchange_source.get_host()?.into();
        let task_output_id = exchange_source.get_task_output_id()?.clone();
        let task_id = task_output_id.get_task_id()?.clone();
        let client = ComputeClient::new(addr).await?;
        let local_execute_plan = exchange_source.local_execute_plan;
        let stream = match local_execute_plan {
            // When in the local execution mode, `GrpcExchangeSource` would send out
            // `ExecuteRequest` and get the data chunks back in a single RPC.
            Some(local_execute_plan) => {
                let plan = try_match_expand!(local_execute_plan, Plan)?;
                let execute_request = ExecuteRequest {
                    task_id: Some(task_id),
                    plan: plan.plan,
                    epoch: plan.epoch,
                };
                client.execute(execute_request).await?
            }
            None => client.get_data(task_output_id.clone()).await?,
        };
        let source = Self {
            stream,
            task_output_id,
        };
        Ok(source)
    }
}

impl Debug for GrpcExchangeSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GrpcExchangeSource")
            .field("task_output_id", &self.task_output_id)
            .finish()
    }
}

impl ExchangeSource for GrpcExchangeSource {
    type TakeDataFuture<'a> = impl Future<Output = Result<Option<DataChunk>>>;

    fn take_data(&mut self) -> Self::TakeDataFuture<'_> {
        async {
            let res = match self.stream.next().await {
                None => return Ok(None),
                Some(r) => r,
            };
            let task_data = res?;
            let data = DataChunk::from_protobuf(task_data.get_record_batch()?)?.compact()?;
            trace!(
                "Receiver taskOutput = {:?}, data = {:?}",
                self.task_output_id,
                data
            );

            Ok(Some(data))
        }
    }
}