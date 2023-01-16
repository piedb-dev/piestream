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

use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;

use piestream_hummock_sdk::key::FullKey;
use piestream_hummock_sdk::HummockEpoch;
use piestream_pb::hummock::SstableInfo;
use tokio::task::JoinHandle;


use crate::hummock::compactor::task_progress::TaskProgress;
use crate::hummock::sstable_store::SstableStoreRef;
use crate::hummock::value::HummockValue;
use crate::hummock::{
    BatchUploadWriter, CachePolicy, HummockResult, MemoryLimiter, SstableBuilder,
    SstableBuilderOptions, SstableWriter, SstableWriterOptions,
};
use crate::monitor::StateStoreMetrics;
use crate::hummock::sstable::builder::TableColumnDescHash;

pub type UploadJoinHandle = JoinHandle<HummockResult<()>>;

#[async_trait::async_trait]
pub trait TableBuilderFactory {
    type Writer: SstableWriter<Output = UploadJoinHandle>;
    async fn open_builder(&self) -> HummockResult<SstableBuilder<Self::Writer>>;
}

pub struct SplitTableOutput {
    pub sst_info: SstableInfo,
    pub upload_join_handle: UploadJoinHandle,
}

/// A wrapper for [`SstableBuilder`] which automatically split key-value pairs into multiple tables,
/// based on their target capacity set in options.
///
/// When building is finished, one may call `finish` to get the results of zero, one or more tables.
pub struct CapacitySplitTableBuilder<F>
where
    F: TableBuilderFactory,
{
    /// When creating a new [`SstableBuilder`], caller use this factory to generate it.
    builder_factory: F,

    sst_outputs: Vec<SplitTableOutput>,

    current_builder: Option<SstableBuilder<F::Writer>>,

    /// Statistics.
    pub stats: Arc<StateStoreMetrics>,

    /// Update the number of sealed Sstables.
    task_progress: Option<Arc<TaskProgress>>,
}

