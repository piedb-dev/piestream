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
use std::time::Duration;

use itertools::Itertools;
use piestream_hummock_sdk::compaction_group::hummock_version_ext::HummockVersionExt;
use piestream_hummock_sdk::compaction_group::StaticCompactionGroupId;
use piestream_hummock_sdk::key::key_with_epoch;
use piestream_hummock_sdk::{
    CompactionGroupId, HummockContextId, HummockEpoch, HummockSstableId, LocalSstableInfo,
};
use piestream_pb::common::{HostAddress, WorkerNode, WorkerType};
use piestream_pb::hummock::compact_task::TaskStatus;
use piestream_pb::hummock::{CompactionConfig, HummockVersion, KeyRange, SstableInfo};

use crate::hummock::compaction::compaction_config::CompactionConfigBuilder;
use crate::hummock::compaction_group::manager::{
    CompactionGroupManager, CompactionGroupManagerRef,
};
use crate::hummock::compaction_group::TableOption;
use crate::hummock::{CompactorManager, HummockManager, HummockManagerRef};
use crate::manager::{ClusterManager, ClusterManagerRef, MetaSrvEnv, META_NODE_ID};
use crate::rpc::metrics::MetaMetrics;
use crate::storage::{MemStore, MetaStore};

pub fn to_local_sstable_info(ssts: &[SstableInfo]) -> Vec<LocalSstableInfo> {
    ssts.iter()
        .map(|sst| (StaticCompactionGroupId::StateDefault.into(), sst.clone()))
        .collect_vec()
}

pub async fn add_test_tables<S>(
    hummock_manager: &HummockManager<S>,
    context_id: HummockContextId,
) -> Vec<Vec<SstableInfo>>
where
    S: MetaStore,
{
    // Increase version by 2.
    let mut epoch: u64 = 1;
    let table_ids = get_sst_ids(hummock_manager, 3).await;
    let test_tables = generate_test_tables(epoch, table_ids);
    register_sstable_infos_to_compaction_group(
        hummock_manager.compaction_group_manager(),
        &test_tables,
        StaticCompactionGroupId::StateDefault.into(),
    )
    .await;
    let ssts = to_local_sstable_info(&test_tables);
    let sst_to_worker = ssts.iter().map(|(_, sst)| (sst.id, context_id)).collect();
    hummock_manager
        .commit_epoch(epoch, ssts, sst_to_worker)
        .await
        .unwrap();
    // Current state: {v0: [], v1: [test_tables]}

    // Simulate a compaction and increase version by 1.
    let mut temp_compactor = false;
    if hummock_manager
        .compactor_manager_ref_for_test()
        .compactor_num()
        == 0
    {
        hummock_manager
            .compactor_manager_ref_for_test()
            .add_compactor(context_id, u64::MAX);
        temp_compactor = true;
    }
    let compactor = hummock_manager.get_idle_compactor().await.unwrap();
    let mut compact_task = hummock_manager
        .get_compact_task(StaticCompactionGroupId::StateDefault.into())
        .await
        .unwrap()
        .unwrap();
    compact_task.target_level = 6;
    hummock_manager
        .assign_compaction_task(&compact_task, compactor.context_id())
        .await
        .unwrap();
    if temp_compactor {
        assert_eq!(compactor.context_id(), context_id);
    }
    let test_tables_2 = generate_test_tables(epoch, get_sst_ids(hummock_manager, 1).await);
    register_sstable_infos_to_compaction_group(
        hummock_manager.compaction_group_manager(),
        &test_tables_2,
        StaticCompactionGroupId::StateDefault.into(),
    )
    .await;
    compact_task.sorted_output_ssts = test_tables_2.clone();
    compact_task.set_task_status(TaskStatus::Success);
    hummock_manager
        .report_compact_task(context_id, &mut compact_task)
        .await
        .unwrap();
    if temp_compactor {
        hummock_manager
            .compactor_manager_ref_for_test()
            .remove_compactor(context_id);
    }
    // Current state: {v0: [], v1: [test_tables], v2: [test_tables_2, test_tables to_delete]}

    // Increase version by 1.
    epoch += 1;
    let test_tables_3 = generate_test_tables(epoch, get_sst_ids(hummock_manager, 1).await);
    register_sstable_infos_to_compaction_group(
        hummock_manager.compaction_group_manager(),
        &test_tables_3,
        StaticCompactionGroupId::StateDefault.into(),
    )
    .await;
    let ssts = to_local_sstable_info(&test_tables_3);
    let sst_to_worker = ssts.iter().map(|(_, sst)| (sst.id, context_id)).collect();
    hummock_manager
        .commit_epoch(epoch, ssts, sst_to_worker)
        .await
        .unwrap();
    // Current state: {v0: [], v1: [test_tables], v2: [test_tables_2, to_delete:test_tables], v3:
    // [test_tables_2, test_tables_3]}
    vec![test_tables, test_tables_2, test_tables_3]
}

