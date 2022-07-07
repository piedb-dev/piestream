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

use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use log::error;
use tokio::sync::Mutex;

use crate::meta_client::FrontendMetaClient;
use crate::scheduler::plan_fragmenter::QueryId;
use crate::scheduler::SchedulerError::PinSnapshot;
use crate::scheduler::SchedulerResult;

/// Cache of hummock snapshot in meta.
pub struct HummockSnapshotManager {
    core: Mutex<HummockSnapshotManagerCore>,
    meta_client: Arc<dyn FrontendMetaClient>,
}
pub type HummockSnapshotManagerRef = Arc<HummockSnapshotManager>;

impl HummockSnapshotManager {
    pub fn new(meta_client: Arc<dyn FrontendMetaClient>) -> Self {
        Self {
            core: Mutex::new(HummockSnapshotManagerCore::new()),
            meta_client,
        }
    }

    pub async fn get_epoch(&self, query_id: QueryId) -> SchedulerResult<u64> {
        let mut core_guard = self.core.lock().await;
        if core_guard.is_outdated {
            let epoch = self
                .meta_client
                .pin_snapshot(core_guard.last_pinned)
                .await
                .map_err(|_e| PinSnapshot(query_id.clone(), core_guard.last_pinned))?;
            core_guard.is_outdated = false;
            core_guard.last_pinned = epoch;
            core_guard.epoch_to_query_ids.insert(epoch, HashSet::new());
        }
        let last_pinned = core_guard.last_pinned;
        core_guard
            .epoch_to_query_ids
            .get_mut(&last_pinned)
            .unwrap()
            .insert(query_id);
        Ok(core_guard.last_pinned)
    }

    pub async fn unpin_snapshot(&self, epoch: u64, query_id: &QueryId) -> SchedulerResult<()> {
        let min_epoch = async {
            // Decrease the ref count of corresponding epoch. If all last pinned epoch is dropped,
            // mark as outdated.
            let mut core_guard = self.core.lock().await;
            let query_ids = core_guard.epoch_to_query_ids.get_mut(&epoch);
            if let Some(query_ids) = query_ids {
                query_ids.remove(query_id);
                if query_ids.is_empty() && epoch == core_guard.last_pinned {
                    core_guard.is_outdated = true;
                }
            }

            // Check the min epoch, if the epoch has no query running on it, this should be the min
            // epoch to be unpin.
            let mut min_epoch = None;
            if let Some((epoch, query_ids)) = core_guard.epoch_to_query_ids.first_key_value() {
                if query_ids.is_empty() {
                    min_epoch = Some(*epoch)
                }
            }
            // Remove the epoch from the map. Need to send RPC unpin_snapshot_before with this epoch
            // later.
            if let Some(min_epoch_inner) = min_epoch.as_ref() {
                core_guard.epoch_to_query_ids.remove(min_epoch_inner);
            }

            min_epoch
        };

        let need_to_request_meta = min_epoch.await;
        if let Some(epoch_inner) = need_to_request_meta {
            let meta_client = self.meta_client.clone();
            tokio::spawn(async move {
                tracing::info!("Unpin epoch {:?} with RPC", epoch_inner);
                let handle =
                    tokio::spawn(
                        async move { meta_client.unpin_snapshot_before(epoch_inner).await },
                    );
                if let Err(join_error) = handle.await && join_error.is_panic() {
                    error!("Request meta to unpin snapshot panic {:?}!", join_error);
                } else {
                    tracing::info!("Unpin epoch RPC succeed in frontend manager {:?}", epoch_inner);
                }
            });
        }
        Ok(())
    }

    /// Used in `ObserverManager`.
    pub async fn update_snapshot_status(&self, epoch: u64) {
        let mut core_guard = self.core.lock().await;
        if core_guard.last_pinned < epoch {
            core_guard.is_outdated = true;
        }
    }
}

#[derive(Default)]
struct HummockSnapshotManagerCore {
    is_outdated: bool,
    last_pinned: u64,
    /// Record the query ids that pin each snapshot.
    /// Send an `unpin_snapshot` RPC when a snapshot is not pinned any more.
    epoch_to_query_ids: BTreeMap<u64, HashSet<QueryId>>,
}

impl HummockSnapshotManagerCore {
    fn new() -> Self {
        Self {
            // Initialize by setting `is_outdated` to `true`.
            is_outdated: true,
            ..Default::default()
        }
    }
}
