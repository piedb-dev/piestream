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

use std::ops::Range;
use std::sync::Arc;

use criterion::async_executor::FuturesExecutor;
use criterion::{criterion_group, criterion_main, Criterion};
use piestream_hummock_sdk::key::key_with_epoch;
use piestream_hummock_sdk::key_range::KeyRange;
use piestream_object_store::object::object_metrics::ObjectStoreMetrics;
use piestream_object_store::object::{InMemObjectStore, ObjectStore, ObjectStoreImpl};
use piestream_pb::hummock::SstableInfo;
use piestream_storage::hummock::compactor::{
    Compactor, ConcatSstableIterator, DummyCompactionFilter, TaskConfig,
};
use piestream_storage::hummock::iterator::{
    ConcatIterator, Forward, HummockIterator, HummockIteratorUnion, MultiSstIterator,
    UnorderedMergeIteratorInner,
};
use piestream_storage::hummock::multi_builder::{
    CapacitySplitTableBuilder, LocalTableBuilderFactory,
};
use piestream_storage::hummock::sstable::SstableIteratorReadOptions;
use piestream_storage::hummock::sstable_store::SstableStoreRef;
use piestream_storage::hummock::value::HummockValue;
use piestream_storage::hummock::{
    CachePolicy, CompactorSstableStore, CompressionAlgorithm, MemoryLimiter, SstableBuilder,
    SstableBuilderOptions, SstableIterator, SstableStore, SstableWriterOptions, TieredCache,
};
use piestream_storage::monitor::{StateStoreMetrics, StoreLocalStatistic};

pub fn mock_sstable_store() -> SstableStoreRef {
    let store = InMemObjectStore::new().monitored(Arc::new(ObjectStoreMetrics::unused()));
    let store = Arc::new(ObjectStoreImpl::InMem(store));
    let path = "test".to_string();
    Arc::new(SstableStore::new(
        store,
        path,
        64 << 20,
        128 << 20,
        TieredCache::none(),
    ))
}

pub fn default_writer_opts() -> SstableWriterOptions {
    SstableWriterOptions {
        capacity_hint: None,
        tracker: None,
        policy: CachePolicy::Fill,
    }
}

pub fn test_key_of(idx: usize, epoch: u64) -> Vec<u8> {
    let user_key = format!("key_test_{:08}", idx * 2).as_bytes().to_vec();
    key_with_epoch(user_key, epoch)
}

const MAX_KEY_COUNT: usize = 128 * 1024;

async fn build_table(
    sstable_store: SstableStoreRef,
    sstable_id: u64,
    range: Range<u64>,
    epoch: u64,
) -> SstableInfo {
    let opt = SstableBuilderOptions {
        capacity: 32 * 1024 * 1024,
        block_capacity: 16 * 1024,
        restart_interval: 16,
        bloom_false_positive: 0.01,
        compression_algorithm: CompressionAlgorithm::None,
    };
    let writer = sstable_store.create_sst_writer(
        sstable_id,
        SstableWriterOptions {
            capacity_hint: None,
            tracker: None,
            policy: CachePolicy::Fill,
        },
    );
    let mut builder = SstableBuilder::for_test(sstable_id, writer, opt);
    let value = b"1234567890123456789";
    let mut full_key = test_key_of(0, epoch);
    let user_len = full_key.len() - 8;
    for i in range {
        let start = (i % 8) as usize;
        let end = (start + 8) as usize;
        full_key[(user_len - 8)..user_len].copy_from_slice(&i.to_be_bytes());
        builder
            .add(&full_key, HummockValue::put(&value[start..end]), true)
            .await
            .unwrap();
    }
    let output = builder.finish().await.unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let handle = output.writer_output;
    let sst = output.sst_info;
    runtime.block_on(handle).unwrap().unwrap();
    sst
}

async fn scan_all_table(info: &SstableInfo, sstable_store: SstableStoreRef) {
    let mut stats = StoreLocalStatistic::default();
    let table = sstable_store.sstable(info, &mut stats).await.unwrap();
    let default_read_options = Arc::new(SstableIteratorReadOptions::default());
    // warm up to make them all in memory. I do not use CachePolicy::Fill because it will fetch
    // block from meta.
    let mut iter = SstableIterator::new(table, sstable_store.clone(), default_read_options);
    iter.rewind().await.unwrap();
    while iter.is_valid() {
        iter.next().await.unwrap();
    }
}

