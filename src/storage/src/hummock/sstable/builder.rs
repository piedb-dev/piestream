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
use bytes::BytesMut;
use std::collections::btree_map::{Entry};
use std::collections::{BTreeMap,HashMap,BTreeSet};

use piestream_common::config::StorageConfig;
use piestream_hummock_sdk::filter_key_extractor::{
    FilterKeyExtractorImpl, FullKeyFilterKeyExtractor,
};

use piestream_hummock_sdk::key::key_with_epoch;
use piestream_hummock_sdk::key::{get_table_id, user_key};
use piestream_pb::hummock::SstableInfo;
use piestream_common::catalog::ColumnDesc;

use super::bloom::Bloom;
use super::utils::CompressionAlgorithm;
use super::{
    BlockBuilder, BlockBuilderOptions, BlockMeta, SstableMeta, SstableWriter, DEFAULT_BLOCK_SIZE,
    DEFAULT_ENTRY_SIZE, DEFAULT_RESTART_INTERVAL, VERSION,
};
use crate::hummock::value::HummockValue;
use crate::hummock::HummockResult;

pub type TableColumnDescHash = HashMap<u32, (String, Vec<ColumnDesc>)>;

pub const DEFAULT_BLOCK_ROW: usize = 1024;
pub const DEFAULT_SSTABLE_SIZE: usize = 4 * 1024 * 1024;
pub const DEFAULT_BLOOM_FALSE_POSITIVE: f64 = 0.1;
#[derive(Clone, Debug)]
pub struct SstableBuilderOptions {
    /// Approximate sstable capacity.
    pub capacity: usize,
    /// Approximate block capacity.
    pub block_capacity: usize,
    /// Restart point interval.
    pub restart_interval: usize,
    /// False positive probability of bloom filter.
    pub bloom_false_positive: f64,
    /// Compression algorithm.
    pub compression_algorithm: CompressionAlgorithm,
}

impl From<&StorageConfig> for SstableBuilderOptions {
    fn from(options: &StorageConfig) -> SstableBuilderOptions {
        let capacity = (options.sstable_size_mb as usize) * (1 << 20);
        SstableBuilderOptions {
            capacity,
            block_capacity: (options.block_size_kb as usize) * (1 << 10),
            restart_interval: DEFAULT_RESTART_INTERVAL,
            bloom_false_positive: options.bloom_false_positive,
            compression_algorithm: CompressionAlgorithm::None,
        }
    }
}

impl Default for SstableBuilderOptions {
    fn default() -> Self {
        Self {
            capacity: DEFAULT_SSTABLE_SIZE,
            block_capacity: DEFAULT_BLOCK_SIZE,
            restart_interval: DEFAULT_RESTART_INTERVAL,
            bloom_false_positive: DEFAULT_BLOOM_FALSE_POSITIVE,
            compression_algorithm: CompressionAlgorithm::None,
        }
    }
}

pub struct SstableBuilderOutput<WO> {
    pub sst_info: SstableInfo,
    pub bloom_filter_size: usize,
    pub writer_output: WO,
    pub avg_key_size: usize,
    pub avg_value_size: usize,
}

pub struct SstableBuilder<W: SstableWriter> {
    /// Options.
    options: SstableBuilderOptions,
    /// Data writer.
    writer: W,
    table_column_hash: Arc<TableColumnDescHash>,
    /// Current block builder.
    block_builder: BlockBuilder,
    /// Block metadata vec.
    block_metas: Vec<BlockMeta>,
    /// `table_id` of added keys.
    table_ids: BTreeSet<u32>,
    table_ids_key_map: BTreeMap<u32, (Vec<u8>, Vec<u8>)>,
    last_table_id: u32,
    /// Hashes of user keys.
    user_key_hashes: Vec<u32>,
    last_full_key: Vec<u8>,
    key_count: usize,
    need_to_build_size: usize,
    current_block_key_count: usize,
    sstable_id: u64,
    raw_value: BytesMut,
    filter_key_extractor: Arc<FilterKeyExtractorImpl>,
    last_bloom_filter_key_length: usize,

