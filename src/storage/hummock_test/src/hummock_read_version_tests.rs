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

use std::ops::Bound;
use std::sync::Arc;

use piestream_common::catalog::TableId;
use piestream_hummock_sdk::compaction_group::StaticCompactionGroupId;
use piestream_meta::hummock::test_utils::setup_compute_env;
use piestream_pb::hummock::SstableInfo;
use piestream_storage::hummock::iterator::test_utils::iterator_test_key_of_epoch;
use piestream_storage::hummock::shared_buffer::shared_buffer_batch::SharedBufferBatch;
use piestream_storage::hummock::store::memtable::ImmutableMemtable;
use piestream_storage::hummock::store::version::{
    HummockReadVersion, StagingData, StagingSstableInfo, VersionUpdate,
};
use piestream_storage::hummock::test_utils::{default_config_for_test, gen_dummy_batch};

use crate::test_utils::prepare_local_version_manager;

#[tokio::test]
async fn test_read_version_basic() {
    let opt = Arc::new(default_config_for_test());
    let (env, hummock_manager_ref, _cluster_manager_ref, worker_node) =
        setup_compute_env(8080).await;

    let local_version_manager =
        prepare_local_version_manager(opt, env, hummock_manager_ref, worker_node).await;

    let mut read_version = HummockReadVersion::new(local_version_manager.get_pinned_version());
    let mut epoch = 1;
    let compaction_group_id = StaticCompactionGroupId::StateDefault.into();
    let table_id = 0;

    {
        // single imm
        let kv_pairs = gen_dummy_batch(epoch);
        let imm = SharedBufferBatch::build_shared_buffer_batch(
            epoch,
            compaction_group_id,
            kv_pairs,
            TableId::from(table_id),
            None,
        )
        .await;

        read_version.update(VersionUpdate::Staging(StagingData::ImmMem(imm)));

        let key = iterator_test_key_of_epoch(0, epoch);
        let key_range = (Bound::Included(key.to_vec()), Bound::Included(key.to_vec()));

        let (staging_imm_iter, staging_sst_iter) =
            read_version
                .staging()
                .prune_overlap(epoch, compaction_group_id, &key_range);

        let staging_imm = staging_imm_iter
            .cloned()
            .collect::<Vec<ImmutableMemtable>>();

        assert_eq!(1, staging_imm.len());
        assert_eq!(0, staging_sst_iter.count());
        assert!(staging_imm.iter().any(|imm| imm.epoch() <= epoch));
    }

    {
        // several epoch
        for _ in 0..5 {
            epoch += 1;
            let kv_pairs = gen_dummy_batch(epoch);
            let imm = SharedBufferBatch::build_shared_buffer_batch(
                epoch,
                compaction_group_id,
                kv_pairs,
                TableId::from(table_id),
                None,
            )
            .await;

            read_version.update(VersionUpdate::Staging(StagingData::ImmMem(imm)));
        }

        let key = iterator_test_key_of_epoch(0, epoch);
        let key_range = (Bound::Included(key.to_vec()), Bound::Included(key.to_vec()));

        let (staging_imm_iter, staging_sst_iter) =
            read_version
                .staging()
                .prune_overlap(epoch, compaction_group_id, &key_range);

        let staging_imm = staging_imm_iter
            .cloned()
            .collect::<Vec<ImmutableMemtable>>();

        assert_eq!(1, staging_imm.len());
        assert_eq!(0, staging_sst_iter.count());
        assert!(staging_imm.iter().any(|imm| imm.epoch() <= epoch));
    }

    {
        // test clean imm with sst update info
        let staging = read_version.staging();
        assert_eq!(6, staging.imm.len());
        let batch_id_vec_for_clear = staging
            .imm
            .iter()
            .rev()
            .map(|imm| imm.batch_id())
            .take(3)
            .rev()
            .collect::<Vec<_>>();

        let epoch_id_vec_for_clear = staging
            .imm
            .iter()
            .rev()
            .map(|imm| imm.epoch())
            .take(3)
            .rev()
            .collect::<Vec<_>>();

        let dummy_sst = StagingSstableInfo::new(
            SstableInfo {
                id: 1,
                key_range: None,
                file_size: 1,
                table_ids: vec![0],
                meta_offset: 1,
                stale_key_count: 1,
                total_key_count: 1,
                divide_version: 0,
            },
            epoch_id_vec_for_clear,
            compaction_group_id,
            batch_id_vec_for_clear,
        );

        {
            read_version.update(VersionUpdate::Staging(StagingData::Sst(dummy_sst)));
        }
    }

    {
        // test clear related batch after update sst

        // after update sst
        // imm(0, 1, 2) => sst{sst_id: 1}
        // staging => {imm(3, 4, 5), sst{sst_id: 1}}
        let staging = read_version.staging();
        assert_eq!(3, read_version.staging().imm.len());
        assert_eq!(1, read_version.staging().sst.len());
        let remain_batch_id_vec = staging
            .imm
            .iter()
            .map(|imm| imm.batch_id())
            .collect::<Vec<_>>();
        assert!(remain_batch_id_vec.iter().any(|batch_id| *batch_id > 2));
    }
}