pub fn generate_test_tables(epoch: u64, sst_ids: Vec<HummockSstableId>) -> Vec<SstableInfo> {
    let mut sst_info = vec![];
    for (i, sst_id) in sst_ids.into_iter().enumerate() {
        sst_info.push(SstableInfo {
            id: sst_id,
            key_range: Some(KeyRange {
                left: iterator_test_key_of_epoch(sst_id, i + 1, epoch),
                right: iterator_test_key_of_epoch(sst_id, (i + 1) * 10, epoch),
                inf: false,
            }),
            file_size: 2,
            table_ids: vec![(i + 1) as u32, (i + 2) as u32],
            meta_offset: 0,
            stale_key_count: 0,
            total_key_count: 0,
        });
    }
    sst_info
}

pub async fn register_sstable_infos_to_compaction_group<S>(
    compaction_group_manager_ref: CompactionGroupManagerRef<S>,
    sstable_infos: &[SstableInfo],
    compaction_group_id: CompactionGroupId,
) where
    S: MetaStore,
{
    let table_ids = sstable_infos
        .iter()
        .flat_map(|sstable_info| &sstable_info.table_ids)
        .dedup()
        .cloned()
        .collect_vec();
    register_table_ids_to_compaction_group(
        compaction_group_manager_ref,
        &table_ids,
        compaction_group_id,
    )
    .await;
}

pub async fn register_table_ids_to_compaction_group<S>(
    compaction_group_manager_ref: CompactionGroupManagerRef<S>,
    table_ids: &[u32],
    compaction_group_id: CompactionGroupId,
) where
    S: MetaStore,
{
    compaction_group_manager_ref
        .register_table_ids(
            &mut table_ids
                .iter()
                .map(|table_id| (*table_id, compaction_group_id, TableOption::default()))
                .collect_vec(),
        )
        .await
        .unwrap();
}

pub async fn unregister_table_ids_from_compaction_group<S>(
    compaction_group_manager_ref: CompactionGroupManagerRef<S>,
    table_ids: &[u32],
) where
    S: MetaStore,
{
    compaction_group_manager_ref
        .unregister_table_ids(table_ids)
        .await
        .unwrap();
}

/// Generate keys like `001_key_test_00002` with timestamp `epoch`.
pub fn iterator_test_key_of_epoch(
    table: HummockSstableId,
    idx: usize,
    ts: HummockEpoch,
) -> Vec<u8> {
    // key format: {prefix_index}_version
    key_with_epoch(
        format!("{:03}_key_test_{:05}", table, idx)
            .as_bytes()
            .to_vec(),
        ts,
    )
}

pub fn get_sorted_sstable_ids(sstables: &[SstableInfo]) -> Vec<HummockSstableId> {
    sstables.iter().map(|table| table.id).sorted().collect_vec()
}

