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

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use itertools::Itertools;
use piestream_common::error::Result;
use piestream_hummock_sdk::HummockSSTableId;
use piestream_pb::hummock::VacuumTask;

use crate::hummock::model::INVALID_TIMESTAMP;
use crate::hummock::{CompactorManager, HummockManagerRef};
use crate::storage::MetaStore;

/// A SST's lifecycle is tracked in `HummockManager::Versioning` via `SstableIdInfo`:
/// - 1 A SST id is generated by meta node. Set `SstableIdInfo::id_create_timestamp`.
/// - 2 Corresponding SST file is created in object store.
/// - 3 The SST is tracked in meta node. Set `SstableIdInfo::meta_create_timestamp`.
/// - 4.1 The SST is compacted and vacuumed as tracked data. Set
///   `SstableIdInfo::meta_delete_timestamp` and delete asynchronously.
/// - 4.2 Or if step 3 didn't happen after some time, the SST is delete as orphan data
///   asynchronously.
pub struct VacuumTrigger<S: MetaStore> {
    hummock_manager: HummockManagerRef<S>,
    /// Use the CompactorManager to dispatch VacuumTask.
    compactor_manager: Arc<CompactorManager>,
    /// SST ids which have been dispatched to vacuum nodes but are not replied yet.
    pending_sst_ids: parking_lot::RwLock<HashSet<HummockSSTableId>>,
}