    total_key_size: usize,
    total_value_size: usize,
    stale_key_count: u64,
    total_key_count: u64,
    
    
}

impl<W: SstableWriter> SstableBuilder<W> {
    pub fn for_test(
        sstable_id: u64, 
        writer: W, 
        options: SstableBuilderOptions,
        table_column_hash: Option<Arc<TableColumnDescHash>>,) -> Self {
        Self::new(
            sstable_id,
            writer,
            options,
            Arc::new(FilterKeyExtractorImpl::FullKey(
                FullKeyFilterKeyExtractor::default(),
            )),
            table_column_hash.unwrap().clone(),
        )
    }

    pub fn new(
        sstable_id: u64,
        writer: W,
        options: SstableBuilderOptions,
        filter_key_extractor: Arc<FilterKeyExtractorImpl>,
        table_column_hash: Arc<TableColumnDescHash>,
    ) -> Self {
        Self {
            options: options.clone(),
            writer,
            table_column_hash: table_column_hash.clone(),
            block_builder: BlockBuilder::new(BlockBuilderOptions {
                capacity: options.block_capacity*2,
                restart_interval: options.restart_interval,
                compression_algorithm: options.compression_algorithm,
            }
            ),
            block_metas: Vec::with_capacity(options.capacity / options.block_capacity + 1),
            table_ids: BTreeSet::new(),
            table_ids_key_map: BTreeMap::new(),
            user_key_hashes: Vec::with_capacity(options.capacity / DEFAULT_ENTRY_SIZE + 1),
            last_table_id: 0,
            raw_value: BytesMut::new(),
            last_full_key: vec![],
            key_count: 0,
            need_to_build_size: ((options.block_capacity as f32)*1.2).round() as usize,
            current_block_key_count: 0,
            sstable_id,
            filter_key_extractor,
            last_bloom_filter_key_length: 0,
            total_key_size: 0,
            total_value_size: 0,
            stale_key_count: 0,
            total_key_count: 0,
        }
    }

    
   /// Add kv pair to sstable.
   pub async fn add(
    &mut self,
    full_key: &[u8],
    value: HummockValue<&[u8]>,
    is_new_user_key: bool,
) -> HummockResult<()> {
    println!("-------value={:?}", &value);
    // Rotate block builder if the previous one has been built.
    if self.block_builder.is_empty() {
        self.block_metas.push(BlockMeta {
            offset: self.writer.data_len() as u32,
            len: 0,
            smallest_key: full_key.to_vec(),
            uncompressed_size: 0,
            table_id: 0,
        })
    }

    //println!("full_key={:?}", &full_key);
    // TODO: refine me
    value.encode(&mut self.raw_value);
    if is_new_user_key {
        let mut extract_key = user_key(full_key);
        if let Some(table_id) = get_table_id(full_key) {
            if self.last_table_id != table_id {
                self.table_ids.insert(table_id);
                 //a block only saves one table data
                if self.last_table_id!=0{
                    println!("last_table_id change build_block current_block_key_count={:?}", self.current_block_key_count);
                    self.build_block().await?;
                    self.block_metas.push(BlockMeta {
                        offset: self.writer.data_len() as u32,
                        len: 0,
                        smallest_key: full_key.to_vec(),
                        uncompressed_size: 0,
                        table_id: 0,
                    })
                }
                self.last_table_id = table_id;
                
                //get the current table column data type
                let current_table_info=match self.table_column_hash.get(&table_id){
                    Some(table_desc)=>{
                        let table_data_type_list=table_desc.1
                                .iter()
                                .map(|column_desc| column_desc.data_type.clone())
                                .collect();
                        tracing::trace!(
                                    "Start building the data block of table_id: {:?} table_name {:?} ", 
                                    table_id, table_desc.0);
                        table_data_type_list
                    }
                    None=>{
                        panic!("Failed to not found description of the table={:?}.", table_id);}
                };
                println!("current_table_info={:?}", current_table_info);
                self.block_builder.set_row_deserializer(current_table_info);
                
                //set table last full key
                match self.table_ids_key_map.entry(self.last_table_id){
                    Entry::Occupied( mut value)=>{
                        let block_last_full_key=&mut value.get_mut().1;
                        block_last_full_key.clear();
                        block_last_full_key.extend(self.last_full_key.iter())
                    }
                    Entry::Vacant(_)=>{
                        //set table first full key
                        self.table_ids_key_map.insert(table_id, (full_key.to_vec(), Vec::new()));
                        //panic!("Failed to not found table_id={:?} mapping when building block ends.", self.last_table_id)
                    }
                };
                self.current_block_key_count=0;
            }
        }
        extract_key = self.filter_key_extractor.extract(extract_key);

        // add bloom_filter check
        // 1. not empty_key
        // 2. extract_key key is not duplicate
        if !extract_key.is_empty()
            && (extract_key != &self.last_full_key[0..self.last_bloom_filter_key_length])
        {
            // avoid duplicate add to bloom filter
            self.user_key_hashes
                .push(farmhash::fingerprint32(extract_key));
            self.last_bloom_filter_key_length = extract_key.len();
        }
    } else {
        self.stale_key_count += 1;
    }
    self.total_key_count += 1;

    self.block_builder.add(full_key, self.raw_value.as_ref());
    self.current_block_key_count += 1;
    self.total_key_size += full_key.len();
    self.total_value_size += self.raw_value.len();
    self.raw_value.clear();

    self.last_full_key.clear();
    self.last_full_key.extend_from_slice(full_key);

    //the same key is stored in a block
    if is_new_user_key 
        && (self.current_block_key_count%DEFAULT_BLOCK_ROW==0  || 
        self.block_builder.approximate_len() >= self.options.block_capacity) {
        //println!("current_block_key_count={:?} approximate_len={:?}", self.current_block_key_count, self.block_builder.approximate_len());
        self.build_block().await?;
    }else if self.block_builder.approximate_len() >= self.need_to_build_size{
        //println!("2222222222222");
        self.build_block().await?;
    }
    self.key_count += 1;

    Ok(())
}

