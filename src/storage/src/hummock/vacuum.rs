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

use std::sync::Arc;

use piestream_object_store::object::ObjectStore;
use piestream_pb::hummock::VacuumTask;
use piestream_rpc_client::HummockMetaClient;

use super::{HummockError, HummockResult};
use crate::hummock::SstableStoreRef;

pub struct Vacuum;

impl Vacuum {
    pub async fn vacuum(
        sstable_store: SstableStoreRef,
        vacuum_task: VacuumTask,
        hummock_meta_client: Arc<dyn HummockMetaClient>,
    ) -> HummockResult<()> {
        let store = sstable_store.store();
        let sst_ids = vacuum_task.sstable_ids;
        for sst_id in &sst_ids {
            // Meta
            store
                .delete(sstable_store.get_sst_meta_path(*sst_id).as_str())
                .await
                .map_err(HummockError::object_io_error)?;
            // Data
            store
                .delete(sstable_store.get_sst_data_path(*sst_id).as_str())
                .await
                .map_err(HummockError::object_io_error)?;
        }

        // TODO: report progress instead of in one go.
        hummock_meta_client
            .report_vacuum_task(VacuumTask {
                sstable_ids: sst_ids,
            })
            .await
            .map_err(|e| {
                HummockError::meta_error(format!("failed to report vacuum task: {e:?}"))
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::iter;
    use std::sync::Arc;

    use itertools::Itertools;
    use piestream_meta::hummock::test_utils::setup_compute_env;
    use piestream_meta::hummock::MockHummockMetaClient;
    use piestream_pb::hummock::VacuumTask;

    use crate::hummock::iterator::test_utils::{default_builder_opt_for_test, mock_sstable_store};
    use crate::hummock::test_utils::gen_default_test_sstable;
    use crate::hummock::vacuum::Vacuum;

    #[tokio::test]
    async fn test_vacuum_tracked_data() {
        let sstable_store = mock_sstable_store();
        // Put some SSTs to object store
        let sst_ids = (1..10).collect_vec();
        let mut sstables = vec![];
        for sstable_id in &sst_ids {
            let sstable = gen_default_test_sstable(
                default_builder_opt_for_test(),
                *sstable_id,
                sstable_store.clone(),
            )
            .await;
            sstables.push(sstable);
        }

        // Delete all existent SSTs and a nonexistent SSTs. Trying to delete a nonexistent SST is
        // OK.
        let nonexistent_id = 11u64;
        let vacuum_task = VacuumTask {
            sstable_ids: sst_ids
                .into_iter()
                .chain(iter::once(nonexistent_id))
                .collect_vec(),
        };
        let (_env, hummock_manager_ref, _cluster_manager_ref, worker_node) =
            setup_compute_env(8080).await;
        let mock_hummock_meta_client = Arc::new(MockHummockMetaClient::new(
            hummock_manager_ref.clone(),
            worker_node.id,
        ));
        Vacuum::vacuum(sstable_store, vacuum_task, mock_hummock_meta_client)
            .await
            .unwrap();
    }
}
