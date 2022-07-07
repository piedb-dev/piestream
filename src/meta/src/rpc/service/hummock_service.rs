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

use std::collections::HashSet;
use std::sync::Arc;

use piestream_common::catalog::TableId;
use piestream_common::error::{tonic_err, ErrorCode};
use piestream_pb::hummock::hummock_manager_service_server::HummockManagerService;
use piestream_pb::hummock::*;
use tonic::{Request, Response, Status};

use crate::hummock::compaction::ManualCompactionOption;
use crate::hummock::compaction_group::manager::CompactionGroupManagerRef;
use crate::hummock::{CompactorManagerRef, HummockManagerRef, VacuumTrigger};
use crate::rpc::service::RwReceiverStream;
use crate::storage::MetaStore;
use crate::stream::FragmentManagerRef;

pub struct HummockServiceImpl<S>
where
    S: MetaStore,
{
    hummock_manager: HummockManagerRef<S>,
    compactor_manager: CompactorManagerRef,
    vacuum_trigger: Arc<VacuumTrigger<S>>,
    compaction_group_manager: CompactionGroupManagerRef<S>,
    fragment_manager: FragmentManagerRef<S>,
}

impl<S> HummockServiceImpl<S>
where
    S: MetaStore,
{
    pub fn new(
        hummock_manager: HummockManagerRef<S>,
        compactor_manager: CompactorManagerRef,
        vacuum_trigger: Arc<VacuumTrigger<S>>,
        compaction_group_manager: CompactionGroupManagerRef<S>,
        fragment_manager: FragmentManagerRef<S>,
    ) -> Self {
        HummockServiceImpl {
            hummock_manager,
            compactor_manager,
            vacuum_trigger,
            compaction_group_manager,
            fragment_manager,
        }
    }
}