    /// Finish building sst.
    ///
    /// Unlike most LSM-Tree implementations, sstable meta and data are encoded separately.
    /// Both meta and data has its own object (file).
    ///
    /// # Format
    ///
    /// data:
    ///
    /// ```plain
    /// | Block 0 | ... | Block N-1 | N (4B) |
    /// ```
    pub async fn finish(mut self) -> HummockResult<SstableBuilderOutput<W::Output>> {
        println!("**********SstableBuilder finish.");
        let smallest_key = self.block_metas[0].smallest_key.clone();
        let largest_key = self.last_full_key.clone();

        self.build_block().await?;
        let meta_offset = self.writer.data_len() as u64;
        assert!(!smallest_key.is_empty());

        let mut meta = SstableMeta {
            block_metas: self.block_metas,
            bloom_filter: if self.options.bloom_false_positive > 0.0 {
                let bits_per_key = Bloom::bloom_bits_per_key(
                    self.user_key_hashes.len(),
                    self.options.bloom_false_positive,
                );
                Bloom::build_from_key_hashes(&self.user_key_hashes, bits_per_key)
            } else {
                vec![]
            },
            estimated_size: 0,
            key_count: self.key_count as u32,
            smallest_key,
            largest_key,
            version: VERSION,
            meta_offset,
        };
        meta.estimated_size = meta.encoded_size() as u32 + meta_offset as u32;
        let sst_info = SstableInfo {
            id: self.sstable_id,
            key_range: Some(piestream_pb::hummock::KeyRange {
                left: meta.smallest_key.clone(),
                right: meta.largest_key.clone(),
                inf: false,
            }),
            file_size: meta.estimated_size as u64,
            table_ids: self.table_ids.into_iter().collect(),
            meta_offset: meta.meta_offset,
            stale_key_count: self.stale_key_count,
            total_key_count: self.total_key_count,
        };
        tracing::trace!(
            "meta_size {} bloom_filter_size {}  add_key_counts {} ",
            meta.encoded_size(),
            meta.bloom_filter.len(),
            self.key_count,
        );
        let bloom_filter_size = meta.bloom_filter.len();
        let avg_key_size = self.total_key_size / self.key_count;
        let avg_value_size = self.total_value_size / self.key_count;

        let writer_output = self.writer.finish(meta).await?;
        Ok(SstableBuilderOutput::<W::Output> {
            sst_info,
            bloom_filter_size,
            writer_output,
            avg_key_size,
            avg_value_size,
        })
    }

