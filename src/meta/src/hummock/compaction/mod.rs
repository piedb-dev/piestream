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

use itertools::Itertools;

pub mod compaction_config;
mod level_selector;
mod manual_compaction_picker;
mod min_overlap_compaction_picker;
mod overlap_strategy;
mod prost_type;
mod tier_compaction_picker;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use piestream_hummock_sdk::compaction_group::hummock_version_ext::HummockVersionExt;
use piestream_hummock_sdk::prost_key_range::KeyRangeExt;
use piestream_hummock_sdk::{CompactionGroupId, HummockCompactionTaskId, HummockEpoch};
use piestream_pb::hummock::compaction_config::CompactionMode;
use piestream_pb::hummock::{CompactTask, CompactionConfig, HummockVersion, KeyRange, Level};

use crate::hummock::compaction::level_selector::{DynamicLevelSelector, LevelSelector};
use crate::hummock::compaction::overlap_strategy::{
    HashStrategy, OverlapStrategy, RangeOverlapStrategy,
};
use crate::hummock::level_handler::LevelHandler;

pub struct CompactStatus {
    compaction_group_id: CompactionGroupId,
    pub(crate) level_handlers: Vec<LevelHandler>,
    // TODO: remove this `CompactionConfig`, which is a duplicate of that in `CompactionGroup`.
    pub compaction_config: CompactionConfig,
    compaction_selector: Arc<dyn LevelSelector>,
}

impl Debug for CompactStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompactStatus")
            .field("level_handlers", &self.level_handlers)
            .field("compaction_selector", &self.compaction_selector.name())
            .finish()
    }
}

impl PartialEq for CompactStatus {
    fn eq(&self, other: &Self) -> bool {
        self.level_handlers.eq(&other.level_handlers)
            && self.compaction_selector.name() == other.compaction_selector.name()
            && self.compaction_config == other.compaction_config
    }
}

impl Clone for CompactStatus {
    fn clone(&self) -> Self {
        Self {
            compaction_group_id: self.compaction_group_id,
            level_handlers: self.level_handlers.clone(),
            compaction_config: self.compaction_config.clone(),
            compaction_selector: self.compaction_selector.clone(),
        }
    }
}

#[derive(Debug, Default)]
pub struct SearchResult {
    select_level: Level,
    target_level: Level,
    split_ranges: Vec<KeyRange>,
    compression_algorithm: String,
    target_file_size: u64,
}

pub fn create_overlap_strategy(compaction_mode: CompactionMode) -> Arc<dyn OverlapStrategy> {
    match compaction_mode {
        CompactionMode::Range => Arc::new(RangeOverlapStrategy::default()),
        CompactionMode::ConsistentHash => Arc::new(HashStrategy::default()),
    }
}

impl CompactStatus {
    pub fn new(
        compaction_group_id: CompactionGroupId,
        config: Arc<CompactionConfig>,
    ) -> CompactStatus {
        let mut level_handlers = vec![];
        for level in 0..=config.max_level {
            level_handlers.push(LevelHandler::new(level as u32));
        }
        let overlap_strategy = create_overlap_strategy(config.compaction_mode());
        CompactStatus {
            compaction_group_id,
            level_handlers,
            compaction_config: (*config).clone(),
            compaction_selector: Arc::new(DynamicLevelSelector::new(config, overlap_strategy)),
        }
    }

