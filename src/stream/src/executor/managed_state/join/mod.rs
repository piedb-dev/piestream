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

mod iter_utils;
mod join_entry_state;

use std::alloc::Global;
use std::ops::{Deref, DerefMut, Index};
use std::sync::Arc;

use anyhow::Context;
use fixedbitset::FixedBitSet;
use futures::future::try_join;
use futures_async_stream::for_await;
use itertools::Itertools;
pub(super) use join_entry_state::JoinEntryState;
use local_stats_alloc::{SharedStatsAlloc, StatsAlloc};
use piestream_common::array::{Row, RowDeserializer};
use piestream_common::bail;
use piestream_common::buffer::Bitmap;
use piestream_common::collection::estimate_size::EstimateSize;
use piestream_common::hash::{HashKey, PrecomputedBuildHasher};
use piestream_common::types::{DataType, Datum, ScalarImpl};
use piestream_common::util::epoch::EpochPair;
use piestream_common::util::ordered::OrderedRowSerde;
use piestream_common::util::sort_util::OrderType;
use piestream_storage::table::streaming_table::state_table::StateTable;
use piestream_storage::StateStore;

use self::iter_utils::zip_by_order_key;
use crate::cache::{EvictableHashMap, ExecutorCache, LruManagerRef, ManagedLruCache};
use crate::executor::error::StreamExecutorResult;
use crate::executor::monitor::StreamingMetrics;
use crate::task::ActorId;

type DegreeType = u64;

pub fn build_degree_row(mut order_key: Row, degree: DegreeType) -> Row {
    let degree_datum = Some(ScalarImpl::Int64(degree as i64));
    order_key.0.push(degree_datum);
    order_key
}

/// This is a row with a match degree
#[derive(Clone, Debug)]
pub struct JoinRow {
    pub row: Row,
    degree: DegreeType,
}

impl Index<usize> for JoinRow {
    type Output = Datum;

    fn index(&self, index: usize) -> &Self::Output {
        &self.row[index]
    }
}

impl JoinRow {
    pub fn new(row: Row, degree: DegreeType) -> Self {
        Self { row, degree }
    }

    pub fn is_zero_degree(&self) -> bool {
        self.degree == 0
    }

    pub fn inc_degree(&mut self) -> DegreeType {
        self.degree += 1;
        self.degree
    }

    pub fn dec_degree(&mut self) -> StreamExecutorResult<DegreeType> {
        if self.degree == 0 {
            bail!("Tried to decrement zero join row degree");
        }
        self.degree -= 1;
        Ok(self.degree)
    }

    /// Return row and degree in `Row` format. The degree part will be inserted in degree table
    /// later, so a pk prefix will be added.
    ///
    /// * `state_order_key_indices` - the order key of `row`
    pub fn into_table_rows(self, state_order_key_indices: &[usize]) -> (Row, Row) {
        let order_key = self.row.by_indices(state_order_key_indices);
        let degree = build_degree_row(order_key, self.degree);
        (self.row, degree)
    }

    pub fn encode(&self) -> EncodedJoinRow {
        let value_indices = (0..self.row.0.len()).collect_vec();
        EncodedJoinRow {
            row: self.row.serialize(&value_indices),
            degree: self.degree,
        }
    }
}

#[derive(Clone, Debug)]
pub struct EncodedJoinRow {
    pub row: Vec<u8>,
    degree: DegreeType,
}

impl EncodedJoinRow {
    fn decode(&self, data_types: &[DataType]) -> StreamExecutorResult<JoinRow> {
        let deserializer = RowDeserializer::new(data_types.to_vec());
        let row = deserializer.deserialize(self.row.as_ref())?;
        Ok(JoinRow {
            row,
            degree: self.degree,
        })
    }

    fn decode_row(&self, data_types: &[DataType]) -> StreamExecutorResult<Row> {
        let deserializer = RowDeserializer::new(data_types.to_vec());
        let row = deserializer.deserialize(self.row.as_ref())?;
        Ok(row)
    }

    pub fn inc_degree(&mut self) -> DegreeType {
        self.degree += 1;
        self.degree
    }

    pub fn dec_degree(&mut self) -> StreamExecutorResult<DegreeType> {
        if self.degree == 0 {
            bail!("Tried to decrement zero join row degree");
        }
        self.degree -= 1;
        Ok(self.degree)
    }

    // TODO(yuhao): only need to decode part of the encoded row.
    // TODO(yuhao): add kv api in state table to avoid manually append pk prefix.
    /// Get a row with the schema in degree state table
    ///
    /// * `state_order_key_indices` - the order key of `row`
    pub fn get_schemaed_degree(
        &self,
        row_data_types: &[DataType],
        state_order_key_indices: &[usize],
    ) -> StreamExecutorResult<Row> {
        let order_key = self
            .decode_row(row_data_types)?
            .by_indices(state_order_key_indices);
        let schemaed_degree = build_degree_row(order_key, self.degree);
        Ok(schemaed_degree)
    }
}

