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

use std::cmp::Ordering::{Equal, Less};
use std::sync::Arc;

use async_trait::async_trait;
use piestream_hummock_sdk::VersionedComparator;

use crate::hummock::iterator::{Backward, HummockIterator, ReadOptions};
use crate::hummock::value::HummockValue;
use crate::hummock::{
    BlockIterator, HummockResult, SSTableIteratorType, SstableStoreRef, TableHolder,
};
use crate::monitor::StoreLocalStatistic;

/// Iterates backwards on a table.
pub struct BackwardSSTableIterator {
    /// The iterator of the current block.
    block_iter: Option<BlockIterator>,

    /// Current block index.
    cur_idx: usize,

    /// Reference to the table
    pub sst: TableHolder,

    sstable_store: SstableStoreRef,

    stats: StoreLocalStatistic,
}

impl BackwardSSTableIterator {
    pub fn new(table: TableHolder, sstable_store: SstableStoreRef) -> Self {
        Self {
            block_iter: None,
            cur_idx: table.value().meta.block_metas.len() - 1,
            sst: table,
            sstable_store,
            stats: StoreLocalStatistic::default(),
        }
    }

    /// Seeks to a block, and then seeks to the key if `seek_key` is given.
    async fn seek_idx(&mut self, idx: isize, seek_key: Option<&[u8]>) -> HummockResult<()> {
        if idx >= self.sst.value().block_count() as isize || idx < 0 {
            self.block_iter = None;
        } else {
            let block = self
                .sstable_store
                .get(
                    self.sst.value(),
                    idx as u64,
                    crate::hummock::CachePolicy::Fill,
                    &mut self.stats,
                )
                .await?;
            let mut block_iter = BlockIterator::new(block);
            if let Some(key) = seek_key {
                block_iter.seek_le(key);
            } else {
                block_iter.seek_to_last();
            }

            self.block_iter = Some(block_iter);
            self.cur_idx = idx as usize;
        }

        Ok(())
    }
}

#[async_trait]
impl HummockIterator for BackwardSSTableIterator {
    type Direction = Backward;

    async fn next(&mut self) -> HummockResult<()> {
        self.stats.scan_key_count += 1;
        let block_iter = self.block_iter.as_mut().expect("no block iter");
        block_iter.prev();

        if block_iter.is_valid() {
            Ok(())
        } else {
            // seek to the previous block
            self.seek_idx(self.cur_idx as isize - 1, None).await
        }
    }

    fn key(&self) -> &[u8] {
        self.block_iter.as_ref().expect("no block iter").key()
    }

    fn value(&self) -> HummockValue<&[u8]> {
        let raw_value = self.block_iter.as_ref().expect("no block iter").value();

        HummockValue::from_slice(raw_value).expect("decode error")
    }

    fn is_valid(&self) -> bool {
        self.block_iter.as_ref().map_or(false, |i| i.is_valid())
    }

    /// Instead of setting idx to 0th block, a `BackwardSSTableIterator` rewinds to the last block
    /// in the table.
    async fn rewind(&mut self) -> HummockResult<()> {
        self.seek_idx(self.sst.value().block_count() as isize - 1, None)
            .await
    }

    async fn seek(&mut self, key: &[u8]) -> HummockResult<()> {
        let block_idx = self
            .sst
            .value()
            .meta
            .block_metas
            .partition_point(|block_meta| {
                // Compare by version comparator
                // Note: we are comparing against the `smallest_key` of the `block`, thus the
                // partition point should be `prev(<=)` instead of `<`.
                let ord = VersionedComparator::compare_key(block_meta.smallest_key.as_slice(), key);
                ord == Less || ord == Equal
            })
            .saturating_sub(1); // considering the boundary of 0
        let block_idx = block_idx as isize;

        self.seek_idx(block_idx, Some(key)).await?;
        if !self.is_valid() {
            // Seek to prev block
            self.seek_idx(block_idx - 1, None).await?;
        }

        Ok(())
    }

    fn collect_local_statistic(&self, stats: &mut StoreLocalStatistic) {
        stats.add(&self.stats)
    }
}

impl SSTableIteratorType for BackwardSSTableIterator {
    fn create(table: TableHolder, sstable_store: SstableStoreRef, _: Arc<ReadOptions>) -> Self {
        BackwardSSTableIterator::new(table, sstable_store)
    }
}