impl<F> CapacitySplitTableBuilder<F>
where
    F: TableBuilderFactory,
{
    /// Creates a new [`CapacitySplitTableBuilder`] using given configuration generator.
    pub fn new(
        builder_factory: F,
        stats: Arc<StateStoreMetrics>,
        task_progress: Option<Arc<TaskProgress>>,
    ) -> Self {
        Self {
            builder_factory,
            sst_outputs: Vec::new(),
            current_builder: None,
            stats,
            task_progress,
        }
    }

    pub fn for_test(builder_factory: F) -> Self {
        Self {
            builder_factory,
            sst_outputs: Vec::new(),
            current_builder: None,
            stats: Arc::new(StateStoreMetrics::unused()),
            task_progress: None,
        }
    }

    /// Returns the number of [`SstableBuilder`]s.
    pub fn len(&self) -> usize {
        self.sst_outputs.len() + if self.current_builder.is_some() { 1 } else { 0 }
    }

    /// Returns true if no builder is created.
    pub fn is_empty(&self) -> bool {
        self.sst_outputs.is_empty() && self.current_builder.is_none()
    }

    /// Adds a user key-value pair to the underlying builders, with given `epoch`.
    ///
    /// If the current builder reaches its capacity, this function will create a new one with the
    /// configuration generated by the closure provided earlier.
    pub async fn add_user_key(
        &mut self,
        user_key: Vec<u8>,
        value: HummockValue<&[u8]>,
        epoch: HummockEpoch,
    ) -> HummockResult<()> {
        assert!(!user_key.is_empty());
        let full_key = FullKey::from_user_key(user_key, epoch);
        self.add_full_key(full_key.as_slice().into_inner(), value, true)
            .await?;
        Ok(())
    }

    /// Adds a key-value pair to the underlying builders.
    ///
    /// If `allow_split` and the current builder reaches its capacity, this function will create a
    /// new one with the configuration generated by the closure provided earlier.
    ///
    /// Note that in some cases like compaction of the same user key, automatic splitting is not
    /// allowed, where `allow_split` should be `false`.
    pub async fn add_full_key(
        &mut self,
        full_key: &[u8],
        value: HummockValue<&[u8]>,
        is_new_user_key: bool,
    ) -> HummockResult<()> {
        if let Some(builder) = self.current_builder.as_ref() {
            if is_new_user_key && builder.reach_capacity() {
                self.seal_current().await?;
            }
        }

        if self.current_builder.is_none() {
            let builder = self.builder_factory.open_builder().await?;
            self.current_builder = Some(builder);
        }

        let builder = self.current_builder.as_mut().unwrap();
        builder.add(full_key, value, is_new_user_key).await?;
        Ok(())
    }

    /// Marks the current builder as sealed. Next call of `add` will always create a new table.
    ///
    /// If there's no builder created, or current one is already sealed before, then this function
    /// will be no-op.
    pub async fn seal_current(&mut self) -> HummockResult<()> {
        if let Some(builder) = self.current_builder.take() {
            let builder_output = builder.finish().await?;

            {
                // report

                if let Some(progress) = &self.task_progress {
                    progress.inc_ssts_sealed();
                }

                if builder_output.bloom_filter_size != 0 {
                    self.stats
                        .sstable_bloom_filter_size
                        .observe(builder_output.bloom_filter_size as _);
                }

                if builder_output.sst_info.file_size != 0 {
                    self.stats
                        .sstable_file_size
                        .observe(builder_output.sst_info.file_size as _);
                }

                if builder_output.avg_key_size != 0 {
                    self.stats
                        .sstable_avg_key_size
                        .observe(builder_output.avg_key_size as _);
                }

                if builder_output.avg_value_size != 0 {
                    self.stats
                        .sstable_avg_value_size
                        .observe(builder_output.avg_value_size as _);
                }
            }

            self.sst_outputs.push(SplitTableOutput {
                upload_join_handle: builder_output.writer_output,
                sst_info: builder_output.sst_info,
            });
        }
        Ok(())
    }

    /// Finalizes all the tables to be ids, blocks and metadata.
    pub async fn finish(mut self) -> HummockResult<Vec<SplitTableOutput>> {
        self.seal_current().await?;
        Ok(self.sst_outputs)
    }
}

/// Used for unit tests and benchmarks.
pub struct LocalTableBuilderFactory {
    next_id: AtomicU64,
    sstable_store: SstableStoreRef,
    options: SstableBuilderOptions,
    policy: CachePolicy,
    limiter: MemoryLimiter,
    table_column_hash: Arc<TableColumnDescHash>,
}

impl LocalTableBuilderFactory {
    pub fn new(
        next_id: u64,
        sstable_store: SstableStoreRef,
        options: SstableBuilderOptions,
        table_column_hash: Arc<TableColumnDescHash>,
    ) -> Self {
        Self {
            next_id: AtomicU64::new(next_id),
            sstable_store,
            options,
            policy: CachePolicy::NotFill,
            limiter: MemoryLimiter::new(1000000),
            table_column_hash: table_column_hash,
        }
    }
}

#[async_trait::async_trait]
impl TableBuilderFactory for LocalTableBuilderFactory {
    type Writer = BatchUploadWriter;