impl EstimateSize for EncodedJoinRow {
    fn estimated_heap_size(&self) -> usize {
        self.row.estimated_heap_size()
    }
}

/// Memcomparable encoding.
type PkType = Vec<u8>;

pub type StateValueType = EncodedJoinRow;
pub type HashValueType = JoinEntryState;

type JoinHashMapInner<K> =
    ExecutorCache<K, HashValueType, PrecomputedBuildHasher, SharedStatsAlloc<Global>>;

pub type JoinManagedCache<K> =
    ManagedLruCache<K, HashValueType, PrecomputedBuildHasher, SharedStatsAlloc<Global>>;

pub struct JoinHashMapMetrics {
    /// Metrics used by join executor
    metrics: Arc<StreamingMetrics>,
    /// Basic information
    actor_id: String,
    side: &'static str,
    /// How many times have we hit the cache of join executor
    lookup_miss_count: usize,
    total_lookup_count: usize,
}

impl JoinHashMapMetrics {
    pub fn new(metrics: Arc<StreamingMetrics>, actor_id: ActorId, side: &'static str) -> Self {
        Self {
            metrics,
            actor_id: actor_id.to_string(),
            side,
            lookup_miss_count: 0,
            total_lookup_count: 0,
        }
    }

    pub fn flush(&mut self) {
        self.metrics
            .join_lookup_miss_count
            .with_label_values(&[&self.actor_id, self.side])
            .inc_by(self.lookup_miss_count as u64);
        self.metrics
            .join_total_lookup_count
            .with_label_values(&[&self.actor_id, self.side])
            .inc_by(self.total_lookup_count as u64);
        self.total_lookup_count = 0;
        self.lookup_miss_count = 0;
    }
}

pub struct JoinHashMap<K: HashKey, S: StateStore> {
    /// Store the join states.
    // SAFETY: This is a self-referential data structure and the allocator is owned by the struct
    // itself. Use the field is safe iff the struct is constructed with [`moveit`](https://crates.io/crates/moveit)'s way.
    inner: JoinHashMapInner<K>,
    /// Data types of the join key columns
    join_key_data_types: Vec<DataType>,
    /// Null safe bitmap for each join pair
    null_matched: FixedBitSet,
    /// The memcomparable serializer of primary key.
    pk_serializer: OrderedRowSerde,
    /// State table. Contains the data from upstream.
    state: TableInner<S>,
    /// Degree table.
    ///
    /// The degree is generated from the hash join executor.
    /// Each row in `state` has a corresponding degree in `degree state`.
    /// A degree value `d` in for a row means the row has `d` matched row in the other join side.
    ///
    /// It will only be used when needed in a side.
    ///
    /// - Full Outer: both side
    /// - Left Outer/Semi/Anti: left side
    /// - Right Outer/Semi/Anti: right side
    /// - Inner: None.
    degree_state: TableInner<S>,
    /// If degree table is need
    need_degree_table: bool,
    /// Metrics of the hash map
    metrics: JoinHashMapMetrics,
}

struct TableInner<S: StateStore> {
    pk_indices: Vec<usize>,
    // This should be identical to the pk in state table.
    order_key_indices: Vec<usize>,
    // This should be identical to the data types in table schema.
    all_data_types: Vec<DataType>,
    pub(crate) table: StateTable<S>,
}

