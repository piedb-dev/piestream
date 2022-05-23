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

use std::collections::{BTreeMap, HashSet};
use std::future::Future;
use std::ops::DerefMut;
use std::sync::Arc;
use std::time::{Duration, Instant};

use itertools::Itertools;
use prost::Message;
use risingwave_common::util::compress::compress_data;
use risingwave_common::util::epoch::INVALID_EPOCH;
use risingwave_hummock_sdk::compact::compact_task_to_string;
use risingwave_hummock_sdk::compaction_group::CompactionGroupId;
use risingwave_hummock_sdk::{
    get_remote_sst_id, HummockContextId, HummockEpoch, HummockRefCount, HummockSSTableId,
    HummockVersionId,
};
use risingwave_pb::common::ParallelUnitMapping;
use risingwave_pb::hummock::{
    CompactTask, CompactTaskAssignment, HummockPinnedSnapshot, HummockPinnedVersion,
    HummockSnapshot, HummockStaleSstables, HummockVersion, Level, LevelType, SstableIdInfo,
    SstableInfo, UncommittedEpoch,
};
use risingwave_pb::meta::subscribe_response::{Info, Operation};
use tokio::sync::RwLock;

use crate::cluster::{ClusterManagerRef, META_NODE_ID};
use crate::hummock::compaction::{CompactStatus, CompactionConfig};
use crate::hummock::compaction_scheduler::CompactionRequestChannelRef;
use crate::hummock::error::{Error, Result};
use crate::hummock::metrics_utils::{trigger_commit_stat, trigger_rw_stat, trigger_sst_stat};
use crate::hummock::model::{
    sstable_id_info, CurrentHummockVersionId, HummockPinnedSnapshotExt, HummockPinnedVersionExt,
    INVALID_TIMESTAMP,
};
use crate::manager::{IdCategory, MetaSrvEnv};
use crate::model::{MetadataModel, ValTransaction, VarTransaction, Worker};
use crate::rpc::metrics::MetaMetrics;
use crate::storage::{MetaStore, Transaction};

// Update to states are performed as follow:
// - Initialize ValTransaction for the meta state to update
// - Make changes on the ValTransaction.
// - Call `commit_multi_var` to commit the changes via meta store transaction. If transaction
//   succeeds, the in-mem state will be updated by the way.
// - Call `abort_multi_var` if the ValTransaction is no longer needed
pub struct HummockManager<S: MetaStore> {
    env: MetaSrvEnv<S>,
    cluster_manager: ClusterManagerRef<S>,
    // When trying to locks compaction and versioning at the same time, compaction lock should
    // be requested before versioning lock.
    compaction: RwLock<Compaction>,
    versioning: RwLock<Versioning>,

    metrics: Arc<MetaMetrics>,

    /// `compaction_scheduler` is used to schedule a compaction for specified CompactionGroupId
    compaction_scheduler: parking_lot::RwLock<Option<CompactionRequestChannelRef>>,
    config: Arc<CompactionConfig>,
}

pub type HummockManagerRef<S> = Arc<HummockManager<S>>;

struct Compaction {
    compact_status: CompactStatus,
    compact_task_assignment: BTreeMap<u64, CompactTaskAssignment>,
}

/// Commit multiple `ValTransaction`s to state store and upon success update the local in-mem state
/// by the way
/// After called, the `ValTransaction` will be dropped.
macro_rules! commit_multi_var {
    ($hummock_mgr:expr, $context_id:expr, $($val_txn:expr),*) => {
        {
            async {
                let mut trx = Transaction::default();
                // Apply the change in `ValTransaction` to trx
                $(
                    $val_txn.apply_to_txn(&mut trx)?;
                )*
                // Commit to state store
                $hummock_mgr.commit_trx($hummock_mgr.env.meta_store(), trx, $context_id)
                .await?;
                // Upon successful commit, commit the change to local in-mem state
                $(
                    $val_txn.commit();
                )*
                Result::Ok(())
            }.await
        }
    };
}

/// Abort the `ValTransaction`s. Simply drop them
macro_rules! abort_multi_var {
    ($($val_txn:expr),*) => {
        $(
            $val_txn.abort();
        )*
    }
}

struct Versioning {
    current_version_id: CurrentHummockVersionId,
    hummock_versions: BTreeMap<HummockVersionId, HummockVersion>,
    pinned_versions: BTreeMap<HummockContextId, HummockPinnedVersion>,
    pinned_snapshots: BTreeMap<HummockContextId, HummockPinnedSnapshot>,
    stale_sstables: BTreeMap<HummockVersionId, HummockStaleSstables>,
    sstable_id_infos: BTreeMap<HummockSSTableId, SstableIdInfo>,
}

impl Versioning {
    pub fn current_version_ref(&self) -> &HummockVersion {
        self.hummock_versions
            .get(&self.current_version_id.id())
            .expect("current version should always be available.")
    }

    pub fn current_version(&self) -> HummockVersion {
        self.current_version_ref().clone()
    }
}

