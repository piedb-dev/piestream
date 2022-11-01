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

use piestream_hummock_sdk::compaction_group::StateTableId;
use tokio::sync::{mpsc, oneshot};

use super::memtable::Memtable;
use crate::hummock::HummockResult;
use crate::store::SyncResult;

// TODO: may use a different type
pub type StateStoreId = u64;

#[expect(dead_code)]
pub struct Batch {
    /// Immutable memtable.
    imm_mem: Arc<Memtable>,
    /// table_id to identify table configuration for writes.
    table_id: StateTableId,
    /// store_id to identify the state store instance.
    store_id: StateStoreId,
    // TODO: may add more
}

pub enum HummockEvent {
    /// Flushes a batch to persistent storage.
    Flush(Batch),

    /// Persists all flushed batches prior to this event.
    // Question: do we need to provide an epoch or an epoch range?
    Sync(oneshot::Sender<SyncResult>),
    // TODO: may add more (e.g. event to add state store instance)
}

#[expect(dead_code)]
pub struct HummockEventHandler {
    receiver: mpsc::UnboundedReceiver<HummockEvent>,
    // TODO: may add more
}

#[expect(dead_code)]
impl HummockEventHandler {
    fn handle(&self) -> HummockResult<()> {
        unimplemented!()
    }
}
