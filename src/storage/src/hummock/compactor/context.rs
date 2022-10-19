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

use std::sync::Arc;

use piestream_common::config::StorageConfig;
use piestream_hummock_sdk::filter_key_extractor::FilterKeyExtractorManagerRef;
use piestream_rpc_client::HummockMetaClient;

use super::task_progress::TaskProgressManagerRef;
use crate::hummock::compactor::{CompactionExecutor, CompactorSstableStoreRef};
use crate::hummock::sstable_store::SstableStoreRef;
use crate::hummock::{MemoryLimiter, SstableIdManagerRef};
use crate::monitor::StateStoreMetrics;

/// A `CompactorContext` describes the context of a compactor.
#[derive(Clone)]
pub struct Context {
    /// Storage configurations.
    pub options: Arc<StorageConfig>,

    /// The meta client.
    pub hummock_meta_client: Arc<dyn HummockMetaClient>,

    /// Sstable store that manages the sstables.
    pub sstable_store: SstableStoreRef,

    /// Statistics.
    pub stats: Arc<StateStoreMetrics>,

    /// True if it is a memory compaction (from shared buffer).
    pub is_share_buffer_compact: bool,

    pub compaction_executor: Arc<CompactionExecutor>,

    pub filter_key_extractor_manager: FilterKeyExtractorManagerRef,

    pub read_memory_limiter: Arc<MemoryLimiter>,

    pub sstable_id_manager: SstableIdManagerRef,

    pub task_progress_manager: TaskProgressManagerRef,
}

#[derive(Clone)]
pub struct CompactorContext {
    pub context: Arc<Context>,
    pub sstable_store: CompactorSstableStoreRef,
}
