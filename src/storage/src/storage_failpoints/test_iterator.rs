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

use std::ops::Bound::Unbounded;
use std::sync::Arc;

use futures::executor::block_on;
use piestream_hummock_sdk::key::user_key;

use crate::hummock::iterator::test_utils::{
    gen_iterator_test_sstable_base, iterator_test_key_of, iterator_test_value_of,
    mock_sstable_store, TEST_KEYS_COUNT,
};
use crate::hummock::iterator::{
    Backward, BackwardConcatIterator, BackwardMergeIterator, BackwardUserIterator,
    BoxedBackwardHummockIterator, BoxedForwardHummockIterator, ConcatIterator, Forward,
    HummockIterator, MergeIterator, ReadOptions, UserIterator,
};
use crate::hummock::test_utils::default_builder_opt_for_test;
use crate::hummock::{BackwardSSTableIterator, SSTableIterator};
use crate::monitor::{StateStoreMetrics, StoreLocalStatistic};

#[tokio::test]
#[cfg(feature = "failpoints")]
async fn test_failpoints_concat_read_err() {
    fail::cfg("disable_block_cache", "return").unwrap();
    fail::cfg("disable_bloom_filter", "return").unwrap();
    let mem_read_err = "mem_read_err";
    let sstable_store = mock_sstable_store();
    let table0 = gen_iterator_test_sstable_base(
        0,
        default_builder_opt_for_test(),
        |x| x * 2,
        sstable_store.clone(),
        TEST_KEYS_COUNT,
    )
    .await;
    let table1 = gen_iterator_test_sstable_base(
        1,
        default_builder_opt_for_test(),
        |x| (TEST_KEYS_COUNT + x) * 2,
        sstable_store.clone(),
        TEST_KEYS_COUNT,
    )
    .await;
    let mut iter = ConcatIterator::new(
        vec![table0.get_sstable_info(), table1.get_sstable_info()],
        sstable_store,
        Arc::new(ReadOptions::default()),
    );
    iter.rewind().await.unwrap();
    fail::cfg(mem_read_err, "return").unwrap();
    let result = iter.seek(iterator_test_key_of(22).as_slice()).await;
    assert!(result.is_err());
    let result = iter
        .seek(iterator_test_key_of(4 * TEST_KEYS_COUNT).as_slice())
        .await;
    assert!(result.is_err());
    let result = iter.seek(iterator_test_key_of(23).as_slice()).await;
    assert!(result.is_err());
    fail::remove(mem_read_err);
    iter.rewind().await.unwrap();
    fail::cfg(mem_read_err, "return").unwrap();
    let mut i = 0;
    while iter.is_valid() {
        let key = iter.key();
        let val = iter.value();
        assert_eq!(key, iterator_test_key_of(i * 2).as_slice());
        assert_eq!(
            val.into_user_value().unwrap(),
            iterator_test_value_of(i * 2).as_slice()
        );
        i += 1;
        let result = iter.next().await;
        if result.is_err() {
            assert!(i < 2 * TEST_KEYS_COUNT);
            break;
        }
    }
    assert!(i < 2 * TEST_KEYS_COUNT);
    assert!(!iter.is_valid());
    fail::remove(mem_read_err);
}
#[tokio::test]
#[cfg(feature = "failpoints")]
async fn test_failpoints_backward_concat_read_err() {
    fail::cfg("disable_block_cache", "return").unwrap();
    fail::cfg("disable_bloom_filter", "return").unwrap();
    let mem_read_err = "mem_read_err";
    let sstable_store = mock_sstable_store();
    let table0 = gen_iterator_test_sstable_base(
        0,
        default_builder_opt_for_test(),
        |x| x * 2,
        sstable_store.clone(),
        TEST_KEYS_COUNT,
    )
    .await;
    let table1 = gen_iterator_test_sstable_base(
        1,
        default_builder_opt_for_test(),
        |x| (TEST_KEYS_COUNT + x) * 2,
        sstable_store.clone(),
        TEST_KEYS_COUNT,
    )
    .await;
    let mut iter = BackwardConcatIterator::new(
        vec![table1.get_sstable_info(), table0.get_sstable_info()],
        sstable_store.clone(),
        Arc::new(ReadOptions::default()),
    );
    iter.rewind().await.unwrap();
    fail::cfg(mem_read_err, "return").unwrap();
    let result = iter.seek(iterator_test_key_of(2).as_slice()).await;
    assert!(result.is_err());
    let result = iter.seek(iterator_test_key_of(3).as_slice()).await;
    assert!(result.is_err());
    fail::remove(mem_read_err);
    iter.rewind().await.unwrap();
    fail::cfg(mem_read_err, "return").unwrap();
    let mut i = TEST_KEYS_COUNT * 2;
    while iter.is_valid() {
        i -= 1;
        let key = iter.key();
        let val = iter.value();
        assert_eq!(key, iterator_test_key_of(i * 2).as_slice());
        assert_eq!(
            val.into_user_value().unwrap(),
            iterator_test_value_of(i * 2).as_slice()
        );
        let result = iter.next().await;
        if result.is_err() {
            assert!(i > 0);
            break;
        }
    }
    assert!(i > 0);
    assert!(!iter.is_valid());
    fail::remove(mem_read_err);
}
#[tokio::test]
#[cfg(feature = "failpoints")]
async fn test_failpoints_merge_invalid_key() {
    fail::cfg("disable_block_cache", "return").unwrap();
    fail::cfg("disable_bloom_filter", "return").unwrap();
    let mem_read_err = "mem_read_err";
    let sstable_store = mock_sstable_store();
    let table0 = gen_iterator_test_sstable_base(
        0,
        default_builder_opt_for_test(),
        |x| x,
        sstable_store.clone(),
        200,
    )
    .await;
    let table1 = gen_iterator_test_sstable_base(
        1,
        default_builder_opt_for_test(),
        |x| 200 + x,
        sstable_store.clone(),
        200,
    )
    .await;
    let tables = vec![table0, table1];
    let mut mi = MergeIterator::new(
        tables
            .iter()
            .map(|table| -> Box<dyn HummockIterator<Direction = Forward>> {
                Box::new(SSTableIterator::new(
                    block_on(sstable_store.sstable(table.id, &mut StoreLocalStatistic::default()))
                        .unwrap(),
                    sstable_store.clone(),
                    Arc::new(ReadOptions::default()),
                ))
            }),
        Arc::new(StateStoreMetrics::unused()),
    );
    mi.rewind().await.unwrap();
    let mut count = 0;
    fail::cfg(mem_read_err, "return").unwrap();
    while mi.is_valid() {
        count += 1;
        if (mi.next().await).is_err() {
            assert!(count < 200 * 2);
        }
    }
    assert!(count < 200 * 2);
    mi.seek(iterator_test_key_of(350).as_slice()).await.unwrap();
    assert!(!mi.is_valid());
    fail::remove(mem_read_err);
}
#[tokio::test]
#[cfg(feature = "failpoints")]
async fn test_failpoints_backward_merge_invalid_key() {
    fail::cfg("disable_block_cache", "return").unwrap();
    fail::cfg("disable_bloom_filter", "return").unwrap();
    let mem_read_err = "mem_read_err";
    let sstable_store = mock_sstable_store();
    let table0 = gen_iterator_test_sstable_base(
        0,
        default_builder_opt_for_test(),
        |x| x,
        sstable_store.clone(),
        200,
    )
    .await;
    let table1 = gen_iterator_test_sstable_base(
        1,
        default_builder_opt_for_test(),
        |x| 200 + x,
        sstable_store.clone(),
        200,
    )
    .await;
    let tables = vec![table0, table1];
    let mut mi = BackwardMergeIterator::new(
        tables
            .iter()
            .map(|table| -> Box<dyn HummockIterator<Direction = Backward>> {
                Box::new(BackwardSSTableIterator::new(
                    block_on(sstable_store.sstable(table.id, &mut StoreLocalStatistic::default()))
                        .unwrap(),
                    sstable_store.clone(),
                ))
            }),
        Arc::new(StateStoreMetrics::unused()),
    );
    mi.rewind().await.unwrap();
    let mut count = 0;
    fail::cfg(mem_read_err, "return").unwrap();
    while mi.is_valid() {
        count += 1;
        if (mi.next().await).is_err() {
            assert!(count < 200 * 2);
        }
    }
    assert!(count < 200 * 2);
    mi.seek(iterator_test_key_of(10).as_slice()).await.unwrap();
    assert!(!mi.is_valid());
    fail::remove(mem_read_err);
}
#[tokio::test]
#[cfg(feature = "failpoints")]
async fn test_failpoints_user_read_err() {
    fail::cfg("disable_block_cache", "return").unwrap();
    fail::cfg("disable_bloom_filter", "return").unwrap();
    let mem_read_err = "mem_read_err";
    let sstable_store = mock_sstable_store();
    let table0 = gen_iterator_test_sstable_base(
        0,
        default_builder_opt_for_test(),
        |x| x,
        sstable_store.clone(),
        200,
    )
    .await;
    let table1 = gen_iterator_test_sstable_base(
        1,
        default_builder_opt_for_test(),
        |x| 200 + x,
        sstable_store.clone(),
        200,
    )
    .await;
    let mut stats = StoreLocalStatistic::default();
    let iters: Vec<BoxedForwardHummockIterator> = vec![
        Box::new(SSTableIterator::new(
            block_on(sstable_store.sstable(table0.id, &mut stats)).unwrap(),
            sstable_store.clone(),
            Arc::new(ReadOptions::default()),
        )),
        Box::new(SSTableIterator::new(
            block_on(sstable_store.sstable(table1.id, &mut stats)).unwrap(),
            sstable_store.clone(),
            Arc::new(ReadOptions::default()),
        )),
    ];

    let mi = MergeIterator::new(iters, Arc::new(StateStoreMetrics::unused()));
    let mut ui = UserIterator::for_test(mi, (Unbounded, Unbounded));
    ui.rewind().await.unwrap();

    fail::cfg(mem_read_err, "return").unwrap();
    let mut i = 0;
    while ui.is_valid() {
        let key = ui.key();
        let val = ui.value();
        assert_eq!(key, user_key(iterator_test_key_of(i).as_slice()));
        assert_eq!(val, iterator_test_value_of(i).as_slice());
        i += 1;
        let result = ui.next().await;
        if result.is_err() {
            assert!(i < 400);
        }
    }
    assert!(i < 400);
    ui.seek(user_key(iterator_test_key_of(350).as_slice()))
        .await
        .unwrap();
    assert!(!ui.is_valid());
    fail::remove(mem_read_err);
}