fn bench_table_build(c: &mut Criterion) {
    c.bench_function("bench_table_build", |b| {
        let sstable_store = mock_sstable_store();
        b.iter(|| {
            let _ = build_table(sstable_store.clone(), 0, 0..(MAX_KEY_COUNT as u64), 1);
        });
    });
}

fn bench_table_scan(c: &mut Criterion) {
    let sstable_store = mock_sstable_store();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let info = runtime.block_on(async {
        build_table(sstable_store.clone(), 0, 0..(MAX_KEY_COUNT as u64), 1).await
    });
    // warm up to make them all in memory. I do not use CachePolicy::Fill because it will fetch
    // block from meta.
    let sstable_store1 = sstable_store.clone();
    let info1 = info.clone();
    runtime.block_on(async move {
        scan_all_table(&info1, sstable_store1).await;
    });

    c.bench_function("bench_table_iterator", |b| {
        let info1 = info.clone();
        b.to_async(FuturesExecutor)
            .iter(|| scan_all_table(&info1, sstable_store.clone()));
    });
}

async fn compact<I: HummockIterator<Direction = Forward>>(iter: I, sstable_store: SstableStoreRef) {
    let opt = SstableBuilderOptions {
        capacity: 32 * 1024 * 1024,
        block_capacity: 64 * 1024,
        restart_interval: 16,
        bloom_false_positive: 0.01,
        compression_algorithm: CompressionAlgorithm::None,
    };
    let mut builder =
        CapacitySplitTableBuilder::for_test(LocalTableBuilderFactory::new(32, sstable_store, opt));

    let task_config = TaskConfig {
        key_range: KeyRange::inf(),
        cache_policy: CachePolicy::Disable,
        gc_delete_keys: false,
        watermark: 0,
    };
    Compactor::compact_and_build_sst(
        &mut builder,
        &task_config,
        Arc::new(StateStoreMetrics::unused()),
        iter,
        DummyCompactionFilter,
    )
    .await
    .unwrap();
}

fn bench_merge_iterator_compactor(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let sstable_store = mock_sstable_store();
    let test_key_size = 256 * 1024;
    let info1 = runtime
        .block_on(async { build_table(sstable_store.clone(), 1, 0..test_key_size, 1).await });
    let info2 = runtime
        .block_on(async { build_table(sstable_store.clone(), 2, 0..test_key_size, 1).await });
    let level1 = vec![info1, info2];

    let info1 = runtime
        .block_on(async { build_table(sstable_store.clone(), 3, 0..test_key_size, 2).await });
    let info2 = runtime
        .block_on(async { build_table(sstable_store.clone(), 4, 0..test_key_size, 2).await });
    let level2 = vec![info1, info2];
    let read_options = Arc::new(SstableIteratorReadOptions { prefetch: true });
    c.bench_function("bench_union_merge_iterator", |b| {
        b.to_async(FuturesExecutor).iter(|| {
            let sstable_store1 = sstable_store.clone();
            let sub_iters = vec![
                HummockIteratorUnion::First(ConcatIterator::new(
                    level1.clone(),
                    sstable_store.clone(),
                    read_options.clone(),
                )),
                HummockIteratorUnion::First(ConcatIterator::new(
                    level2.clone(),
                    sstable_store.clone(),
                    read_options.clone(),
                )),
            ];
            let iter = MultiSstIterator::for_compactor(sub_iters);
            async move { compact(iter, sstable_store1).await }
        });
    });
    let compact_store = Arc::new(CompactorSstableStore::new(
        sstable_store.clone(),
        MemoryLimiter::unlimit(),
    ));
    c.bench_function("bench_merge_iterator", |b| {
        b.to_async(&runtime).iter(|| {
            let sub_iters = vec![
                ConcatSstableIterator::new(level1.clone(), KeyRange::inf(), compact_store.clone()),
                ConcatSstableIterator::new(level2.clone(), KeyRange::inf(), compact_store.clone()),
            ];
            let iter = UnorderedMergeIteratorInner::for_compactor(sub_iters);
            let sstable_store1 = sstable_store.clone();
            async move { compact(iter, sstable_store1).await }
        });
    });
}

criterion_group!(
    benches,
    bench_table_build,
    bench_table_scan,
    bench_merge_iterator_compactor
);
criterion_main!(benches);
