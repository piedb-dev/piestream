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

use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use fail::fail_point;
use piestream_hummock_sdk::compaction_group::StaticCompactionGroupId;
use piestream_hummock_sdk::{
    HummockContextId, HummockEpoch, HummockSstableId, HummockVersionId, LocalSstableInfo,
    SstIdRange,
};
use piestream_pb::hummock::{
    CompactTask, CompactTaskProgress, CompactionGroup, HummockSnapshot, HummockVersion,
    SubscribeCompactTasksResponse, VacuumTask,
};
use piestream_rpc_client::error::{Result, RpcError};
use piestream_rpc_client::HummockMetaClient;
use tonic::Streaming;

use crate::hummock::HummockManager;
use crate::storage::MemStore;

pub struct MockHummockMetaClient {
    hummock_manager: Arc<HummockManager<MemStore>>,
    context_id: HummockContextId,
}

impl MockHummockMetaClient {
    pub fn new(
        hummock_manager: Arc<HummockManager<MemStore>>,
        context_id: HummockContextId,
    ) -> MockHummockMetaClient {
        MockHummockMetaClient {
            hummock_manager,
            context_id,
        }
    }

    pub async fn get_compact_task(&self) -> Option<CompactTask> {
        self.hummock_manager
            .get_compact_task(StaticCompactionGroupId::StateDefault.into())
            .await
            .unwrap_or(None)
    }
}

fn mock_err(error: super::error::Error) -> RpcError {
    anyhow!("mock error: {}", error).into()
}

#[async_trait]
impl HummockMetaClient for MockHummockMetaClient {
    async fn unpin_version_before(&self, unpin_version_before: HummockVersionId) -> Result<()> {
        self.hummock_manager
            .unpin_version_before(self.context_id, unpin_version_before)
            .await
            .map_err(mock_err)
    }

    async fn get_current_version(&self) -> Result<HummockVersion> {
        Ok(self.hummock_manager.get_current_version().await)
    }

    async fn pin_snapshot(&self) -> Result<HummockSnapshot> {
        self.hummock_manager
            .pin_snapshot(self.context_id)
            .await
            .map_err(mock_err)
    }

    async fn get_epoch(&self) -> Result<HummockSnapshot> {
        self.hummock_manager.get_last_epoch().map_err(mock_err)
    }

    async fn unpin_snapshot(&self) -> Result<()> {
        self.hummock_manager
            .unpin_snapshot(self.context_id)
            .await
            .map_err(mock_err)
    }

    async fn unpin_snapshot_before(&self, pinned_epochs: HummockEpoch) -> Result<()> {
        self.hummock_manager
            .unpin_snapshot_before(
                self.context_id,
                HummockSnapshot {
                    committed_epoch: pinned_epochs,
                    current_epoch: pinned_epochs,
                },
            )
            .await
            .map_err(mock_err)
    }

    async fn get_new_sst_ids(&self, number: u32) -> Result<SstIdRange> {
        fail_point!("get_new_sst_ids_err", |_| Err(anyhow!(
            "failpoint get_new_sst_ids_err"
        )
        .into()));
        self.hummock_manager
            .get_new_sst_ids(number)
            .await
            .map_err(mock_err)
    }

    async fn report_compaction_task(&self, mut compact_task: CompactTask) -> Result<()> {
        self.hummock_manager
            .report_compact_task(self.context_id, &mut compact_task)
            .await
            .map(|_| ())
            .map_err(mock_err)
    }

    async fn commit_epoch(
        &self,
        epoch: HummockEpoch,
        sstables: Vec<LocalSstableInfo>,
    ) -> Result<()> {
        let sst_to_worker = sstables
            .iter()
            .map(|(_, sst)| (sst.id, self.context_id))
            .collect();
        self.hummock_manager
            .commit_epoch(epoch, sstables, sst_to_worker)
            .await
            .map_err(mock_err)
    }

    async fn subscribe_compact_tasks(
        &self,
        _max_concurrent_task_number: u64,
    ) -> Result<Streaming<SubscribeCompactTasksResponse>> {
        unimplemented!()
    }

    async fn report_compaction_task_progress(
        &self,
        _progress: Vec<CompactTaskProgress>,
    ) -> Result<()> {
        unimplemented!()
    }

    async fn report_vacuum_task(&self, _vacuum_task: VacuumTask) -> Result<()> {
        Ok(())
    }

    async fn get_compaction_groups(&self) -> Result<Vec<CompactionGroup>> {
        todo!()
    }

    async fn trigger_manual_compaction(
        &self,
        _compaction_group_id: u64,
        _table_id: u32,
        _level: u32,
    ) -> Result<()> {
        todo!()
    }

    async fn report_full_scan_task(&self, _sst_ids: Vec<HummockSstableId>) -> Result<()> {
        unimplemented!()
    }

    async fn trigger_full_gc(&self, _sst_retention_time_sec: u64) -> Result<()> {
        unimplemented!()
    }
}

impl MockHummockMetaClient {
    pub fn hummock_manager_ref(&self) -> Arc<HummockManager<MemStore>> {
        self.hummock_manager.clone()
    }
}
