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

use std::fmt::{Debug, Formatter};

use futures::Stream;
use futures_async_stream::try_stream;
use log::debug;
use risingwave_common::array::DataChunk;
use risingwave_common::error::{Result, RwError};
use risingwave_pb::batch_plan::{PlanNode as BatchPlanProst, TaskId, TaskOutputId};
use risingwave_pb::common::HostAddress;
use risingwave_rpc_client::{ComputeClient, ExchangeSource};
use uuid::Uuid;

use super::HummockSnapshotManagerRef;
use crate::scheduler::execution::QueryExecution;
use crate::scheduler::plan_fragmenter::Query;
use crate::scheduler::worker_node_manager::WorkerNodeManagerRef;
use crate::scheduler::ExecutionContextRef;

pub trait DataChunkStream = Stream<Item = Result<DataChunk>>;

pub struct QueryResultFetcher {
    // TODO: Remove these after implemented worker node level snapshot pinnning
    epoch: u64,
    hummock_snapshot_manager: HummockSnapshotManagerRef,

    task_output_id: TaskOutputId,
    task_host: HostAddress,
}

/// Manages execution of batch queries.
#[derive(Clone)]
pub struct QueryManager {
    worker_node_manager: WorkerNodeManagerRef,
    hummock_snapshot_manager: HummockSnapshotManagerRef,
}

impl QueryManager {
    pub fn new(
        worker_node_manager: WorkerNodeManagerRef,
        hummock_snapshot_manager: HummockSnapshotManagerRef,
    ) -> Self {
        Self {
            worker_node_manager,
            hummock_snapshot_manager,
        }
    }

    /// Schedule query to single node.
    ///
    /// This is kept for dml only.
    pub async fn schedule_single(
        &self,
        _context: ExecutionContextRef,
        plan: BatchPlanProst,
    ) -> Result<impl Stream<Item = Result<DataChunk>>> {
        let worker_node_addr = self.worker_node_manager.next_random()?.host.unwrap();
        let compute_client: ComputeClient = ComputeClient::new((&worker_node_addr).into()).await?;

        // Build task id and task sink id
        let task_id = TaskId {
            query_id: Uuid::new_v4().to_string(),
            stage_id: 0,
            task_id: 0,
        };
        let task_output_id = TaskOutputId {
            task_id: Some(task_id.clone()),
            output_id: 0,
        };

        let epoch = self.hummock_snapshot_manager.get_epoch().await?;

        compute_client
            .create_task(task_id.clone(), plan, epoch)
            .await?;

        let query_result_fetcher = QueryResultFetcher::new(
            epoch,
            self.hummock_snapshot_manager.clone(),
            task_output_id,
            worker_node_addr,
        );

        Ok(query_result_fetcher.run())
    }

    pub async fn schedule(
        &self,
        _context: ExecutionContextRef,
        query: Query,
    ) -> Result<impl DataChunkStream> {
        // Cheat compiler to resolve type
        let epoch = self.hummock_snapshot_manager.get_epoch().await?;

        let query_execution = QueryExecution::new(
            query,
            epoch,
            self.worker_node_manager.clone(),
            self.hummock_snapshot_manager.clone(),
        );

        let query_result_fetcher = query_execution.start().await?;

        Ok(query_result_fetcher.run())
    }
}

impl QueryResultFetcher {
    pub fn new(
        epoch: u64,
        hummock_snapshot_manager: HummockSnapshotManagerRef,
        task_output_id: TaskOutputId,
        task_host: HostAddress,
    ) -> Self {
        Self {
            epoch,
            hummock_snapshot_manager,
            task_output_id,
            task_host,
        }
    }

    #[try_stream(ok = DataChunk, error = RwError)]
    async fn run(self) {
        debug!(
            "Starting to run query result fetcher, task output id: {:?}, task_host: {:?}",
            self.task_output_id, self.task_host
        );
        let compute_client: ComputeClient = ComputeClient::new((&self.task_host).into()).await?;

        let mut source = compute_client.get_data(self.task_output_id).await?;
        while let Some(chunk) = source.take_data().await? {
            yield chunk;
        }

        let epoch = self.epoch;
        // Unpin corresponding snapshot.
        self.hummock_snapshot_manager.unpin_snapshot(epoch).await?;
    }
}

impl Debug for QueryResultFetcher {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryResultFetcher")
            .field("epoch", &self.epoch)
            .field("task_output_id", &self.task_output_id)
            .field("task_host", &self.task_host)
            .finish()
    }
}