    pub fn approximate_len(&self) -> usize {
        self.writer.data_len()
            + self.block_builder.approximate_len()
            + self.user_key_hashes.len() * 4
    }

    async fn build_block(&mut self) -> HummockResult<()> {
        // Skip empty block.
        if self.block_builder.is_empty() {
            return Ok(());
        }

        println!("self.block_metas.len={:?}", self.block_metas.len());
        let mut block_meta = self.block_metas.last_mut().unwrap();
        //block_meta.uncompressed_size = self.block_builder.uncompressed_block_size() as u32;
        let (uncompressed_size, block) = self.block_builder.build();
        block_meta.uncompressed_size = uncompressed_size;
        self.writer.write_block(block, block_meta).await?;
        //let block_build=&self.block_builder;
        //block_meta.uncompressed_size = block_build.uncompressed_block_size() as u32;
        block_meta.len = self.writer.data_len() as u32 - block_meta.offset;
        block_meta.table_id=self.last_table_id;
        println!("write to block_meta={:?}",  block_meta);
        self.block_builder.clear();
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.user_key_hashes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.user_key_hashes.is_empty()
    }

    /// Returns true if we roughly reached capacity
    pub fn reach_capacity(&self) -> bool {
        self.approximate_len() >= self.options.capacity
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::hummock::iterator::test_utils::mock_sstable_store;
    use crate::hummock::test_utils::{
        default_builder_opt_for_test, new_gen_default_test_sstable, mock_sst_writer,
        get_table_column_hash, test_table_and_key_of, new_test_value_of, TEST_KEYS_COUNT,
    };

    #[tokio::test]
    #[should_panic]
    async fn test_empty() {
        let opt = SstableBuilderOptions {
            capacity: 0,
            block_capacity: 100*4096,
            restart_interval: 16,
            bloom_false_positive: 0.1,
            compression_algorithm: CompressionAlgorithm::None,
        };

        let b = SstableBuilder::for_test(0, mock_sst_writer(&opt), opt, None);

        b.finish().await.unwrap();
    }

    /*fn get_table_column_hash()->Option<Arc<TableColumnDescHash>>{
        use piestream_common::types::DataType;

        let columns = vec![
            //ColumnDesc::new_atomic(DataType::Int32, "age", 1),
            ColumnDesc::new_atomic(DataType::Varchar, "name", 0),
        ];
        let mut mapping: HashMap<u32, (String, Vec<ColumnDesc>)> = HashMap::new();
        mapping.insert(1, ("school".to_string(), columns.clone()));
        mapping.insert(2, ("city".to_string(), columns));
        println!("mapping={:?}", mapping);
        Some(Arc::new(mapping))
    }

    /// The key (with epoch 0) of an index in the test table
    pub fn test_table_and_key_of(idx: usize) -> Vec<u8> {
        let mut user_key=vec![];
        user_key.push('t' as u8);
        if idx<TEST_KEYS_COUNT/2{
            user_key.extend_from_slice(&1_u32.to_be_bytes());
        }else{
            user_key.extend_from_slice(&2_u32.to_be_bytes());
        }
        let  key = format!("{:05}", idx).as_bytes().to_vec();
        let key_with_epoch=key_with_epoch(key, 233);
        user_key.extend_from_slice(&key_with_epoch.to_vec().as_slice());
        //println!("user_key={:?}", user_key);
        user_key
    }

    pub fn new_test_value_of(idx: usize) -> Vec<u8> {
        let  value = &b"666666"[..];
        let mut v=vec![];
        v.push(1_u8);
        v.extend_from_slice(&(value.len() as u32).to_ne_bytes());
        v.extend_from_slice(&value);
        v
    }*/

    #[tokio::test]
    async fn new_test_basic() {
        let opt = default_builder_opt_for_test();
        let mut b = SstableBuilder::for_test(0, mock_sst_writer(&opt), opt, get_table_column_hash());

        for i in 0..TEST_KEYS_COUNT {
            b.add(&test_table_and_key_of(i), HummockValue::put(&new_test_value_of(i)), true)
                .await
                .unwrap();
        }

        let output = b.finish().await.unwrap();
        let info = output.sst_info;
        println!("info={:?}", info);

        assert_eq!(test_table_and_key_of(0), info.key_range.as_ref().unwrap().left);
        assert_eq!(
            test_table_and_key_of(TEST_KEYS_COUNT - 1),
            info.key_range.as_ref().unwrap().right
        );
        let (data, meta) = output.writer_output;
        assert_eq!(info.file_size, meta.estimated_size as u64);
        let offset = info.meta_offset as usize;
        let meta2 = SstableMeta::decode(&mut &data[offset..]).unwrap();
        assert_eq!(meta2, meta);
    }
    #[tokio::test]
    async fn new_test_one_record() {
        let opt = default_builder_opt_for_test();
        let mut b = SstableBuilder::for_test(0, mock_sst_writer(&opt), opt, get_table_column_hash());

        let max=1;
        for i in 0..max {
            b.add(&test_table_and_key_of(i), HummockValue::put(&new_test_value_of(i)), true)
                .await
                .unwrap();
        }

        let output = b.finish().await.unwrap();
        let info = output.sst_info;
        println!("info={:?}", info);

        assert_eq!(test_table_and_key_of(0), info.key_range.as_ref().unwrap().left);
        assert_eq!(
            test_table_and_key_of(max - 1),
            info.key_range.as_ref().unwrap().right
        );
        let (data, meta) = output.writer_output;
        assert_eq!(info.file_size, meta.estimated_size as u64);
        let offset = info.meta_offset as usize;
        let meta2 = SstableMeta::decode(&mut &data[offset..]).unwrap();
        assert_eq!(meta2, meta);
    }

    /*#[tokio::test]
    async fn test_basic() {
        let opt = default_builder_opt_for_test();
        let mut b = SstableBuilder::for_test(0, mock_sst_writer(&opt), opt, get_table_column_hash());

        for i in 0..TEST_KEYS_COUNT {
            b.add(&test_key_of(i), HummockValue::put(&test_value_of(i)), true)
                .await
                .unwrap();
        }

        let output = b.finish().await.unwrap();
        let info = output.sst_info;

        assert_eq!(test_key_of(0), info.key_range.as_ref().unwrap().left);
        assert_eq!(
            test_key_of(TEST_KEYS_COUNT - 1),
            info.key_range.as_ref().unwrap().right
        );
        let (data, meta) = output.writer_output;
        assert_eq!(info.file_size, meta.estimated_size as u64);
        let offset = info.meta_offset as usize;
        let meta2 = SstableMeta::decode(&mut &data[offset..]).unwrap();
        assert_eq!(meta2, meta);
    }*/

    async fn test_with_bloom_filter(with_blooms: bool) {
        let key_count = 1000;

        let opts = SstableBuilderOptions {
            capacity: 0,
            block_capacity: 4096,
            restart_interval: 16,
            bloom_false_positive: if with_blooms { 0.01 } else { 0.0 },
            compression_algorithm: CompressionAlgorithm::None,
        };

        // build remote table
        let sstable_store = mock_sstable_store();
        let table = new_gen_default_test_sstable(opts, 0, sstable_store).await;

        assert_eq!(table.has_bloom_filter(), with_blooms);
        for i in 0..key_count {
            let full_key = test_table_and_key_of(i);
            assert!(!table.surely_not_have_user_key(user_key(full_key.as_slice())));
        }
    }

    #[tokio::test]
    async fn test_bloom_filter() {
        test_with_bloom_filter(false).await;
        test_with_bloom_filter(true).await;
    }
}
