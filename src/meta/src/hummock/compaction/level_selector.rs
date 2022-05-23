//  Copyright 2022 Singularity Data
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
//
// Copyright (c) 2011-present, Facebook, Inc.  All rights reserved.
// This source code is licensed under both the GPLv2 (found in the
// COPYING file in the root directory) and Apache 2.0 License
// (found in the LICENSE.Apache file in the root directory).

use std::sync::Arc;

use risingwave_pb::hummock::Level;

use crate::hummock::compaction::compaction_picker::{CompactionPicker, MinOverlappingPicker};
use crate::hummock::compaction::overlap_strategy::{
    HashStrategy, OverlapStrategy, RangeOverlapStrategy,
};
use crate::hummock::compaction::tier_compaction_picker::TierCompactionPicker;
use crate::hummock::compaction::CompactionMode::{ConsistentHashMode, RangeMode};
use crate::hummock::compaction::{CompactionConfig, SearchResult};
use crate::hummock::level_handler::LevelHandler;

const SCORE_BASE: u64 = 100;

pub trait LevelSelector: Sync + Send {
    fn need_compaction(&self, levels: &[Level], level_handlers: &mut [LevelHandler]) -> bool;

    fn pick_compaction(
        &self,
        task_id: u64,
        levels: &[Level],
        level_handlers: &mut [LevelHandler],
    ) -> Option<SearchResult>;

    fn name(&self) -> &'static str;
}

#[derive(Default)]
pub struct SelectContext {
    level_max_bytes: Vec<u64>,

    // All data will be placed in the last level. When the cluster is empty, the files in L0 will
    // be compact to `max_level`, and the `max_level` would be `base_level`. When the total
    // size of the files in  `base_level` reaches its capacity, we will place data in a higher
    // level, which equals to `base_level -= 1;`.
    base_level: usize,
    score_levels: Vec<(u64, usize)>,
}

// TODO: Set these configurations by meta rpc
pub struct DynamicLevelSelector {
    config: Arc<CompactionConfig>,
    overlap_strategy: Arc<dyn OverlapStrategy>,
}

impl Default for DynamicLevelSelector {
    fn default() -> Self {
        let config = Arc::new(CompactionConfig::default());
        let overlap_strategy = match &config.compaction_mode {
            RangeMode => Arc::new(RangeOverlapStrategy::default()) as Arc<dyn OverlapStrategy>,
            ConsistentHashMode => Arc::new(HashStrategy::default()),
        };
        DynamicLevelSelector::new(config, overlap_strategy)
    }
}

impl DynamicLevelSelector {
    pub fn new(config: Arc<CompactionConfig>, overlap_strategy: Arc<dyn OverlapStrategy>) -> Self {
        DynamicLevelSelector {
            config,
            overlap_strategy,
        }
    }

    fn create_compaction_picker(
        &self,
        level: usize,
        base_level: usize,
        task_id: u64,
    ) -> Box<dyn CompactionPicker> {
        if level == 0 {
            Box::new(TierCompactionPicker::new(
                task_id,
                base_level,
                self.config.clone(),
                self.overlap_strategy.clone(),
            ))
        } else {
            Box::new(MinOverlappingPicker::new(
                task_id,
                level,
                self.overlap_strategy.clone(),
            ))
        }
    }