impl<S> HummockManager<S>
where
    S: MetaStore,
{
    pub async fn new(
        env: MetaSrvEnv<S>,
        cluster_manager: ClusterManagerRef<S>,
        metrics: Arc<MetaMetrics>,
    ) -> Result<HummockManager<S>> {
        let instance = HummockManager {
            env,
            versioning: RwLock::new(Versioning {
                current_version_id: CurrentHummockVersionId::new(),
                hummock_versions: Default::default(),
                pinned_versions: Default::default(),
                pinned_snapshots: Default::default(),
                stale_sstables: Default::default(),
                sstable_id_infos: Default::default(),
            }),
            compaction: RwLock::new(Compaction {
                compact_status: CompactStatus::default(),
                compact_task_assignment: Default::default(),
            }),
            metrics,
            cluster_manager,
            compaction_scheduler: parking_lot::RwLock::new(None),
            // TODO: load it from etcd or configuration file.
            config: Arc::new(CompactionConfig::default()),
        };

        instance.load_meta_store_state().await?;
        instance.release_invalid_contexts().await?;
        instance.cancel_unassigned_compaction_task().await?;
        // Release snapshots pinned by meta on restarting.
        instance.release_contexts([META_NODE_ID]).await?;

        Ok(instance)
    }

    /// Load state from meta store.
    async fn load_meta_store_state(&self) -> Result<()> {
        let config = self.config.clone();
        let mut compaction_guard = self.compaction.write().await;

        compaction_guard.compact_status = CompactStatus::get(self.env.meta_store())
            .await?
            .unwrap_or_else(|| CompactStatus::new(config));

        compaction_guard.compact_task_assignment =
            CompactTaskAssignment::list(self.env.meta_store())
                .await?
                .into_iter()
                .map(|assigned| (assigned.key().unwrap().id, assigned))
                .collect();

        let mut versioning_guard = self.versioning.write().await;
        versioning_guard.current_version_id = CurrentHummockVersionId::get(self.env.meta_store())
            .await?
            .unwrap_or_else(CurrentHummockVersionId::new);

        versioning_guard.hummock_versions = HummockVersion::list(self.env.meta_store())
            .await?
            .into_iter()
            .map(|version| (version.id, version))
            .collect();

        // Insert the initial version.
        if versioning_guard.hummock_versions.is_empty() {
            let mut init_version = HummockVersion {
                id: versioning_guard.current_version_id.id(),
                levels: vec![Level {
                    level_idx: 0,
                    level_type: LevelType::Overlapping as i32,
                    table_infos: vec![],
                }],
                uncommitted_epochs: vec![],
                max_committed_epoch: INVALID_EPOCH,
                safe_epoch: INVALID_EPOCH,
            };
            for l in 0..self.config.max_level {
                init_version.levels.push(Level {
                    level_idx: (l + 1) as u32,
                    level_type: LevelType::Nonoverlapping as i32,
                    table_infos: vec![],
                });
            }
            init_version.insert(self.env.meta_store()).await?;
            versioning_guard
                .hummock_versions
                .insert(init_version.id, init_version);
        }

        versioning_guard.pinned_versions = HummockPinnedVersion::list(self.env.meta_store())
            .await?
            .into_iter()
            .map(|p| (p.context_id, p))
            .collect();
        versioning_guard.pinned_snapshots = HummockPinnedSnapshot::list(self.env.meta_store())
            .await?
            .into_iter()
            .map(|p| (p.context_id, p))
            .collect();

        versioning_guard.stale_sstables = HummockStaleSstables::list(self.env.meta_store())
            .await?
            .into_iter()
            .map(|s| (s.version_id, s))
            .collect();

        versioning_guard.sstable_id_infos = SstableIdInfo::list(self.env.meta_store())
            .await?
            .into_iter()
            .map(|s| (s.id, s))
            .collect();

        Ok(())
    }

    /// We use worker node id as the `context_id`.
    /// If the `context_id` is provided, the transaction will abort if the `context_id` is not
    /// valid, which means the worker node is not a valid member of the cluster.
    async fn commit_trx(
        &self,
        meta_store: &S,
        mut trx: Transaction,
        context_id: Option<HummockContextId>,
    ) -> Result<()> {
        if let Some(context_id) = context_id {
            if context_id == META_NODE_ID {
                // Using the preserved meta id is allowed.
            } else if let Some(worker) = self.cluster_manager.get_worker_by_id(context_id).await {
                trx.check_exists(Worker::cf_name(), worker.key()?.encode_to_vec());
            } else {
                // The worker is not found in cluster.
                return Err(Error::InvalidContext(context_id));
            }
        }
        meta_store.txn(trx).await.map_err(Into::into)
    }

    /// Pin a hummock version that is greater than `last_pinned`. The pin belongs to `context_id`
    /// and will be unpinned when `context_id` is invalidated.
    /// `last_pinned` helps to make `pin_version` retryable:
    /// 1 Return the smallest already pinned version of `context_id` that is greater than
    /// `last_pinned`, if any.
    /// 2 Otherwise pin and return the current greatest version.
    pub async fn pin_version(
        &self,
        context_id: HummockContextId,
        last_pinned: HummockVersionId,
    ) -> Result<HummockVersion> {
        let mut versioning_guard = self.versioning.write().await;
        let versioning = versioning_guard.deref_mut();
        let mut pinned_versions = VarTransaction::new(&mut versioning.pinned_versions);
        let hummock_versions = &versioning.hummock_versions;
        let current_version_id = versioning.current_version_id.clone();
        let mut context_pinned_version = pinned_versions.new_entry_txn_or_default(
            context_id,
            HummockPinnedVersion {
                context_id,
                version_id: vec![],
            },
        );

        let mut already_pinned = false;
        let version_id = {
            let partition_point = context_pinned_version
                .version_id
                .iter()
                .sorted()
                .cloned()
                .collect_vec()
                .partition_point(|p| *p <= last_pinned);
            if partition_point < context_pinned_version.version_id.len() {
                already_pinned = true;
                context_pinned_version.version_id[partition_point]
            } else {
                current_version_id.id()
            }
        };

        if !already_pinned {
            context_pinned_version.pin_version(version_id);
            commit_multi_var!(self, Some(context_id), context_pinned_version)?;
        } else {
            abort_multi_var!(context_pinned_version);
        }

        let ret = Ok(hummock_versions.get(&version_id).unwrap().clone());

        #[cfg(test)]
        {
            drop(versioning_guard);
            self.check_state_consistency().await;
        }

        ret
    }

    pub async fn unpin_version(
        &self,
        context_id: HummockContextId,
        pinned_version_ids: impl AsRef<[HummockVersionId]>,
    ) -> Result<()> {
        let mut versioning_guard = self.versioning.write().await;
        let mut pinned_versions = VarTransaction::new(&mut versioning_guard.pinned_versions);
        let mut context_pinned_version = match pinned_versions.new_entry_txn(context_id) {
            None => {
                return Ok(());
            }
            Some(context_pinned_version) => context_pinned_version,
        };
        for pinned_version_id in pinned_version_ids.as_ref() {
            context_pinned_version.unpin_version(*pinned_version_id);
        }
        commit_multi_var!(self, Some(context_id), context_pinned_version)?;

        #[cfg(test)]
        {
            drop(versioning_guard);
            self.check_state_consistency().await;
        }

        Ok(())
    }

    pub async fn add_tables(
        &self,
        context_id: HummockContextId,
        sstables: Vec<SstableInfo>,
        epoch: HummockEpoch,
    ) -> Result<HummockVersion> {
        let mut versioning_guard = self.versioning.write().await;

        let versioning = versioning_guard.deref_mut();
        let mut current_version_id = VarTransaction::new(&mut versioning.current_version_id);
        let mut hummock_versions = VarTransaction::new(&mut versioning.hummock_versions);
        let mut sstable_id_infos = VarTransaction::new(&mut versioning.sstable_id_infos);

        let current_hummock_version = hummock_versions
            .get(&current_version_id.id())
            .unwrap()
            .clone();

        // Track SST in meta.
        for sst_id in sstables.iter().map(|s| s.id) {
            match sstable_id_infos.get_mut(&sst_id) {
                None => {
                    return Err(Error::InternalError(format!(
                        "Invalid SST id {}, may have been vacuumed",
                        sst_id
                    )));
                }
                Some(sst_id_info) => {
                    if sst_id_info.meta_delete_timestamp != INVALID_TIMESTAMP {
                        return Err(Error::InternalError(format!(
                            "SST id {} has been marked for vacuum",
                            sst_id
                        )));
                    }
                    if sst_id_info.meta_create_timestamp != INVALID_TIMESTAMP {
                        // This is a duplicate request.
                        return Ok(current_hummock_version);
                    }
                    sst_id_info.meta_create_timestamp = sstable_id_info::get_timestamp_now();
                }
            }
        }

        // Check whether the epoch is valid
        // TODO: return error instead of panic
        if epoch <= current_hummock_version.max_committed_epoch {
            panic!(
                "Epoch {} <= max_committed_epoch {}",
                epoch, current_hummock_version.max_committed_epoch
            );
        }

        current_version_id.increase();
        let mut new_hummock_version = hummock_versions
            .new_entry_txn_or_default(current_version_id.id(), current_hummock_version);
        new_hummock_version.id = current_version_id.id();

        // Create new_version by adding tables in UncommittedEpoch
        match new_hummock_version
            .uncommitted_epochs
            .iter_mut()
            .find(|e| e.epoch == epoch)
        {
            Some(uncommitted_epoch) => {
                uncommitted_epoch.tables.extend(sstables);
            }
            None => new_hummock_version
                .uncommitted_epochs
                .push(UncommittedEpoch {
                    epoch,
                    tables: sstables,
                }),
        };

        let ret_hummock_version = new_hummock_version.clone();

        commit_multi_var!(
            self,
            Some(context_id),
            current_version_id,
            sstable_id_infos,
            new_hummock_version
        )?;

        // Update metrics
        trigger_commit_stat(&self.metrics, &ret_hummock_version);

        #[cfg(test)]
        {
            drop(versioning_guard);
            self.check_state_consistency().await;
        }

        Ok(ret_hummock_version)
    }

    /// Make sure `max_commited_epoch` is pinned and return it.
    /// Assume that frontend will only pass the latest epoch value recorded by frontend to
    /// `last_pinned`. Meta will unpin snapshots which are pinned and in (`last_pinned`,
    /// `max_commited_epoch`).
    pub async fn pin_snapshot(
        &self,
        context_id: HummockContextId,
        last_pinned: HummockEpoch,
    ) -> Result<HummockSnapshot> {
        let mut versioning_guard = self.versioning.write().await;

        // Use the max_committed_epoch in storage as the snapshot ts so only committed changes are
        // visible in the snapshot.
        let version_id = versioning_guard.current_version_id.id();

        let max_committed_epoch = versioning_guard
            .hummock_versions
            .get(&version_id)
            .unwrap()
            .max_committed_epoch;

        let mut pinned_snapshots = VarTransaction::new(&mut versioning_guard.pinned_snapshots);
        let mut context_pinned_snapshot = pinned_snapshots.new_entry_txn_or_default(
            context_id,
            HummockPinnedSnapshot {
                context_id,
                snapshot_id: vec![],
            },
        );

        // Unpin the snapshots pinned by meta but frontend doesn't know.
        let to_unpin = context_pinned_snapshot
            .snapshot_id
            .iter()
            .filter(|e| **e > last_pinned && **e < max_committed_epoch)
            .cloned()
            .collect_vec();
        let mut snapshots_change = !to_unpin.is_empty();
        for epoch in to_unpin {
            context_pinned_snapshot.unpin_snapshot(epoch);
        }

        if !context_pinned_snapshot
            .snapshot_id
            .contains(&max_committed_epoch)
        {
            snapshots_change = true;
            context_pinned_snapshot.pin_snapshot(max_committed_epoch);
        }

        if snapshots_change {
            commit_multi_var!(self, Some(context_id), context_pinned_snapshot)?;
        } else {
            abort_multi_var!(context_pinned_snapshot);
        }

        #[cfg(test)]
        {
            drop(versioning_guard);
            self.check_state_consistency().await;
        }

        Ok(HummockSnapshot {
            epoch: max_committed_epoch,
        })
    }

    pub async fn unpin_snapshot(
        &self,
        context_id: HummockContextId,
        hummock_snapshots: impl AsRef<[HummockSnapshot]>,
    ) -> Result<()> {
        let mut versioning_guard = self.versioning.write().await;
        let mut pinned_snapshots = VarTransaction::new(&mut versioning_guard.pinned_snapshots);

        let mut context_pinned_snapshot = match pinned_snapshots.new_entry_txn(context_id) {
            None => {
                return Ok(());
            }
            Some(context_pinned_snapshot) => context_pinned_snapshot,
        };
        for hummock_snapshot in hummock_snapshots.as_ref() {
            context_pinned_snapshot.unpin_snapshot(hummock_snapshot.epoch);
        }
        commit_multi_var!(self, Some(context_id), context_pinned_snapshot)?;

        #[cfg(test)]
        {
            drop(versioning_guard);
            self.check_state_consistency().await;
        }

        Ok(())
    }

    pub async fn get_compact_task(&self) -> Result<Option<CompactTask>> {
        let start_time = Instant::now();
        let mut compaction_guard = self.compaction.write().await;

        let compaction = compaction_guard.deref_mut();
        let mut compact_status = VarTransaction::new(&mut compaction.compact_status);
        let current_version = self.versioning.read().await.current_version();
        let compact_task = compact_status.get_compact_task(&current_version.levels);
        let ret = match compact_task {
            None => Ok(None),
            Some(mut compact_task) => {
                compact_task.watermark = {
                    let versioning_guard = self.versioning.read().await;
                    let current_version_id = versioning_guard.current_version_id.id();
                    let max_committed_epoch = versioning_guard
                        .hummock_versions
                        .get(&current_version_id)
                        .unwrap()
                        .max_committed_epoch;
                    versioning_guard
                        .pinned_snapshots
                        .values()
                        .flat_map(|v| v.snapshot_id.clone())
                        .fold(max_committed_epoch, std::cmp::min)
                };
                let table_ids = compact_task
                    .input_ssts
                    .iter()
                    .flat_map(|level| {
                        level
                            .table_infos
                            .iter()
                            .flat_map(|sst_info| {
                                sst_info.vnode_bitmaps.iter().map(|bitmap| bitmap.table_id)
                            })
                            .collect_vec()
                    })
                    .collect::<HashSet<u32>>();
                compact_task.vnode_mappings.reserve_exact(table_ids.len());
                for table_id in table_ids {
                    if let Some(vnode_mapping) = self
                        .env
                        .hash_mapping_manager()
                        .get_table_hash_mapping(&table_id)
                    {
                        let (original_indices, compressed_data) = compress_data(&vnode_mapping);
                        let compressed_mapping = ParallelUnitMapping {
                            table_id,
                            original_indices,
                            data: compressed_data,
                        };
                        compact_task.vnode_mappings.push(compressed_mapping);
                    }
                }

                commit_multi_var!(self, None, compact_status)?;
                tracing::trace!(
                    "pick up {} tables in level {} to compact, The number of total tables is {}. cost time: {:?}",
                    compact_task.input_ssts[0].table_infos.len(),
                    compact_task.input_ssts[0].level_idx,
                    current_version.levels[compact_task.input_ssts[0].level_idx as usize]
                        .table_infos
                        .len(),
                    start_time.elapsed()
                );
                Ok(Some(compact_task))
            }
        };

        #[cfg(test)]
        {
            drop(compaction_guard);
            self.check_state_consistency().await;
        }

        ret
    }

    /// Assigns a compaction task to a compactor
    pub async fn assign_compaction_task<T: Future<Output = bool>>(
        &self,
        compact_task: &CompactTask,
        assignee_context_id: HummockContextId,
        send_task: T,
    ) -> Result<()> {
        let mut compaction_guard = self.compaction.write().await;
        if !send_task.await {
            return Err(Error::CompactorUnreachable(assignee_context_id));
        }

        let compaction = compaction_guard.deref_mut();
        let mut compact_task_assignment =
            VarTransaction::new(&mut compaction.compact_task_assignment);
        if let Some(assignment) = compact_task_assignment.get(&compact_task.task_id) {
            return Err(Error::CompactionTaskAlreadyAssigned(
                compact_task.task_id,
                assignment.context_id,
            ));
        }
        compact_task_assignment.insert(
            compact_task.task_id,
            CompactTaskAssignment {
                compact_task: Some(compact_task.clone()),
                context_id: assignee_context_id,
            },
        );
        commit_multi_var!(self, Some(assignee_context_id), compact_task_assignment)?;

        #[cfg(test)]
        {
            drop(compaction_guard);
            self.check_state_consistency().await;
        }

        Ok(())
    }

    /// `report_compact_task` is retryable. `task_id` in `compact_task` parameter is used as the
    /// idempotency key. Return Ok(false) to indicate the `task_id` is not found, which may have
    /// been processed previously.
    pub async fn report_compact_task(&self, compact_task: &CompactTask) -> Result<bool> {
        let mut compaction_guard = self.compaction.write().await;
        let start_time = Instant::now();
        let compaction = compaction_guard.deref_mut();
        let mut compact_status = VarTransaction::new(&mut compaction.compact_status);
        let mut compact_task_assignment =
            VarTransaction::new(&mut compaction.compact_task_assignment);
        let assignee_context_id = match compact_task_assignment.remove(&compact_task.task_id) {
            None => {
                // The task is not found.
                return Ok(false);
            }
            Some(assignment) => assignment.context_id,
        };
        compact_status.report_compact_task(compact_task);
        if compact_task.task_status {
            // The compaction task is finished.
            let mut versioning_guard = self.versioning.write().await;
            let old_version = versioning_guard.current_version();
            let versioning = versioning_guard.deref_mut();
            let mut current_version_id = VarTransaction::new(&mut versioning.current_version_id);
            let mut hummock_versions = VarTransaction::new(&mut versioning.hummock_versions);
            let mut stale_sstables = VarTransaction::new(&mut versioning.stale_sstables);
            let mut sstable_id_infos = VarTransaction::new(&mut versioning.sstable_id_infos);
            let mut version_stale_sstables = stale_sstables.new_entry_txn_or_default(
                old_version.id,
                HummockStaleSstables {
                    version_id: old_version.id,
                    id: vec![],
                },
            );
            for level in &compact_task.input_ssts {
                version_stale_sstables
                    .id
                    .extend(level.table_infos.iter().map(|sst| sst.id).collect_vec());
            }
            let mut new_version = CompactStatus::apply_compact_result(compact_task, old_version);
            current_version_id.increase();
            new_version.id = current_version_id.id();
            hummock_versions.insert(new_version.id, new_version);

            for SstableInfo { id: ref sst_id, .. } in &compact_task.sorted_output_ssts {
                match sstable_id_infos.get_mut(sst_id) {
                    None => {
                        return Err(Error::InternalError(format!(
                            "invalid sst id {}, may have been vacuumed",
                            sst_id
                        )));
                    }
                    Some(mut sst_id_info) => {
                        sst_id_info.meta_create_timestamp = sstable_id_info::get_timestamp_now();
                    }
                }
            }

            commit_multi_var!(
                self,
                Some(assignee_context_id),
                compact_status,
                compact_task_assignment,
                current_version_id,
                hummock_versions,
                version_stale_sstables,
                sstable_id_infos
            )?;
        } else {
            // The compaction task is cancelled.
            commit_multi_var!(
                self,
                Some(assignee_context_id),
                compact_status,
                compact_task_assignment
            )?;
        }

        tracing::trace!(
            "Reported compaction task. {}. cost time: {:?}",
            compact_task_to_string(compact_task),
            start_time.elapsed(),
        );

        trigger_sst_stat(
            &self.metrics,
            &compaction_guard.compact_status,
            self.versioning.read().await.current_version_ref(),
        );
        if let Some(ref compact_task_metrics) = compact_task.metrics {
            trigger_rw_stat(&self.metrics, compact_task_metrics);
        }

        // TODO: use correct id after compaction group is supported.
        self.try_send_compaction_request(0.into());

        #[cfg(test)]
        {
            drop(compaction_guard);
            self.check_state_consistency().await;
        }

        Ok(true)
    }

    pub async fn commit_epoch(&self, epoch: HummockEpoch) -> Result<()> {
        let mut versioning_guard = self.versioning.write().await;
        let old_version = versioning_guard.current_version();
        let versioning = versioning_guard.deref_mut();
        let mut current_version_id = VarTransaction::new(&mut versioning.current_version_id);
        let mut hummock_versions = VarTransaction::new(&mut versioning.hummock_versions);
        current_version_id.increase();
        let mut new_hummock_version =
            hummock_versions.new_entry_txn_or_default(current_version_id.id(), old_version);
        new_hummock_version.id = current_version_id.id();
        // TODO: return error instead of panic
        if epoch <= new_hummock_version.max_committed_epoch {
            panic!(
                "Epoch {} <= max_committed_epoch {}",
                epoch, new_hummock_version.max_committed_epoch
            );
        }

        // TODO #447: the epoch should fail and rollback if any precedent epoch is uncommitted.
        // get tables in the committing epoch
        if let Some(idx) = new_hummock_version
            .uncommitted_epochs
            .iter()
            .position(|e| e.epoch == epoch)
        {
            // deref mut here so that we can mutably borrow different fields in new_hummock_version
            let new_hummock_version = new_hummock_version.deref_mut();
            let uncommitted_epoch = &new_hummock_version.uncommitted_epochs[idx];

            // Commit tables by moving them into level0
            let version_first_level = new_hummock_version.levels.first_mut().unwrap();
            if version_first_level.level_idx == 0 {
                uncommitted_epoch
                    .tables
                    .iter()
                    .for_each(|t| version_first_level.table_infos.push(t.clone()));
            } else {
                return Err(Error::InternalError(
                    "unsupported LevelType::Nonoverlapping".to_owned(),
                ));
            }
            // Remove the epoch from uncommitted_epochs
            new_hummock_version.uncommitted_epochs.swap_remove(idx);
        }
        // Create a new_version, possibly merely to bump up the version id and max_committed_epoch.
        new_hummock_version.max_committed_epoch = epoch;
        commit_multi_var!(self, None, new_hummock_version, current_version_id)?;

        // Update metrics
        trigger_commit_stat(&self.metrics, versioning.current_version_ref());

        tracing::trace!("new committed epoch {}", epoch);

        self.env
            .notification_manager()
            .notify_frontend_asynchronously(
                Operation::Update, // Frontends don't care about operation.
                Info::HummockSnapshot(HummockSnapshot { epoch }),
            );

        // TODO: use correct id after compaction group is supported.
        self.try_send_compaction_request(0.into());

        #[cfg(test)]
        {
            drop(versioning_guard);
            self.check_state_consistency().await;
        }

        Ok(())
    }

    pub async fn abort_epoch(&self, epoch: HummockEpoch) -> Result<()> {
        let mut versioning_guard = self.versioning.write().await;
        let old_version = versioning_guard.current_version();
        let versioning = versioning_guard.deref_mut();
        let mut current_version_id = VarTransaction::new(&mut versioning.current_version_id);
        let mut hummock_versions = VarTransaction::new(&mut versioning.hummock_versions);
        let mut stale_sstables = VarTransaction::new(&mut versioning.stale_sstables);
        let old_version_id = old_version.id;
        current_version_id.increase();
        let mut new_hummock_version =
            hummock_versions.new_entry_txn_or_default(current_version_id.id(), old_version);
        new_hummock_version.id = current_version_id.id();

        // Get and remove tables in the committing epoch
        let uncommitted_epoch = new_hummock_version
            .uncommitted_epochs
            .iter()
            .position(|e| e.epoch == epoch)
            .map(|idx| new_hummock_version.uncommitted_epochs.swap_remove(idx));

        if let Some(epoch_info) = uncommitted_epoch {
            // Remove tables of the aborting epoch
            let mut version_stale_sstables = stale_sstables.new_entry_txn_or_default(
                old_version_id,
                HummockStaleSstables {
                    version_id: old_version_id,
                    id: vec![],
                },
            );
            version_stale_sstables
                .id
                .extend(epoch_info.tables.iter().map(|t| t.id));

            commit_multi_var!(
                self,
                None,
                current_version_id,
                new_hummock_version,
                version_stale_sstables
            )?;
        } else {
            abort_multi_var!(current_version_id, new_hummock_version, stale_sstables);
        }

        #[cfg(test)]
        {
            drop(versioning_guard);
            self.check_state_consistency().await;
        }

        Ok(())
    }

    pub async fn get_new_table_id(&self) -> Result<HummockSSTableId> {
        // TODO id_gen_manager generates u32, we need u64
        let sstable_id = get_remote_sst_id(
            self.env
                .id_gen_manager()
                .generate::<{ IdCategory::HummockSSTableId }>()
                .await
                .map(|id| id as HummockSSTableId)?,
        );

        let mut versioning_guard = self.versioning.write().await;
        let new_sst_id_info = SstableIdInfo {
            id: sstable_id,
            id_create_timestamp: sstable_id_info::get_timestamp_now(),
            meta_create_timestamp: INVALID_TIMESTAMP,
            meta_delete_timestamp: INVALID_TIMESTAMP,
        };
        new_sst_id_info.insert(self.env.meta_store()).await?;

        // Update in-mem state after transaction succeeds.
        versioning_guard
            .sstable_id_infos
            .insert(new_sst_id_info.id, new_sst_id_info);

        #[cfg(test)]
        {
            drop(versioning_guard);
            self.check_state_consistency().await;
        }

        Ok(sstable_id)
    }

    /// Release resources pinned by these contexts, including:
    /// - Version
    /// - Snapshot
    /// - Compaction task
    pub async fn release_contexts(
        &self,
        context_ids: impl AsRef<[HummockContextId]>,
    ) -> Result<()> {
        let mut compaction_guard = self.compaction.write().await;
        let compaction = compaction_guard.deref_mut();
        let mut compact_status = VarTransaction::new(&mut compaction.compact_status);
        let mut compact_task_assignment =
            VarTransaction::new(&mut compaction.compact_task_assignment);
        let mut versioning_guard = self.versioning.write().await;
        let versioning = versioning_guard.deref_mut();
        let mut pinned_versions = VarTransaction::new(&mut versioning.pinned_versions);
        let mut pinned_snapshots = VarTransaction::new(&mut versioning.pinned_snapshots);

        let mut to_commit = false;
        for context_id in context_ids.as_ref() {
            tracing::debug!("Release context {}", *context_id);
            for assignment in compact_task_assignment.values() {
                if assignment.context_id == *context_id {
                    compact_status.report_compact_task(
                        assignment
                            .compact_task
                            .as_ref()
                            .expect("compact_task shouldn't be None"),
                    );
                }
            }
            compact_task_assignment.retain(|_, v| v.context_id != *context_id);
            to_commit = pinned_versions.remove(context_id).is_some() || to_commit;
            to_commit = pinned_snapshots.remove(context_id).is_some() || to_commit;
        }
        if !to_commit {
            return Ok(());
        }

        if to_commit {
            commit_multi_var!(
                self,
                None,
                compact_status,
                compact_task_assignment,
                pinned_versions,
                pinned_snapshots
            )?;
        } else {
            abort_multi_var!(
                compact_status,
                compact_task_assignment,
                pinned_versions,
                pinned_snapshots
            );
        }

        #[cfg(test)]
        {
            drop(versioning_guard);
            drop(compaction_guard);
            self.check_state_consistency().await;
        }

        Ok(())
    }

    /// List version ids in ascending order.
    pub async fn list_version_ids_asc(&self) -> Result<Vec<HummockVersionId>> {
        let versioning_guard = self.versioning.read().await;
        let version_ids = versioning_guard
            .hummock_versions
            .keys()
            .cloned()
            .collect_vec();
        Ok(version_ids)
    }

    /// Get the reference count of given version id
    pub async fn get_version_pin_count(
        &self,
        version_id: HummockVersionId,
    ) -> Result<HummockRefCount> {
        let versioning_guard = self.versioning.read().await;
        let count = versioning_guard
            .pinned_versions
            .values()
            .filter(|version_pin| version_pin.version_id.contains(&version_id))
            .count();
        Ok(count as HummockRefCount)
    }

    pub async fn get_ssts_to_delete(
        &self,
        version_id: HummockVersionId,
    ) -> Result<Vec<HummockSSTableId>> {
        let versioning_guard = self.versioning.read().await;
        Ok(versioning_guard
            .stale_sstables
            .get(&version_id)
            .map(|s| s.id.clone())
            .unwrap_or_default())
    }

    pub async fn delete_will_not_be_used_ssts(
        &self,
        version_id: HummockVersionId,
        ssts_in_use: &HashSet<HummockSSTableId>,
    ) -> Result<()> {
        let mut versioning_guard = self.versioning.write().await;
        let versioning = versioning_guard.deref_mut();
        let mut stale_sstables = VarTransaction::new(&mut versioning.stale_sstables);
        let mut sstable_id_infos = VarTransaction::new(&mut versioning.sstable_id_infos);
        if let Some(ssts_to_delete) = stale_sstables.get_mut(&version_id) {
            // Delete sstables that are stale in the view of `version_id` Version, and
            // are Not referred by any other older Version.
            // No newer version would use any stale sstables in the current Version.
            let num_ssts_to_delete = ssts_to_delete.id.len();
            for idx in (0..num_ssts_to_delete).rev() {
                let sst_id = ssts_to_delete.id[idx];
                if !ssts_in_use.contains(&sst_id) && let Some(mut sst_id_info) = sstable_id_infos.get_mut(&sst_id) {
                    sst_id_info.meta_delete_timestamp = sstable_id_info::get_timestamp_now();
                    // We don't want to repetitively set the delete timestamp of these that have been set,
                    // so we remove these ones.
                    ssts_to_delete.id.swap_remove(idx);
                }
            }
        }

        commit_multi_var!(self, None, stale_sstables, sstable_id_infos)?;

        #[cfg(test)]
        {
            drop(versioning_guard);
            self.check_state_consistency().await;
        }

        Ok(())
    }

    /// Delete metadata of the given `version_ids`
    pub async fn delete_versions(&self, version_ids: &[HummockVersionId]) -> Result<()> {
        let mut versioning_guard = self.versioning.write().await;
        let versioning = versioning_guard.deref_mut();
        let pinned_versions_ref = &versioning.pinned_versions;
        let mut hummock_versions = VarTransaction::new(&mut versioning.hummock_versions);
        let mut stale_sstables = VarTransaction::new(&mut versioning.stale_sstables);
        for version_id in version_ids {
            if hummock_versions.remove(version_id).is_none() {
                continue;
            }
            if let Some(ssts_to_delete) = stale_sstables.get_mut(version_id) {
                if !ssts_to_delete.id.is_empty() {
                    return Err(Error::InternalError(format!(
                        "Version {} still has stale ssts undeleted:{:?}",
                        version_id, ssts_to_delete.id
                    )));
                }
            }
            stale_sstables.remove(version_id);

            for version_pin in pinned_versions_ref.values() {
                assert!(
                    !version_pin.version_id.contains(version_id),
                    "version still referenced shouldn't be deleted."
                );
            }
        }
        commit_multi_var!(self, None, hummock_versions, stale_sstables)?;

        #[cfg(test)]
        {
            drop(versioning_guard);
            self.check_state_consistency().await;
        }

        Ok(())
    }

    // TODO: use proc macro to call check_state_consistency
    #[cfg(test)]
    pub async fn check_state_consistency(&self) {
        let get_state = || async {
            let compaction_guard = self.compaction.read().await;
            let versioning_guard = self.versioning.read().await;
            let compact_status_copy = compaction_guard.compact_status.clone();
            let compact_task_assignment_copy = compaction_guard.compact_task_assignment.clone();
            let current_version_id_copy = versioning_guard.current_version_id.clone();
            let hummmock_versions_copy = versioning_guard.hummock_versions.clone();
            let pinned_versions_copy = versioning_guard.pinned_versions.clone();
            let pinned_snapshots_copy = versioning_guard.pinned_snapshots.clone();
            let stale_sstables_copy = versioning_guard.stale_sstables.clone();
            let sst_id_infos_copy = versioning_guard.sstable_id_infos.clone();
            (
                compact_status_copy,
                compact_task_assignment_copy,
                current_version_id_copy,
                hummmock_versions_copy,
                pinned_versions_copy,
                pinned_snapshots_copy,
                stale_sstables_copy,
                sst_id_infos_copy,
            )
        };
        let mem_state = get_state().await;
        self.load_meta_store_state()
            .await
            .expect("Failed to load state from meta store");
        let loaded_state = get_state().await;
        assert_eq!(
            mem_state, loaded_state,
            "hummock in-mem state is inconsistent with meta store state",
        );
    }

    /// When `version_id` is `None`, this function returns all the `SstableIdInfo` across all the
    /// versions. With `version_id` being specified, this function returns all the
    /// `SstableIdInfo` of `version_id` Version.
    pub async fn list_sstable_id_infos(
        &self,
        version_id: Option<HummockVersionId>,
    ) -> Result<Vec<SstableIdInfo>> {
        let versioning_guard = self.versioning.read().await;
        if version_id.is_none() {
            Ok(versioning_guard
                .sstable_id_infos
                .values()
                .cloned()
                .collect_vec())
        } else {
            let version_id = version_id.unwrap();
            let versioning = versioning_guard.hummock_versions.get(&version_id);
            versioning
                .map(|versioning| {
                    versioning
                        .levels
                        .iter()
                        .flat_map(|level| {
                            level.table_infos.iter().map(|table_info| {
                                versioning_guard
                                    .sstable_id_infos
                                    .get(&table_info.id)
                                    .unwrap()
                                    .clone()
                            })
                        })
                        .collect_vec()
                })
                .ok_or_else(|| {
                    Error::InternalError(format!(
                        "list_sstable_id_infos cannot find version:{}",
                        version_id
                    ))
                })
        }
    }

    pub async fn delete_sstable_ids(&self, sst_ids: impl AsRef<[HummockSSTableId]>) -> Result<()> {
        let mut versioning_guard = self.versioning.write().await;
        let mut sstable_id_infos = VarTransaction::new(&mut versioning_guard.sstable_id_infos);

        // Update in-mem state after transaction succeeds.
        for sst_id in sst_ids.as_ref() {
            sstable_id_infos.remove(sst_id);
        }

        commit_multi_var!(self, None, sstable_id_infos)?;

        #[cfg(test)]
        {
            drop(versioning_guard);
            self.check_state_consistency().await;
        }

        Ok(())
    }

    /// Release invalid contexts, aka worker node ids which are no longer valid in `ClusterManager`.
    async fn release_invalid_contexts(&self) -> Result<Vec<HummockContextId>> {
        let active_context_ids = {
            let compaction_guard = self.compaction.read().await;
            let versioning_guard = self.versioning.read().await;
            let mut active_context_ids = HashSet::new();
            active_context_ids.extend(
                compaction_guard
                    .compact_task_assignment
                    .values()
                    .map(|c| c.context_id),
            );
            active_context_ids.extend(versioning_guard.pinned_versions.keys());
            active_context_ids.extend(versioning_guard.pinned_snapshots.keys());
            active_context_ids
        };

        let mut invalid_context_ids = vec![];
        for active_context_id in &active_context_ids {
            if !self.check_context(*active_context_id).await {
                invalid_context_ids.push(*active_context_id);
            }
        }

        self.release_contexts(&invalid_context_ids).await?;

        Ok(invalid_context_ids)
    }

    /// Checks whether `context_id` is valid.
    pub async fn check_context(&self, context_id: HummockContextId) -> bool {
        self.cluster_manager
            .get_worker_by_id(context_id)
            .await
            .is_some()
    }

    /// Marks SSTs which haven't been added in meta (`meta_create_timestamp` is not set) for at
    /// least `sst_retention_interval` since `id_create_timestamp`
    pub async fn mark_orphan_ssts(
        &self,
        sst_retention_interval: Duration,
    ) -> Result<Vec<SstableIdInfo>> {
        let mut versioning_guard = self.versioning.write().await;
        let mut sstable_id_infos = VarTransaction::new(&mut versioning_guard.sstable_id_infos);

        let now = sstable_id_info::get_timestamp_now();
        let mut marked = vec![];
        for (_, sstable_id_info) in sstable_id_infos.iter_mut() {
            if sstable_id_info.meta_delete_timestamp != INVALID_TIMESTAMP {
                continue;
            }
            let is_orphan = sstable_id_info.meta_create_timestamp == INVALID_TIMESTAMP
                && now >= sstable_id_info.id_create_timestamp
                && now - sstable_id_info.id_create_timestamp >= sst_retention_interval.as_secs();
            if is_orphan {
                sstable_id_info.meta_delete_timestamp = now;
                marked.push(sstable_id_info.clone());
            }
        }
        if marked.is_empty() {
            return Ok(vec![]);
        }

        commit_multi_var!(self, None, sstable_id_infos)?;

        #[cfg(test)]
        {
            drop(versioning_guard);
            self.check_state_consistency().await;
        }

        tracing::debug!("Mark {:?} as orphan SSTs", marked);
        Ok(marked)
    }

    /// Gets current version without pinning it.
    pub async fn get_current_version(&self) -> HummockVersion {
        self.versioning.read().await.current_version()
    }

    pub fn set_compaction_scheduler(&self, sender: CompactionRequestChannelRef) {
        *self.compaction_scheduler.write() = Some(sender);
    }

    /// Cancels pending compaction tasks which are not yet assigned to any compactor.
    async fn cancel_unassigned_compaction_task(&self) -> Result<()> {
        let mut compaction_guard = self.compaction.write().await;
        let compaction = compaction_guard.deref_mut();
        let mut compact_status = VarTransaction::new(&mut compaction.compact_status);
        if compact_status.cancel_compaction_tasks_if(|pending_task_id| {
            !compaction
                .compact_task_assignment
                .contains_key(&pending_task_id)
        }) > 0
        {
            commit_multi_var!(self, None, compact_status)?;
        }
        #[cfg(test)]
        {
            drop(compaction_guard);
            self.check_state_consistency().await;
        }
        Ok(())
    }

    /// Sends a compaction request to compaction scheduler.
    fn try_send_compaction_request(&self, compaction_group: CompactionGroupId) -> bool {
        if let Some(sender) = self.compaction_scheduler.read().as_ref() {
            return sender.try_send(compaction_group);
        }
        false
    }
}