#[tokio::test]
#[cfg(feature = "failpoints")]
async fn test_failpoints_backward_user_read_err() {
    fail::cfg("disable_block_cache", "return").unwrap();
    fail::cfg("disable_bloom_filter", "return").unwrap();
    let mem_read_err = "mem_read_err";
    let sstable_store = mock_sstable_store();
    let table0 = gen_iterator_test_sstable_base(
        0,
        default_builder_opt_for_test(),
        |x| x,
        sstable_store.clone(),
        200,
    )
    .await;
    let table1 = gen_iterator_test_sstable_base(
        1,
        default_builder_opt_for_test(),
        |x| 200 + x,
        sstable_store.clone(),
        200,
    )
    .await;
    let mut stats = StoreLocalStatistic::default();
    let iters: Vec<BoxedBackwardHummockIterator> = vec![
        Box::new(BackwardSSTableIterator::new(
            block_on(sstable_store.sstable(table0.id, &mut stats)).unwrap(),
            sstable_store.clone(),
        )),
        Box::new(BackwardSSTableIterator::new(
            block_on(sstable_store.sstable(table1.id, &mut stats)).unwrap(),
            sstable_store.clone(),
        )),
    ];

    let mi = BackwardMergeIterator::new(iters, Arc::new(StateStoreMetrics::unused()));
    let mut ui = BackwardUserIterator::new(mi, (Unbounded, Unbounded));
    ui.rewind().await.unwrap();

    fail::cfg(mem_read_err, "return").unwrap();
    let mut i = 2 * 200;
    while ui.is_valid() {
        i -= 1;
        let key = ui.key();
        let val = ui.value();
        assert_eq!(key, user_key(iterator_test_key_of(i).as_slice()));
        assert_eq!(val, iterator_test_value_of(i).as_slice());
        let result = ui.next().await;
        if result.is_err() {
            assert!(i > 0);
        }
    }
    assert!(i > 0);
    ui.seek(user_key(iterator_test_key_of(10).as_slice()))
        .await
        .unwrap();
    assert!(!ui.is_valid());
    fail::remove(mem_read_err);
}