    // TODO: calculate this scores in apply compact result.
    /// `calculate_level_base_size` calculate base level and the base size of LSM tree build for
    /// current dataset. In other words,  `level_max_bytes` is our compaction goal which shall
    /// reach. This algorithm refers to the implementation in  `</>https://github.com/facebook/rocksdb/blob/v7.2.2/db/version_set.cc#L3706</>`
    fn calculate_level_base_size(&self, levels: &[Level]) -> SelectContext {
        let mut first_non_empty_level = 0;
        let mut max_level_size = 0;
        let mut ctx = SelectContext::default();

        let mut l0_size = 0;
        for level in levels.iter() {
            let mut total_file_size = 0;
            for table in &level.table_infos {
                total_file_size += table.file_size;
            }
            if level.level_idx > 0 {
                if total_file_size > 0 && first_non_empty_level == 0 {
                    first_non_empty_level = level.level_idx as usize;
                }
                max_level_size = std::cmp::max(max_level_size, total_file_size);
            } else {
                l0_size = total_file_size;
            }
        }

        ctx.level_max_bytes
            .resize(self.config.max_level as usize + 1, u64::MAX);

        if max_level_size == 0 {
            // Use the bottommost level.
            ctx.base_level = self.config.max_level;
            return ctx;
        }

        let base_bytes_max = std::cmp::max(self.config.max_bytes_for_level_base, l0_size);
        let base_bytes_min = base_bytes_max / self.config.max_bytes_for_level_multiplier;

        let mut cur_level_size = max_level_size;
        for _ in first_non_empty_level..self.config.max_level {
            cur_level_size /= self.config.max_bytes_for_level_multiplier;
        }

        let base_level_size = if cur_level_size <= base_bytes_min {
            // Case 1. If we make target size of last level to be max_level_size,
            // target size of the first non-empty level would be smaller than
            // base_bytes_min. We set it be base_bytes_min.
            ctx.base_level = first_non_empty_level;
            base_bytes_min + 1
        } else {
            ctx.base_level = first_non_empty_level;
            while ctx.base_level > 1 && cur_level_size > base_bytes_max {
                ctx.base_level -= 1;
                cur_level_size /= self.config.max_bytes_for_level_multiplier;
            }
            std::cmp::min(base_bytes_max, cur_level_size)
        };

        let level_multiplier = self.config.max_bytes_for_level_multiplier as f64;
        let mut level_size = base_level_size;
        for i in ctx.base_level..=self.config.max_level {
            // Don't set any level below base_bytes_max. Otherwise, the LSM can
            // assume an hourglass shape where L1+ sizes are smaller than L0. This
            // causes compaction scoring, which depends on level sizes, to favor L1+
            // at the expense of L0, which may fill up and stall.
            ctx.level_max_bytes[i] = std::cmp::max(level_size, base_bytes_max);
            level_size = (level_size as f64 * level_multiplier) as u64;
        }
        ctx
    }

    fn get_priority_levels(
        &self,
        levels: &[Level],
        handlers: &mut [LevelHandler],
    ) -> SelectContext {
        let mut ctx = self.calculate_level_base_size(levels);

        // The bottommost level can not be input level.
        for level in &levels[..self.config.max_level] {
            let level_idx = level.level_idx as usize;
            let mut total_size = 0;
            let mut idle_file_count = 0;
            for table in &level.table_infos {
                if !handlers[level_idx].is_pending_compact(&table.id) {
                    total_size += table.file_size;
                    idle_file_count += 1;
                }
            }
            if total_size == 0 {
                continue;
            }
            if level_idx == 0 {
                // The number of files in L0 can grow quickly due to frequent checkpoint. So, if we
                // set level0_trigger_number too small, the manager will always
                // compact the files in L0 and it would make the whole tree more unbalanced. So we
                // set level0_trigger_number a large number to make compaction
                // manager can trigger compaction jobs of other level but we add a base score
                // `idle_file_count + 100` so that the manager can trigger L0
                // compaction when the other levels are all balanced.
                let score = idle_file_count * SCORE_BASE / self.config.level0_trigger_number as u64
                    + idle_file_count
                    + SCORE_BASE;
                let score = std::cmp::max(
                    total_size * SCORE_BASE / self.config.max_bytes_for_level_base,
                    score,
                );
                ctx.score_levels.push((score, 0));
            } else {
                ctx.score_levels.push((
                    total_size * SCORE_BASE / ctx.level_max_bytes[level_idx],
                    level_idx,
                ));
            }
        }

        // sort reverse to pick the largest one.
        ctx.score_levels.sort_by(|a, b| b.0.cmp(&a.0));
        ctx
    }
}