    async fn open_builder(&self) -> HummockResult<SstableBuilder<BatchUploadWriter>> {
        let id = self.next_id.fetch_add(1, SeqCst);
        let tracker = self.limiter.require_memory(1).await.unwrap();
        let writer_options = SstableWriterOptions {
            capacity_hint: Some(self.options.capacity),
            tracker: Some(tracker),
            policy: self.policy,
        };
        let writer = self
            .sstable_store
            .clone()
            .create_sst_writer(id, writer_options);
        let builder = SstableBuilder::for_test(id, writer, self.options.clone(), Some(self.table_column_hash.clone()));

        Ok(builder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hummock::iterator::test_utils::mock_sstable_store;
    use crate::hummock::sstable::utils::CompressionAlgorithm;
    use crate::hummock::test_utils::{get_table_column_hash, default_builder_opt_for_test};
    use crate::hummock::{SstableBuilderOptions, DEFAULT_RESTART_INTERVAL};

    pub fn get_table_and_key_of(key: Vec<u8>) -> Vec<u8> {
        let mut user_key=vec![];
        user_key.push('t' as u8);
        user_key.extend_from_slice(&1_u32.to_be_bytes());
        user_key.extend_from_slice(&key.to_vec().as_slice());
        user_key
    }

    pub fn get_value_of(value: Vec<u8>) -> Vec<u8> {
        let mut v=vec![];
        v.push(1_u8);
        v.extend_from_slice(&(value.len() as u32).to_ne_bytes());
        v.extend_from_slice(&value);
        v
    }

    #[tokio::test]
    async fn test_empty() {
        let block_size = 1 << 10;
        let table_capacity = 4 * block_size;
        let opts = SstableBuilderOptions {
            capacity: table_capacity,
            block_capacity: block_size,
            restart_interval: DEFAULT_RESTART_INTERVAL,
            bloom_false_positive: 0.1,
            compression_algorithm: CompressionAlgorithm::None,
        };
        let builder_factory = LocalTableBuilderFactory::new(1001, mock_sstable_store(), opts, get_table_column_hash());
        let builder = CapacitySplitTableBuilder::for_test(builder_factory);
        let results = builder.finish().await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_lots_of_tables() {
        let block_size = 1 << 10;
        let table_capacity = 4 * block_size;
        let opts = SstableBuilderOptions {
            capacity: table_capacity,
            block_capacity: block_size,
            restart_interval: DEFAULT_RESTART_INTERVAL,
            bloom_false_positive: 0.1,
            compression_algorithm: CompressionAlgorithm::None,
        };
        let builder_factory = LocalTableBuilderFactory::new(1001, mock_sstable_store(), opts, get_table_column_hash());
        let mut builder = CapacitySplitTableBuilder::for_test(builder_factory);

        for i in 0..table_capacity {
            builder
                .add_user_key(
                    get_table_and_key_of(b"key".to_vec()),
                    HummockValue::put(get_value_of(b"value".to_vec()).as_slice()),
                    (table_capacity - i) as u64,
                )
                .await
                .unwrap();
        }

        let results = builder.finish().await.unwrap();
        assert!(results.len() > 1);
    }

    #[tokio::test]
    async fn test_table_seal() {
        let opts = default_builder_opt_for_test();
        let mut builder = CapacitySplitTableBuilder::for_test(LocalTableBuilderFactory::new(
            1001,
            mock_sstable_store(),
            opts,
            get_table_column_hash()
        ));
        let mut epoch = 100;

        macro_rules! add {
            () => {
                epoch -= 1;
                builder
                    .add_user_key(get_table_and_key_of(b"k".to_vec()), HummockValue::put(get_value_of(b"v".to_vec()).as_slice()), epoch)
                    .await
                    .unwrap();
            };
        }

        assert_eq!(builder.len(), 0);
        builder.seal_current().await.unwrap();
        assert_eq!(builder.len(), 0);
        add!();
        assert_eq!(builder.len(), 1);
        add!();
        assert_eq!(builder.len(), 1);
        builder.seal_current().await.unwrap();
        assert_eq!(builder.len(), 1);
        add!();
        assert_eq!(builder.len(), 2);
        builder.seal_current().await.unwrap();
        assert_eq!(builder.len(), 2);
        builder.seal_current().await.unwrap();
        assert_eq!(builder.len(), 2);

        let results = builder.finish().await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_initial_not_allowed_split() {
        let opts = default_builder_opt_for_test();
        let mut builder = CapacitySplitTableBuilder::for_test(LocalTableBuilderFactory::new(
            1001,
            mock_sstable_store(),
            opts,
            get_table_column_hash()
        ));

        builder
            .add_full_key(
                FullKey::from_user_key_slice(get_table_and_key_of(b"k".to_vec()).as_slice(), 233)
                    .as_slice()
                    .into_inner(),
                HummockValue::put(get_value_of(b"v".to_vec()).as_slice()),
                true,
            )
            .await
            .unwrap();
    }
}