    pub fn get_compact_task(
        &mut self,
        levels: &[Level],
        task_id: HummockCompactionTaskId,
        compaction_group_id: CompactionGroupId,
        manual_compaction_option: Option<ManualCompactionOption>,
    ) -> Option<CompactTask> {
        // When we compact the files, we must make the result of compaction meet the following
        // conditions, for any user key, the epoch of it in the file existing in the lower
        // layer must be larger.

        let ret = if let Some(manual_compaction_option) = manual_compaction_option {
            self.manual_pick_compaction(levels, task_id, manual_compaction_option)?
        } else {
            self.pick_compaction(levels, task_id)?
        };

        let select_level_id = ret.select_level.level_idx;
        let target_level_id = ret.target_level.level_idx;

        let splits = if ret.split_ranges.is_empty() {
            vec![KeyRange::inf()]
        } else {
            ret.split_ranges
        };

        let compression_algorithm = match ret.compression_algorithm.as_str() {
            "Lz4" => 1,
            "Zstd" => 2,
            _ => 0,
        };

        let compact_task = CompactTask {
            input_ssts: vec![ret.select_level, ret.target_level],
            splits,
            watermark: HummockEpoch::MAX,
            sorted_output_ssts: vec![],
            task_id,
            target_level: target_level_id,
            is_target_ultimate_and_leveling: target_level_id as usize
                == self.level_handlers.len() - 1
                && select_level_id > 0,
            task_status: false,
            vnode_mappings: vec![],
            compaction_group_id,
            existing_table_ids: vec![],
            compression_algorithm,
            target_file_size: ret.target_file_size,
            compaction_filter_mask: 0,
            table_options: HashMap::default(),
        };
        Some(compact_task)
    }

    fn pick_compaction(
        &mut self,
        levels: &[Level],
        task_id: HummockCompactionTaskId,
    ) -> Option<SearchResult> {
        self.compaction_selector
            .pick_compaction(task_id, levels, &mut self.level_handlers)
    }

    fn manual_pick_compaction(
        &mut self,
        levels: &[Level],
        task_id: HummockCompactionTaskId,
        manual_compaction_option: ManualCompactionOption,
    ) -> Option<SearchResult> {
        // manual_compaction no need to select level
        // level determined by option
        self.compaction_selector.manual_pick_compaction(
            task_id,
            levels,
            &mut self.level_handlers,
            manual_compaction_option,
        )
    }

    /// Declares a task is either finished or canceled.
    pub fn report_compact_task(&mut self, compact_task: &CompactTask) {
        for level in &compact_task.input_ssts {
            self.level_handlers[level.level_idx as usize].remove_task(compact_task.task_id);
        }
    }

    pub fn cancel_compaction_tasks_if<F: Fn(u64) -> bool>(&mut self, should_cancel: F) -> u32 {
        let mut count: u32 = 0;
        for level in &mut self.level_handlers {
            for pending_task_id in level.pending_tasks_ids() {
                if should_cancel(pending_task_id) {
                    level.remove_task(pending_task_id);
                    count += 1;
                }
            }
        }
        count
    }

    /// Applies the compact task result and get a new hummock version.
    pub fn apply_compact_result(
        compact_task: &CompactTask,
        based_hummock_version: HummockVersion,
    ) -> HummockVersion {
        let mut new_version = based_hummock_version;
        new_version.safe_epoch = std::cmp::max(new_version.safe_epoch, compact_task.watermark);
        let mut removed_table: HashSet<u64> = HashSet::default();
        for input_level in &compact_task.input_ssts {
            for table in &input_level.table_infos {
                removed_table.insert(table.id);
            }
        }
        let new_version_levels =
            new_version.get_compaction_group_levels_mut(compact_task.compaction_group_id);

        HummockVersion::apply_compact_ssts(
            new_version_levels,
            &compact_task
                .input_ssts
                .iter()
                .map(|level| level.level_idx)
                .collect_vec(),
            &removed_table,
            compact_task.target_level,
            compact_task.sorted_output_ssts.clone(),
        );

        new_version
    }

    pub fn compaction_group_id(&self) -> CompactionGroupId {
        self.compaction_group_id
    }

    pub fn get_config(&self) -> &CompactionConfig {
        &self.compaction_config
    }
}

#[derive(Clone, Debug)]
pub struct ManualCompactionOption {
    pub key_range: KeyRange,
    pub internal_table_id: HashSet<u32>,
    pub level: usize,
}

impl Default for ManualCompactionOption {
    fn default() -> Self {
        Self {
            key_range: KeyRange {
                left: vec![],
                right: vec![],
                inf: true,
            },
            internal_table_id: HashSet::default(),
            level: 1,
        }
    }
}

pub trait CompactionPicker {
    fn pick_compaction(
        &self,
        levels: &[Level],
        level_handlers: &mut [LevelHandler],
    ) -> Option<SearchResult>;
}
