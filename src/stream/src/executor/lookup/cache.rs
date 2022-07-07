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

use madsim::collections::BTreeSet;
use piestream_common::array::{Op, Row, StreamChunk};
use piestream_common::collection::evictable::EvictableHashMap;

use crate::executor::JOIN_CACHE_SIZE;

/// A cache for lookup's arrangement side.
pub struct LookupCache {
    data: EvictableHashMap<Row, BTreeSet<Row>>,
}

impl LookupCache {
    /// Lookup a row in cache. If not found, return `None`.
    pub fn lookup(&mut self, key: &Row) -> Option<&BTreeSet<Row>> {
        self.data.get(key)
    }

    /// Update a key after lookup cache misses.
    pub fn batch_update(&mut self, key: Row, value: impl Iterator<Item = Row>) {
        self.data.push(key, value.collect());
    }

    /// Apply a batch from the arrangement side
    pub fn apply_batch(&mut self, chunk: StreamChunk, arrange_join_keys: &[usize]) {
        for (op, row) in chunk.rows() {
            let key = row.row_by_indices(arrange_join_keys);
            if let Some(values) = self.data.get_mut(&key) {
                // the item is in cache, update it
                let value = row.to_owned_row();
                match op {
                    Op::Insert | Op::UpdateInsert => {
                        values.insert(value);
                    }
                    Op::Delete | Op::UpdateDelete => {
                        values.remove(&value);
                    }
                }
            }
        }
    }

    /// Flush the cache and evict the items.
    pub fn flush(&mut self) {
        self.data.evict_to_target_cap();
    }

    pub fn new() -> Self {
        Self {
            data: EvictableHashMap::new(JOIN_CACHE_SIZE),
        }
    }
}