impl LevelSelector for DynamicLevelSelector {
    fn need_compaction(&self, levels: &[Level], level_handlers: &mut [LevelHandler]) -> bool {
        let ctx = self.get_priority_levels(levels, level_handlers);
        ctx.score_levels
            .first()
            .map(|(score, _)| *score > SCORE_BASE)
            .unwrap_or(false)
    }

    fn pick_compaction(
        &self,
        task_id: u64,
        levels: &[Level],
        level_handlers: &mut [LevelHandler],
    ) -> Option<SearchResult> {
        let ctx = self.get_priority_levels(levels, level_handlers);
        for (score, level_idx) in ctx.score_levels {
            if score <= SCORE_BASE {
                return None;
            }
            let picker = self.create_compaction_picker(level_idx, ctx.base_level, task_id);
            if let Some(ret) = picker.pick_compaction(levels, level_handlers) {
                return Some(ret);
            }
        }
        None
    }

    fn name(&self) -> &'static str {
        "DynamicLevelSelector"
    }
}

#[cfg(test)]
pub mod tests {
    use std::ops::Range;

    use itertools::Itertools;
    use risingwave_pb::hummock::{LevelType, SstableInfo};

    use super::*;
    use crate::hummock::compaction::overlap_strategy::RangeOverlapStrategy;
    use crate::hummock::compaction::tier_compaction_picker::tests::generate_table;
    use crate::hummock::compaction::CompactionMode::RangeMode;

    pub fn generate_tables(
        ids: Range<u64>,
        keys: Range<usize>,
        epoch: u64,
        file_size: u64,
    ) -> Vec<SstableInfo> {
        let step = (keys.end - keys.start) / (ids.end - ids.start) as usize;
        let mut start = keys.start;
        let mut tables = vec![];
        for id in ids {
            let mut table = generate_table(id, 1, start, start + step - 1, epoch);
            table.file_size = file_size;
            tables.push(table);
            start += step;
        }
        tables
    }

    #[test]
    fn test_dynamic_level() {
        let config = CompactionConfig {
            max_bytes_for_level_base: 100,
            max_level: 4,
            max_bytes_for_level_multiplier: 5,
            max_compaction_bytes: 0,
            level0_max_file_number: 0,
            level0_trigger_number: 2,
            compaction_mode: RangeMode,
        };
        let selector =
            DynamicLevelSelector::new(Arc::new(config), Arc::new(RangeOverlapStrategy::default()));
        let mut levels = vec![
            Level {
                level_idx: 0,
                level_type: LevelType::Overlapping as i32,
                table_infos: vec![],
            },
            Level {
                level_idx: 1,
                level_type: LevelType::Nonoverlapping as i32,
                table_infos: vec![],
            },
            Level {
                level_idx: 2,
                level_type: LevelType::Nonoverlapping as i32,
                table_infos: generate_tables(0..5, 0..1000, 3, 10),
            },
            Level {
                level_idx: 3,
                level_type: LevelType::Nonoverlapping as i32,
                table_infos: generate_tables(5..10, 0..1000, 2, 50),
            },
            Level {
                level_idx: 4,
                level_type: LevelType::Nonoverlapping as i32,
                table_infos: generate_tables(10..15, 0..1000, 1, 200),
            },
        ];
        let ctx = selector.calculate_level_base_size(&levels);
        assert_eq!(ctx.base_level, 2);
        assert_eq!(ctx.level_max_bytes[2], 100);
        assert_eq!(ctx.level_max_bytes[3], 200);
        assert_eq!(ctx.level_max_bytes[4], 1000);

        levels[4]
            .table_infos
            .append(&mut generate_tables(15..20, 2000..3000, 1, 400));
        let ctx = selector.calculate_level_base_size(&levels);
        // data size increase, so we need increase one level to place more data.
        assert_eq!(ctx.base_level, 1);
        assert_eq!(ctx.level_max_bytes[1], 100);
        assert_eq!(ctx.level_max_bytes[2], 120);
        assert_eq!(ctx.level_max_bytes[3], 600);
        assert_eq!(ctx.level_max_bytes[4], 3000);

        // append a large data to L0 but it does not change the base size of LSM tree.
        levels[0]
            .table_infos
            .append(&mut generate_tables(20..26, 0..1000, 1, 100));
        let ctx = selector.calculate_level_base_size(&levels);
        assert_eq!(ctx.base_level, 2);
        assert_eq!(ctx.level_max_bytes[2], 600);
        assert_eq!(ctx.level_max_bytes[3], 605);
        assert_eq!(ctx.level_max_bytes[4], 3025);

        levels[0].table_infos.clear();
        levels[1].table_infos = generate_tables(26..32, 0..1000, 1, 100);
        let ctx = selector.calculate_level_base_size(&levels);
        assert_eq!(ctx.base_level, 1);
        assert_eq!(ctx.level_max_bytes[1], 100);
        assert_eq!(ctx.level_max_bytes[2], 120);
        assert_eq!(ctx.level_max_bytes[3], 600);
        assert_eq!(ctx.level_max_bytes[4], 3000);
    }