#[async_trait::async_trait]
impl<S> HummockManagerService for HummockServiceImpl<S>
where
    S: MetaStore,
{
    type SubscribeCompactTasksStream = RwReceiverStream<SubscribeCompactTasksResponse>;

    async fn pin_version(
        &self,
        request: Request<PinVersionRequest>,
    ) -> Result<Response<PinVersionResponse>, Status> {
        let req = request.into_inner();
        let result = self
            .hummock_manager
            .pin_version(req.context_id, req.last_pinned)
            .await;
        match result {
            Ok(pinned_version) => Ok(Response::new(PinVersionResponse {
                status: None,
                pinned_version: Some(pinned_version),
            })),
            Err(e) => Err(tonic_err(e)),
        }
    }

    async fn unpin_version(
        &self,
        request: Request<UnpinVersionRequest>,
    ) -> Result<Response<UnpinVersionResponse>, Status> {
        let req = request.into_inner();
        let result = self
            .hummock_manager
            .unpin_version(req.context_id, req.pinned_version_ids)
            .await;
        match result {
            Ok(_) => Ok(Response::new(UnpinVersionResponse { status: None })),
            Err(e) => Err(tonic_err(e)),
        }
    }

    async fn report_compaction_tasks(
        &self,
        request: Request<ReportCompactionTasksRequest>,
    ) -> Result<Response<ReportCompactionTasksResponse>, Status> {
        let req = request.into_inner();
        match req.compact_task {
            None => Ok(Response::new(ReportCompactionTasksResponse {
                status: None,
            })),
            Some(compact_task) => {
                let result = self
                    .hummock_manager
                    .report_compact_task(&compact_task)
                    .await;
                match result {
                    Ok(_) => Ok(Response::new(ReportCompactionTasksResponse {
                        status: None,
                    })),
                    Err(e) => Err(tonic_err(e)),
                }
            }
        }
    }

    async fn pin_snapshot(
        &self,
        request: Request<PinSnapshotRequest>,
    ) -> Result<Response<PinSnapshotResponse>, Status> {
        let req = request.into_inner();
        let result = self
            .hummock_manager
            .pin_snapshot(req.context_id, req.last_pinned)
            .await;
        match result {
            Ok(hummock_snapshot) => Ok(Response::new(PinSnapshotResponse {
                status: None,
                snapshot: Some(hummock_snapshot),
            })),
            Err(e) => Err(tonic_err(e)),
        }
    }

    async fn unpin_snapshot(
        &self,
        request: Request<UnpinSnapshotRequest>,
    ) -> Result<Response<UnpinSnapshotResponse>, Status> {
        let req = request.into_inner();
        if let Err(e) = self
            .hummock_manager
            .unpin_snapshot(req.context_id, req.snapshots)
            .await
        {
            return Err(tonic_err(e));
        }
        Ok(Response::new(UnpinSnapshotResponse { status: None }))
    }

    async fn unpin_snapshot_before(
        &self,
        request: Request<UnpinSnapshotBeforeRequest>,
    ) -> Result<Response<UnpinSnapshotBeforeResponse>, Status> {
        let req = request.into_inner();
        if let Err(e) = self
            .hummock_manager
            .unpin_snapshot_before(req.context_id, req.min_snapshot.unwrap())
            .await
        {
            return Err(tonic_err(e));
        }
        Ok(Response::new(UnpinSnapshotBeforeResponse { status: None }))
    }

    async fn get_new_table_id(
        &self,
        _request: Request<GetNewTableIdRequest>,
    ) -> Result<Response<GetNewTableIdResponse>, Status> {
        let result = self.hummock_manager.get_new_table_id().await;
        match result {
            Ok(table_id) => Ok(Response::new(GetNewTableIdResponse {
                status: None,
                table_id,
            })),
            Err(e) => Err(tonic_err(e)),
        }
    }

    async fn subscribe_compact_tasks(
        &self,
        request: Request<SubscribeCompactTasksRequest>,
    ) -> Result<Response<Self::SubscribeCompactTasksStream>, Status> {
        let context_id = request.into_inner().context_id;
        // check_context and add_compactor as a whole is not atomic, but compactor_manager will
        // remove invalid compactor eventually.
        if !self.hummock_manager.check_context(context_id).await {
            return Err(tonic_err(ErrorCode::MetaError(format!(
                "invalid hummock context {}",
                context_id
            ))));
        }
        let rx = self.compactor_manager.add_compactor(context_id);
        Ok(Response::new(RwReceiverStream::new(rx)))
    }

    async fn report_vacuum_task(
        &self,
        request: Request<ReportVacuumTaskRequest>,
    ) -> Result<Response<ReportVacuumTaskResponse>, Status> {
        if let Some(vacuum_task) = request.into_inner().vacuum_task {
            self.vacuum_trigger
                .report_vacuum_task(vacuum_task)
                .await
                .map_err(tonic_err)?;
        }
        Ok(Response::new(ReportVacuumTaskResponse { status: None }))
    }

    async fn get_compaction_groups(
        &self,
        _request: Request<GetCompactionGroupsRequest>,
    ) -> Result<Response<GetCompactionGroupsResponse>, Status> {
        let resp = GetCompactionGroupsResponse {
            status: None,
            compaction_groups: self
                .compaction_group_manager
                .compaction_groups()
                .await
                .iter()
                .map(|cg| cg.into())
                .collect(),
        };
        Ok(Response::new(resp))
    }

    async fn trigger_manual_compaction(
        &self,
        request: Request<TriggerManualCompactionRequest>,
    ) -> Result<Response<TriggerManualCompactionResponse>, Status> {
        let request = request.into_inner();
        let compaction_group_id = request.compaction_group_id;
        let mut option = ManualCompactionOption {
            level: request.level as usize,
            ..Default::default()
        };

        // rewrite the key_range
        match request.key_range {
            Some(key_range) => {
                option.key_range = key_range;
            }

            None => {
                option.key_range = KeyRange {
                    inf: true,
                    ..Default::default()
                }
            }
        }

        // get internal_table_id by fragment_manager
        let table_id = TableId::new(request.table_id);
        if let Ok(table_frgament) = self
            .fragment_manager
            .select_table_fragments_by_table_id(&table_id)
            .await
        {
            option.internal_table_id = HashSet::from_iter(table_frgament.internal_table_ids());
        }
        option.internal_table_id.insert(request.table_id); // need to handle outter table_id (mv)

        tracing::info!(
            "Try trigger_manual_compaction compaction_group_id {} option {:?}",
            compaction_group_id,
            &option
        );

        let result_state = match self
            .hummock_manager
            .trigger_manual_compaction(compaction_group_id, option)
            .await
        {
            Ok(_) => None,

            Err(error) => {
                return Err(tonic_err(error));
            }
        };

        let resp = TriggerManualCompactionResponse {
            status: result_state,
        };
        Ok(Response::new(resp))
    }

    async fn list_sstable_id_infos(
        &self,
        request: Request<ListSstableIdInfosRequest>,
    ) -> Result<Response<ListSstableIdInfosResponse>, Status> {
        let version_id = request.into_inner().version_id;
        let result = self
            .hummock_manager
            .list_sstable_id_infos(Some(version_id))
            .await;
        match result {
            Ok(sstable_id_infos) => Ok(Response::new(ListSstableIdInfosResponse {
                status: None,
                sstable_id_infos,
            })),
            Err(e) => Err(tonic_err(e)),
        }
    }
}