impl<K: HashKey, S: StateStore> JoinHashMap<K, S> {
    /// Create a [`JoinHashMap`] with the given LRU capacity.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        lru_manager: Option<LruManagerRef>,
        cache_size: usize,
        join_key_data_types: Vec<DataType>,
        state_all_data_types: Vec<DataType>,
        state_table: StateTable<S>,
        state_pk_indices: Vec<usize>,
        degree_all_data_types: Vec<DataType>,
        degree_table: StateTable<S>,
        degree_pk_indices: Vec<usize>,
        null_matched: FixedBitSet,
        need_degree_table: bool,
        metrics: Arc<StreamingMetrics>,
        actor_id: ActorId,
        side: &'static str,
    ) -> Self {
        let alloc = StatsAlloc::new(Global).shared();
        // TODO: unify pk encoding with state table.
        let pk_data_types = state_pk_indices
            .iter()
            .map(|i| state_all_data_types[*i].clone())
            .collect();
        let pk_serializer = OrderedRowSerde::new(
            pk_data_types,
            vec![OrderType::Ascending; state_pk_indices.len()],
        );

        let state = TableInner {
            pk_indices: state_pk_indices,
            order_key_indices: state_table.pk_indices().to_vec(),
            all_data_types: state_all_data_types,
            table: state_table,
        };

        let degree_state = TableInner {
            pk_indices: degree_pk_indices,
            order_key_indices: degree_table.pk_indices().to_vec(),
            all_data_types: degree_all_data_types,
            table: degree_table,
        };

        let cache = if let Some(lru_manager) = lru_manager {
            ExecutorCache::Managed(
                lru_manager.create_cache_with_hasher_in(PrecomputedBuildHasher, alloc),
            )
        } else {
            ExecutorCache::Local(EvictableHashMap::with_hasher_in(
                cache_size,
                PrecomputedBuildHasher,
                alloc,
            ))
        };

        Self {
            inner: cache,
            join_key_data_types,
            null_matched,
            pk_serializer,
            state,
            degree_state,
            need_degree_table,
            metrics: JoinHashMapMetrics::new(metrics, actor_id, side),
        }
    }

    pub fn init(&mut self, epoch: EpochPair) {
        self.update_epoch(epoch.curr);
        self.state.table.init_epoch(epoch);
        self.degree_state.table.init_epoch(epoch);
    }

    pub fn update_epoch(&mut self, epoch: u64) {
        // Update the current epoch in `ManagedLruCache`
        self.inner.update_epoch(epoch)
    }

    pub fn update_vnode_bitmap(&mut self, vnode_bitmap: Arc<Bitmap>) {
        self.state.table.update_vnode_bitmap(vnode_bitmap.clone());
        self.degree_state.table.update_vnode_bitmap(vnode_bitmap);
    }

    /// Returns a mutable reference to the value of the key in the memory, if does not exist, look
    /// up in remote storage and return. If not exist in remote storage, a
    /// `JoinEntryState` with empty cache will be returned.
    #[expect(dead_code)]
    #[cfg(any())]
    pub async fn get<'a>(&'a mut self, key: &K) -> Option<&'a HashValueType> {
        // TODO: add metrics for get
        let state = self.inner.get(key);
        // TODO: we should probably implement a entry function for `LruCache`
        match state {
            Some(_) => self.inner.get(key),
            None => {
                let remote_state = self.fetch_cached_state(key).await.unwrap();
                self.inner.put(key.clone(), remote_state);
                self.inner.get(key)
            }
        }
    }

    /// Returns a mutable reference to the value of the key in the memory, if does not exist, look
    /// up in remote storage and return. If not exist in remote storage, a
    /// `JoinEntryState` with empty cache will be returned.
    #[expect(dead_code)]
    #[cfg(any())]
    pub async fn get_mut<'a>(&'a mut self, key: &'a K) -> Option<&'a mut HashValueType> {
        // TODO: add metrics for get_mut
        let state = self.inner.get(key);
        // TODO: we should probably implement a entry function for `LruCache`
        match state {
            Some(_) => self.inner.get_mut(key),
            None => {
                let remote_state = self.fetch_cached_state(key).await.unwrap();
                self.inner.put(key.clone(), remote_state);
                self.inner.get_mut(key)
            }
        }
    }

    /// Remove the key in the memory, returning the value at the key if the
    /// key was previously in the map. If does not exist, look
    /// up in remote storage and return. If not exist in remote storage, a
    /// `JoinEntryState` with empty cache will be returned.
    /// WARNING: This will NOT remove anything from remote storage.
    pub async fn remove_state<'a>(&mut self, key: &K) -> StreamExecutorResult<HashValueType> {
        let state = self.inner.pop(key);
        self.metrics.total_lookup_count += 1;
        Ok(match state {
            Some(state) => state,
            None => {
                self.metrics.lookup_miss_count += 1;
                self.fetch_cached_state(key).await?
            }
        })
    }

    /// Fetch cache from the state store. Should only be called if the key does not exist in memory.
    /// Will return a empty `JoinEntryState` even when state does not exist in remote.
    async fn fetch_cached_state(&self, key: &K) -> StreamExecutorResult<JoinEntryState> {
        let key = key.clone().deserialize(&self.join_key_data_types)?;

        let table_iter_fut = self.state.table.iter_key_and_val(&key);

        let mut entry_state = JoinEntryState::default();

        if self.need_degree_table {
            let degree_table_iter_fut = self.degree_state.table.iter_key_and_val(&key);

            let (table_iter, degree_table_iter) =
                try_join(table_iter_fut, degree_table_iter_fut).await?;

            // We need this because ttl may remove some entries from table but leave the entries
            // with the same stream key in degree table.
            let zipped_iter = zip_by_order_key(table_iter, degree_table_iter);

            #[for_await]
            for row_and_degree in zipped_iter {
                let (row, degree) = row_and_degree?;
                let pk = row
                    .extract_memcomparable_by_indices(&self.pk_serializer, &self.state.pk_indices);
                let degree_i64 = degree
                    .0
                    .last()
                    .cloned()
                    .context("Empty row")?
                    .context("Fail to fetch a degree")?;
                entry_state.insert(
                    pk,
                    JoinRow::new(row.into_owned(), *degree_i64.as_int64() as u64).encode(),
                );
            }
        } else {
            let table_iter = table_iter_fut.await?;

            #[for_await]
            for row in table_iter {
                let row = row?.1;
                let pk = row
                    .extract_memcomparable_by_indices(&self.pk_serializer, &self.state.pk_indices);
                entry_state.insert(pk, JoinRow::new(row.into_owned(), 0).encode());
            }
        };

        Ok(entry_state)
    }

    pub async fn flush(&mut self, epoch: EpochPair) -> StreamExecutorResult<()> {
        self.metrics.flush();
        self.state.table.commit(epoch).await?;
        self.degree_state.table.commit(epoch).await?;
        Ok(())
    }

    /// Insert a join row
    pub fn insert(&mut self, key: &K, value: JoinRow) {
        if let Some(entry) = self.inner.get_mut(key) {
            let pk = value
                .row
                .extract_memcomparable_by_indices(&self.pk_serializer, &self.state.pk_indices);
            entry.insert(pk, value.encode());
        }
        // If no cache maintained, only update the flush buffer.
        let (row, degree) = value.into_table_rows(&self.state.order_key_indices);
        self.state.table.insert(row);
        self.degree_state.table.insert(degree);
    }

    /// Insert a row.
    /// Used when the side does not need to update degree.
    pub fn insert_row(&mut self, key: &K, value: Row) {
        let join_row = JoinRow::new(value.clone(), 0);

        if let Some(entry) = self.inner.get_mut(key) {
            let pk =
                value.extract_memcomparable_by_indices(&self.pk_serializer, &self.state.pk_indices);
            entry.insert(pk, join_row.encode());
        }
        // If no cache maintained, only update the state table.
        self.state.table.insert(value);
    }

    /// Delete a join row
    pub fn delete(&mut self, key: &K, value: JoinRow) {
        if let Some(entry) = self.inner.get_mut(key) {
            let pk = value
                .row
                .extract_memcomparable_by_indices(&self.pk_serializer, &self.state.pk_indices);
            entry.remove(pk);
        }

        // If no cache maintained, only update the state table.
        let (row, degree) = value.into_table_rows(&self.state.order_key_indices);
        self.state.table.delete(row);
        self.degree_state.table.delete(degree);
    }

    /// Delete a row
    /// Used when the side does not need to update degree.
    pub fn delete_row(&mut self, key: &K, value: Row) {
        if let Some(entry) = self.inner.get_mut(key) {
            let pk =
                value.extract_memcomparable_by_indices(&self.pk_serializer, &self.state.pk_indices);
            entry.remove(pk);
        }

        // If no cache maintained, only update the state table.
        self.state.table.delete(value);
    }

    /// Insert a [`JoinEntryState`]
    pub fn insert_state(&mut self, key: &K, state: JoinEntryState) {
        self.inner.put(key.clone(), state);
    }

    pub fn inc_degree(&mut self, join_row: &mut StateValueType) -> StreamExecutorResult<()> {
        let old_degree = join_row
            .get_schemaed_degree(&self.state.all_data_types, &self.state.order_key_indices)?;
        join_row.inc_degree();
        let new_degree = join_row
            .get_schemaed_degree(&self.state.all_data_types, &self.state.order_key_indices)?;

        self.degree_state.table.update(old_degree, new_degree);
        Ok(())
    }

    pub fn dec_degree(&mut self, join_row: &mut StateValueType) -> StreamExecutorResult<()> {
        let old_degree = join_row
            .get_schemaed_degree(&self.state.all_data_types, &self.state.order_key_indices)?;
        join_row.dec_degree()?;
        let new_degree = join_row
            .get_schemaed_degree(&self.state.all_data_types, &self.state.order_key_indices)?;

        self.degree_state.table.update(old_degree, new_degree);
        Ok(())
    }

    /// Cached rows for this hash table.
    #[expect(dead_code)]
    pub fn cached_rows(&self) -> usize {
        self.values().map(|e| e.len()).sum()
    }

    /// Cached entry count for this hash table.
    pub fn entry_count(&self) -> usize {
        self.len()
    }

    /// Estimated memory usage for this hash table.
    #[expect(dead_code)]
    pub fn estimated_size(&self) -> usize {
        self.iter()
            .map(|(k, v)| k.estimated_size() + v.estimated_size())
            .sum()
    }

    pub fn null_matched(&self) -> &FixedBitSet {
        &self.null_matched
    }
}

impl<K: HashKey, S: StateStore> Deref for JoinHashMap<K, S> {
    type Target = JoinHashMapInner<K>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<K: HashKey, S: StateStore> DerefMut for JoinHashMap<K, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