    #[test]
    fn test_pick_compaction() {
        let config = CompactionConfig {
            max_bytes_for_level_base: 100,
            max_level: 4,
            max_bytes_for_level_multiplier: 5,
            max_compaction_bytes: 10000,
            level0_max_file_number: 0,
            level0_trigger_number: 2,
            compaction_mode: RangeMode,
        };
        let selector =
            DynamicLevelSelector::new(Arc::new(config), Arc::new(RangeOverlapStrategy::default()));
        let mut levels = vec![
            Level {
                level_idx: 0,
                level_type: LevelType::Overlapping as i32,
                table_infos: generate_tables(15..20, 0..600, 3, 10),
            },
            Level {
                level_idx: 1,
                level_type: LevelType::Nonoverlapping as i32,
                table_infos: vec![],
            },
            Level {
                level_idx: 2,
                level_type: LevelType::Nonoverlapping as i32,
                table_infos: generate_tables(0..5, 0..1000, 3, 10),
            },
            Level {
                level_idx: 3,
                level_type: LevelType::Nonoverlapping as i32,
                table_infos: generate_tables(5..10, 0..1000, 2, 50),
            },
            Level {
                level_idx: 4,
                level_type: LevelType::Nonoverlapping as i32,
                table_infos: generate_tables(10..15, 0..1000, 1, 200),
            },
        ];
        let mut levels_handlers = (0..5).into_iter().map(LevelHandler::new).collect_vec();
        let compaction = selector
            .pick_compaction(1, &levels, &mut levels_handlers)
            .unwrap();
        assert_eq!(compaction.select_level.level_idx, 0);
        assert_eq!(compaction.target_level.level_idx, 2);
        assert_eq!(compaction.select_level.table_infos.len(), 5);
        assert_eq!(compaction.target_level.table_infos.len(), 3);
        levels_handlers[0].remove_task(1);
        levels_handlers[2].remove_task(1);
        levels[0].table_infos.clear();
        levels[2].table_infos = generate_tables(20..30, 0..1000, 3, 10);
        let compaction = selector
            .pick_compaction(2, &levels, &mut levels_handlers)
            .unwrap();
        assert_eq!(compaction.select_level.level_idx, 3);
        assert_eq!(compaction.target_level.level_idx, 4);
        assert_eq!(compaction.select_level.table_infos.len(), 1);
        assert_eq!(compaction.target_level.table_infos.len(), 1);

        // no compaction need to be scheduled because we do not calculate the size of pending files
        // to score.
        let compaction = selector.pick_compaction(2, &levels, &mut levels_handlers);
        assert!(compaction.is_none());
    }
}