impl<S> VacuumTrigger<S>
where
    S: MetaStore,
{
    pub fn new(
        hummock_manager: HummockManagerRef<S>,
        compactor_manager: Arc<CompactorManager>,
    ) -> Self {
        Self {
            hummock_manager,
            compactor_manager,
            pending_sst_ids: Default::default(),
        }
    }

    /// Qualified versions' metadata are deleted and related stale SSTs are marked for deletion.
    /// Return number of deleted versions.
    /// A version can be deleted when:
    /// - It is not the greatest version. We never vacuum the greatest version.
    /// - And it's not being pinned, and we know it won't be pinned in the future because only
    ///   greatest version can be newly pinned.
    /// Besides, some of the stale SSTs in one version that are not referred by any other
    /// version(only older version) can also be deleted, even if the version itself is NOT
    /// qualified to be deleted. If a version is not pinned and all of its stale ssts are
    /// qualified to be deleted, then this version can be deleted.
    pub async fn vacuum_version_metadata(&self) -> piestream_common::error::Result<u64> {
        let batch_size = 16usize;
        let mut vacuum_count: usize = 0;
        let version_ids = self.hummock_manager.list_version_ids_asc().await?;
        if version_ids.is_empty() {
            return Ok(0);
        }
        let mut ssts_in_use = HashSet::new();
        let mut versions_to_delete = Vec::with_capacity(batch_size);
        // Iterate version ids in ascending order. Skip the greatest version id.
        for version_id in version_ids.iter().take(version_ids.len() - 1) {
            let pin_count = self
                .hummock_manager
                .get_version_pin_count(*version_id)
                .await?;
            if pin_count > 0 {
                // All the versions after this one must not be deleted.
                let sstable_infos = self
                    .hummock_manager
                    .list_sstable_id_infos(Some(*version_id))
                    .await?;
                ssts_in_use.extend(sstable_infos.iter().map(|info| info.id));
                continue;
            } else {
                self.hummock_manager
                    .delete_will_not_be_used_ssts(*version_id, &ssts_in_use)
                    .await?;
                let stale_ssts_left = self.hummock_manager.get_ssts_to_delete(*version_id).await?;
                if stale_ssts_left.is_empty() {
                    versions_to_delete.push(*version_id);
                }
                if versions_to_delete.len() >= batch_size {
                    vacuum_count += versions_to_delete.len();
                    self.hummock_manager
                        .delete_versions(&versions_to_delete)
                        .await?;
                    versions_to_delete.clear();
                }
            }
        }
        if !versions_to_delete.is_empty() {
            vacuum_count += versions_to_delete.len();
            self.hummock_manager
                .delete_versions(&versions_to_delete)
                .await?;
            versions_to_delete.clear();
        }
        Ok(vacuum_count as u64)
    }

    /// Qualified SSTs and their metadata(aka `SstableIdInfo`) are deleted.
    /// Return number of SSTs to delete.
    /// Two types of SSTs can be deleted:
    /// - Orphan SST. The SST is 1) not tracked in meta, that's to say `meta_create_timestamp` is
    ///   not set, 2) and the SST has existed longer than `ORPHAN_SST_RETENTION_INTERVAL` since
    ///   `id_create_timestamp`. Its `meta_delete_timestamp` field will then be set.
    /// - SST marked for deletion. The SST is marked for deletion by `vacuum_tracked_data`, that's
    ///   to say `meta_delete_timestamp` is set.
    pub async fn vacuum_sst_data(
        &self,
        orphan_sst_retention_interval: Duration,
    ) -> piestream_common::error::Result<Vec<HummockSSTableId>> {
        // Select SSTs to delete.
        let ssts_to_delete = {
            // 1. Retry the pending SSTs first.
            // It is possible some vacuum workers have been asked to vacuum these SSTs previously,
            // but they don't report the results yet due to either latency or failure.
            // This is OK since trying to delete the same SST multiple times is safe.
            let pending_sst_ids = self.pending_sst_ids.read().iter().cloned().collect_vec();
            if !pending_sst_ids.is_empty() {
                pending_sst_ids
            } else {
                // 2. If no pending SSTs, then fetch new ones.
                // Set orphan SSTs' meta_delete_timestamp field.
                self.hummock_manager
                    .mark_orphan_ssts(orphan_sst_retention_interval)
                    .await?;
                let ssts_to_delete = self
                    .hummock_manager
                    .list_sstable_id_infos(None)
                    .await?
                    .into_iter()
                    .filter(|sstable_id_info| {
                        sstable_id_info.meta_delete_timestamp != INVALID_TIMESTAMP
                    })
                    .map(|sstable_id_info| sstable_id_info.id)
                    .collect_vec();
                if ssts_to_delete.is_empty() {
                    return Ok(vec![]);
                }
                // Keep these SST ids, so that we can remove them from metadata later.
                self.pending_sst_ids.write().extend(ssts_to_delete.clone());
                ssts_to_delete
            }
        };

        let mut batch_idx = 0;
        let batch_size = 32usize;
        let mut sent_batch = Vec::with_capacity(ssts_to_delete.len());
        while batch_idx < ssts_to_delete.len() {
            let delete_batch = ssts_to_delete
                .iter()
                .skip(batch_idx)
                .take(batch_size)
                .cloned()
                .collect_vec();
            // 1. Pick a worker.
            let compactor = match self.compactor_manager.next_compactor() {
                None => {
                    tracing::warn!("No vacuum worker is available.");
                    break;
                }
                Some(compactor) => compactor,
            };

            // 2. Send task.
            match compactor
                .send_task(
                    None,
                    Some(VacuumTask {
                        // The SST id doesn't necessarily have a counterpart SST file in S3, but
                        // it's OK trying to delete it.
                        sstable_ids: delete_batch.clone(),
                    }),
                )
                .await
            {
                Ok(_) => {
                    tracing::debug!(
                        "Try to vacuum SSTs {:?} in worker {}.",
                        delete_batch,
                        compactor.context_id()
                    );
                    batch_idx += batch_size;
                    sent_batch.extend(delete_batch);
                }
                Err(err) => {
                    tracing::warn!(
                        "Failed to send vacuum task to worker {}: {:#?}",
                        compactor.context_id(),
                        err
                    );
                    self.compactor_manager
                        .remove_compactor(compactor.context_id());
                }
            }
        }
        Ok(sent_batch)
    }

    pub async fn report_vacuum_task(&self, vacuum_task: VacuumTask) -> Result<()> {
        let deleted_sst_ids = self
            .pending_sst_ids
            .read()
            .iter()
            .filter(|p| vacuum_task.sstable_ids.contains(p))
            .cloned()
            .collect_vec();
        if !deleted_sst_ids.is_empty() {
            self.hummock_manager
                .delete_sstable_ids(&deleted_sst_ids)
                .await?;
            self.pending_sst_ids
                .write()
                .retain(|p| !deleted_sst_ids.contains(p));
        }
        tracing::info!("Finish vacuuming SSTs {:?}", vacuum_task.sstable_ids);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use itertools::Itertools;
    use piestream_pb::hummock::VacuumTask;

    use crate::hummock::test_utils::{add_test_tables, setup_compute_env};
    use crate::hummock::{start_vacuum_scheduler, CompactorManager, VacuumTrigger};

    #[tokio::test]
    async fn test_shutdown_vacuum() {
        let (_env, hummock_manager, _cluster_manager, _worker_node) = setup_compute_env(80).await;
        let compactor_manager = Arc::new(CompactorManager::new());
        let vacuum = Arc::new(VacuumTrigger::new(hummock_manager, compactor_manager));
        let (join_handle, shutdown_sender) = start_vacuum_scheduler(vacuum);
        shutdown_sender.send(()).unwrap();
        join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_vacuum_version_metadata() {
        let (_env, hummock_manager, _cluster_manager, worker_node) = setup_compute_env(80).await;
        let context_id = worker_node.id;
        let compactor_manager = Arc::new(CompactorManager::default());
        let vacuum = Arc::new(VacuumTrigger::new(
            hummock_manager.clone(),
            compactor_manager.clone(),
        ));

        let pinned_version = hummock_manager
            .pin_version(context_id, u64::MAX)
            .await
            .unwrap();

        // Vacuum no version because the smallest v0 is pinned.
        assert_eq!(
            VacuumTrigger::vacuum_version_metadata(&vacuum)
                .await
                .unwrap(),
            0
        );
        hummock_manager
            .unpin_version(context_id, vec![pinned_version.id])
            .await
            .unwrap();

        add_test_tables(hummock_manager.as_ref(), context_id).await;
        // Current state: {v0: [], v1: [test_tables], v2: [test_tables_2, to_delete:test_tables],
        // v3: [test_tables_2, test_tables_3]}

        // Vacuum v0, v1, v2
        assert_eq!(
            VacuumTrigger::vacuum_version_metadata(&vacuum)
                .await
                .unwrap(),
            3
        );
    }

    #[tokio::test]
    async fn test_vacuum_orphan_sst_data() {
        let (_env, hummock_manager, _cluster_manager, _worker_node) = setup_compute_env(80).await;
        let compactor_manager = Arc::new(CompactorManager::default());
        let vacuum = VacuumTrigger::new(hummock_manager.clone(), compactor_manager.clone());
        // 1. acquire 2 SST ids.
        hummock_manager.get_new_table_id().await.unwrap();
        hummock_manager.get_new_table_id().await.unwrap();
        // 2. no expired SST id.
        assert_eq!(
            VacuumTrigger::vacuum_sst_data(&vacuum, Duration::from_secs(60),)
                .await
                .unwrap()
                .len(),
            0
        );
        // 3. 2 expired SST id but no vacuum node.
        assert_eq!(
            VacuumTrigger::vacuum_sst_data(&vacuum, Duration::from_secs(0),)
                .await
                .unwrap()
                .len(),
            0
        );
        let _receiver = compactor_manager.add_compactor(0);
        // 4. 2 expired SST ids.
        let sst_ids = VacuumTrigger::vacuum_sst_data(&vacuum, Duration::from_secs(0))
            .await
            .unwrap();
        assert_eq!(sst_ids.len(), 2);
        // 5. got the same 2 expired sst ids because the previous pending SST ids are not
        // reported.
        let sst_ids_2 = VacuumTrigger::vacuum_sst_data(&vacuum, Duration::from_secs(0))
            .await
            .unwrap();
        assert_eq!(sst_ids, sst_ids_2);
        // 6. report the previous pending SST ids to indicate their success.
        vacuum
            .report_vacuum_task(VacuumTask {
                sstable_ids: sst_ids_2,
            })
            .await
            .unwrap();
        assert_eq!(
            VacuumTrigger::vacuum_sst_data(&vacuum, Duration::from_secs(0),)
                .await
                .unwrap()
                .len(),
            0
        );
    }

    #[tokio::test]
    async fn test_vacuum_marked_for_deletion_sst_data() {
        let (_env, hummock_manager, _cluster_manager, worker_node) = setup_compute_env(80).await;
        let context_id = worker_node.id;
        let compactor_manager = Arc::new(CompactorManager::default());
        let vacuum = Arc::new(VacuumTrigger::new(
            hummock_manager.clone(),
            compactor_manager.clone(),
        ));
        let _receiver = compactor_manager.add_compactor(0);

        let sst_infos = add_test_tables(hummock_manager.as_ref(), context_id).await;
        // Current state: {v0: [], v1: [test_tables], v2: [test_tables_2, to_delete:test_tables],
        // v3: [test_tables_2, test_tables_3]}

        // Vacuum v0, v1, v2
        assert_eq!(
            VacuumTrigger::vacuum_version_metadata(&vacuum)
                .await
                .unwrap(),
            3
        );

        // Found test_table is marked for deletion.
        assert_eq!(
            VacuumTrigger::vacuum_sst_data(&vacuum, Duration::from_secs(600),)
                .await
                .unwrap()
                .len(),
            3
        );

        // The vacuum task is not reported yet.
        assert_eq!(
            VacuumTrigger::vacuum_sst_data(&vacuum, Duration::from_secs(600),)
                .await
                .unwrap()
                .len(),
            3
        );

        // The vacuum task is reported.
        vacuum
            .report_vacuum_task(VacuumTask {
                sstable_ids: sst_infos
                    .first()
                    .unwrap()
                    .iter()
                    .map(|s| s.id)
                    .collect_vec(),
            })
            .await
            .unwrap();

        // test_table is already reported.
        assert_eq!(
            VacuumTrigger::vacuum_sst_data(&vacuum, Duration::from_secs(600),)
                .await
                .unwrap()
                .len(),
            0
        );
    }
}