pub fn get_sorted_committed_sstable_ids(hummock_version: &HummockVersion) -> Vec<HummockSstableId> {
    let levels =
        hummock_version.get_compaction_group_levels(StaticCompactionGroupId::StateDefault.into());
    levels
        .levels
        .iter()
        .chain(levels.l0.as_ref().unwrap().sub_levels.iter())
        .flat_map(|levels| levels.table_infos.iter().map(|info| info.id))
        .sorted()
        .collect_vec()
}
pub async fn setup_compute_env_with_config(
    port: i32,
    config: CompactionConfig,
) -> (
    MetaSrvEnv<MemStore>,
    HummockManagerRef<MemStore>,
    ClusterManagerRef<MemStore>,
    WorkerNode,
) {
    let env = MetaSrvEnv::for_test().await;
    let cluster_manager = Arc::new(
        ClusterManager::new(env.clone(), Duration::from_secs(1))
            .await
            .unwrap(),
    );

    let compaction_group_manager = Arc::new(
        CompactionGroupManager::with_config(env.clone(), config.clone())
            .await
            .unwrap(),
    );

    let compactor_manager = Arc::new(CompactorManager::for_test());

    let hummock_manager = Arc::new(
        HummockManager::new(
            env.clone(),
            cluster_manager.clone(),
            Arc::new(MetaMetrics::new()),
            compaction_group_manager,
            compactor_manager,
        )
        .await
        .unwrap(),
    );
    let fake_host_address = HostAddress {
        host: "127.0.0.1".to_string(),
        port,
    };
    let fake_parallelism = 4;
    let worker_node = cluster_manager
        .add_worker_node(WorkerType::ComputeNode, fake_host_address, fake_parallelism)
        .await
        .unwrap();
    (env, hummock_manager, cluster_manager, worker_node)
}

pub async fn setup_compute_env(
    port: i32,
) -> (
    MetaSrvEnv<MemStore>,
    HummockManagerRef<MemStore>,
    ClusterManagerRef<MemStore>,
    WorkerNode,
) {
    let config = CompactionConfigBuilder::new()
        .level0_tier_compact_file_number(1)
        .build();
    setup_compute_env_with_config(port, config).await
}

pub async fn get_sst_ids<S>(
    hummock_manager: &HummockManager<S>,
    number: u32,
) -> Vec<HummockSstableId>
where
    S: MetaStore,
{
    let range = hummock_manager.get_new_sst_ids(number).await.unwrap();
    (range.start_id..range.end_id).collect_vec()
}

pub async fn commit_from_meta_node<S>(
    hummock_manager_ref: &HummockManager<S>,
    epoch: HummockEpoch,
    ssts: Vec<LocalSstableInfo>,
) -> crate::hummock::error::Result<()>
where
    S: MetaStore,
{
    let sst_to_worker = ssts.iter().map(|(_, sst)| (sst.id, META_NODE_ID)).collect();
    hummock_manager_ref
        .commit_epoch(epoch, ssts, sst_to_worker)
        .await
}

pub async fn add_ssts<S>(
    epoch: HummockEpoch,
    hummock_manager: &HummockManager<S>,
    context_id: HummockContextId,
) -> Vec<SstableInfo>
where
    S: MetaStore,
{
    let table_ids = get_sst_ids(hummock_manager, 3).await;
    let test_tables = generate_test_tables(epoch, table_ids);
    register_sstable_infos_to_compaction_group(
        hummock_manager.compaction_group_manager(),
        &test_tables,
        StaticCompactionGroupId::StateDefault.into(),
    )
    .await;
    let ssts = to_local_sstable_info(&test_tables);
    let sst_to_worker = ssts.iter().map(|(_, sst)| (sst.id, context_id)).collect();
    hummock_manager
        .commit_epoch(epoch, ssts, sst_to_worker)
        .await
        .unwrap();
    test_tables
}
