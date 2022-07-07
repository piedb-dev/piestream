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

use std::time::Duration;

use piestream_common::util::addr::HostAddr;
use piestream_pb::batch_plan::exchange_info::DistributionMode;
use piestream_pb::batch_plan::{ExchangeInfo, PlanFragment, PlanNode, TaskId, TaskOutputId};
use piestream_pb::task_service::exchange_service_client::ExchangeServiceClient;
use piestream_pb::task_service::task_service_client::TaskServiceClient;
use piestream_pb::task_service::{
    CreateTaskRequest, CreateTaskResponse, ExecuteRequest, GetDataRequest, GetDataResponse,
    GetStreamRequest, GetStreamResponse,
};
use tonic::transport::{Channel, Endpoint};
use tonic::Streaming;

use crate::error::Result;

#[derive(Clone)]
pub struct ComputeClient {
    pub exchange_client: ExchangeServiceClient<Channel>,
    pub task_client: TaskServiceClient<Channel>,
    pub addr: HostAddr,
}

impl ComputeClient {
    pub async fn new(addr: HostAddr) -> Result<Self> {
        let channel = Endpoint::from_shared(format!("http://{}", &addr))?
            .connect_timeout(Duration::from_secs(5))
            .connect()
            .await?;
        let exchange_client = ExchangeServiceClient::new(channel.clone());
        let task_client = TaskServiceClient::new(channel);
        Ok(Self {
            exchange_client,
            task_client,
            addr,
        })
    }

    pub async fn get_data(&self, output_id: TaskOutputId) -> Result<Streaming<GetDataResponse>> {
        Ok(self
            .exchange_client
            .to_owned()
            .get_data(GetDataRequest {
                task_output_id: Some(output_id),
            })
            .await?
            .into_inner())
    }

    pub async fn get_stream(
        &self,
        up_fragment_id: u32,
        down_fragment_id: u32,
    ) -> Result<Streaming<GetStreamResponse>> {
        Ok(self
            .exchange_client
            .to_owned()
            .get_stream(GetStreamRequest {
                up_fragment_id,
                down_fragment_id,
            })
            .await
            .inspect_err(|_| {
                tracing::error!(
                    "failed to create stream from remote_input {} from fragment {} to fragment {}",
                    self.addr,
                    up_fragment_id,
                    down_fragment_id
                )
            })?
            .into_inner())
    }

    // TODO: Remove this
    pub async fn create_task(&self, task_id: TaskId, plan: PlanNode, epoch: u64) -> Result<()> {
        let plan = PlanFragment {
            root: Some(plan),
            exchange_info: Some(ExchangeInfo {
                mode: DistributionMode::Single as i32,
                ..Default::default()
            }),
        };
        let _ = self
            .create_task_inner(CreateTaskRequest {
                task_id: Some(task_id),
                plan: Some(plan),
                epoch,
            })
            .await?;
        Ok(())
    }

    pub async fn create_task2(
        &self,
        task_id: TaskId,
        plan: PlanFragment,
        epoch: u64,
    ) -> Result<()> {
        let _ = self
            .create_task_inner(CreateTaskRequest {
                task_id: Some(task_id),
                plan: Some(plan),
                epoch,
            })
            .await?;
        Ok(())
    }

    async fn create_task_inner(&self, req: CreateTaskRequest) -> Result<CreateTaskResponse> {
        Ok(self
            .task_client
            .to_owned()
            .create_task(req)
            .await?
            .into_inner())
    }

    pub async fn execute(&self, req: ExecuteRequest) -> Result<Streaming<GetDataResponse>> {
        Ok(self.task_client.to_owned().execute(req).await?.into_inner())
    }
}