/// Mirror the tests used for `SSTableIterator`
#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use rand::prelude::*;
    use piestream_hummock_sdk::key::key_with_epoch;

    use super::*;
    use crate::assert_bytes_eq;
    use crate::hummock::iterator::test_utils::mock_sstable_store;
    use crate::hummock::test_utils::{
        create_small_table_cache, default_builder_opt_for_test, gen_default_test_sstable,
        test_key_of, test_value_of, TEST_KEYS_COUNT,
    };

    #[tokio::test]
    async fn test_backward_sstable_iterator() {
        // build remote table
        let sstable_store = mock_sstable_store();
        let table =
            gen_default_test_sstable(default_builder_opt_for_test(), 0, sstable_store.clone())
                .await;
        // We should have at least 10 blocks, so that table iterator test could cover more code
        // path.
        assert!(table.meta.block_metas.len() > 10);
        let cache = create_small_table_cache();
        let handle = cache.insert(0, 0, 1, Box::new(table));
        let mut sstable_iter = BackwardSSTableIterator::new(handle, sstable_store);
        let mut cnt = TEST_KEYS_COUNT;
        sstable_iter.rewind().await.unwrap();

        while sstable_iter.is_valid() {
            cnt -= 1;
            let key = sstable_iter.key();
            let value = sstable_iter.value();
            assert_bytes_eq!(key, test_key_of(cnt));
            assert_bytes_eq!(value.into_user_value().unwrap(), test_value_of(cnt));
            sstable_iter.next().await.unwrap();
        }

        assert_eq!(cnt, 0);
    }

    #[tokio::test]
    async fn test_backward_sstable_seek() {
        let sstable_store = mock_sstable_store();
        let table =
            gen_default_test_sstable(default_builder_opt_for_test(), 0, sstable_store.clone())
                .await;
        // We should have at least 10 blocks, so that table iterator test could cover more code
        // path.
        assert!(table.meta.block_metas.len() > 10);
        let cache = create_small_table_cache();
        let handle = cache.insert(0, 0, 1, Box::new(table));
        let mut sstable_iter = BackwardSSTableIterator::new(handle, sstable_store);
        let mut all_key_to_test = (0..TEST_KEYS_COUNT).collect_vec();
        let mut rng = thread_rng();
        all_key_to_test.shuffle(&mut rng);

        // We seek and access all the keys in random order
        for i in all_key_to_test {
            sstable_iter.seek(&test_key_of(i)).await.unwrap();
            // sstable_iter.next().await.unwrap();
            let key = sstable_iter.key();
            assert_bytes_eq!(key, test_key_of(i));
        }

        // Seek to key #TEST_KEYS_COUNT-500 and start iterating
        sstable_iter
            .seek(&test_key_of(TEST_KEYS_COUNT - 500))
            .await
            .unwrap();
        for i in (0..TEST_KEYS_COUNT - 500 + 1).rev() {
            let key = sstable_iter.key();
            assert_eq!(key, test_key_of(i), "key index:{}", i);
            sstable_iter.next().await.unwrap();
        }
        assert!(!sstable_iter.is_valid());

        let largest_key = key_with_epoch(format!("key_zzzz_{:05}", 0).as_bytes().to_vec(), 233);
        sstable_iter.seek(largest_key.as_slice()).await.unwrap();
        let key = sstable_iter.key();
        assert_eq!(key, test_key_of(TEST_KEYS_COUNT - 1));

        // Seek to > last key
        let smallest_key = key_with_epoch(format!("key_aaaa_{:05}", 0).as_bytes().to_vec(), 233);
        sstable_iter.seek(smallest_key.as_slice()).await.unwrap();
        assert!(!sstable_iter.is_valid());

        // Seek to non-existing key
        for idx in (1..TEST_KEYS_COUNT).rev() {
            // Seek to the previous key of each existing key. e.g.,
            // Our key space is `key_test_00000`, `key_test_00002`, `key_test_00004`, ...
            // And we seek to `key_test_00001` (will produce `key_test_00002`), `key_test_00003`
            // (will produce `key_test_00004`).
            sstable_iter
                .seek(
                    key_with_epoch(
                        format!("key_test_{:05}", idx * 2 - 1).as_bytes().to_vec(),
                        0,
                    )
                    .as_slice(),
                )
                .await
                .unwrap();

            let key = sstable_iter.key();
            assert_eq!(key, test_key_of(idx - 1));
            sstable_iter.next().await.unwrap();
        }
        assert!(!sstable_iter.is_valid());
    }
}
