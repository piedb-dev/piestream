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

use std::marker::PhantomData;
use std::sync::Arc;

use futures::{stream, StreamExt};
use futures_async_stream::try_stream;
use iter_chunks::IterChunks;
use itertools::Itertools;
use madsim::collections::HashMap;
use piestream_common::array::column::Column;
use piestream_common::array::{StreamChunk, Vis};
use piestream_common::buffer::Bitmap;
use piestream_common::catalog::Schema;
use piestream_common::collection::evictable::EvictableHashMap;
use piestream_common::error::Result;
use piestream_common::hash::{HashCode, HashKey};
use piestream_common::util::hash_util::CRC32FastBuilder;
use piestream_storage::table::state_table::StateTable;
use piestream_storage::StateStore;

use super::{
    expect_first_barrier, pk_input_arrays, Executor, PkDataTypes, PkIndicesRef,
    StreamExecutorResult,
};
use crate::executor::aggregation::{
    agg_input_arrays, generate_agg_schema, generate_managed_agg_state, AggCall, AggState,
};
use crate::executor::error::StreamExecutorError;
use crate::executor::{BoxedMessageStream, Message, PkIndices, PROCESSING_WINDOW_SIZE};

/// [`HashAggExecutor`] could process large amounts of data using a state backend. It works as
/// follows:
///
/// * The executor pulls data from the upstream, and apply the data chunks to the corresponding
///   aggregation states.
/// * While processing, it will record which keys have been modified in this epoch using
///   `modified_keys`.
/// * Upon a barrier is received, the executor will call `.flush` on the storage backend, so that
///   all modifications will be flushed to the storage backend. Meanwhile, the executor will go
///   through `modified_keys`, and produce a stream chunk based on the state changes.
pub struct HashAggExecutor<K: HashKey, S: StateStore> {
    input: Box<dyn Executor>,

    extra: HashAggExecutorExtra<S>,

    _phantom: PhantomData<K>,
}

struct HashAggExecutorExtra<S: StateStore> {
    /// See [`Executor::schema`].
    schema: Schema,

    /// See [`Executor::pk_indices`].
    pk_indices: PkIndices,

    /// See [`Executor::identity`].
    identity: String,

    /// Pk indices from input
    input_pk_indices: Vec<usize>,

    /// Schema from input
    input_schema: Schema,

    /// A [`HashAggExecutor`] may have multiple [`AggCall`]s.
    agg_calls: Vec<AggCall>,

    /// Indices of the columns
    /// all of the aggregation functions in this executor should depend on same group of keys
    key_indices: Vec<usize>,

    state_tables: Vec<StateTable<S>>,
}

impl<K: HashKey, S: StateStore> Executor for HashAggExecutor<K, S> {
    fn execute(self: Box<Self>) -> BoxedMessageStream {
        self.execute_inner().boxed()
    }

    fn schema(&self) -> &Schema {
        &self.extra.schema
    }

    fn pk_indices(&self) -> PkIndicesRef {
        &self.extra.pk_indices
    }

    fn identity(&self) -> &str {
        &self.extra.identity
    }
}

impl<K: HashKey, S: StateStore> HashAggExecutor<K, S> {
    pub fn new(
        input: Box<dyn Executor>,
        agg_calls: Vec<AggCall>,
        pk_indices: PkIndices,
        executor_id: u64,
        key_indices: Vec<usize>,
        state_tables: Vec<StateTable<S>>,
    ) -> Result<Self> {
        let input_info = input.info();
        let schema = generate_agg_schema(input.as_ref(), &agg_calls, Some(&key_indices));

        Ok(Self {
            input,
            extra: HashAggExecutorExtra {
                schema,
                pk_indices,
                identity: format!("HashAggExecutor-{:X}", executor_id),
                input_pk_indices: input_info.pk_indices,
                input_schema: input_info.schema,
                agg_calls,
                key_indices,
                state_tables,
            },
            _phantom: PhantomData,
        })
    }

    /// Get unique keys, hash codes and visibility map of each key in a batch.
    ///
    /// The returned order is the same as how we get distinct final columns from original columns.
    ///
    /// `keys` are Hash Keys of all the rows
    /// `key_hash_codes` are hash codes of the deserialized `keys`
    /// `visibility`, leave invisible ones out of aggregation
    fn get_unique_keys(
        keys: Vec<K>,
        key_hash_codes: Vec<HashCode>,
        visibility: &Option<Bitmap>,
    ) -> Result<Vec<(K, HashCode, Bitmap)>> {
        let total_num_rows = keys.len();
        assert_eq!(key_hash_codes.len(), total_num_rows);
        // Each hash key, e.g. `key1` corresponds to a visibility map that not only shadows
        // all the rows whose keys are not `key1`, but also shadows those rows shadowed in the
        // `input` The visibility map of each hash key will be passed into
        // `StreamingAggStateImpl`.
        let mut key_to_vis_maps = HashMap::new();

        // Give all the unique keys an order and iterate them later,
        // the order is the same as how we get distinct final columns from original columns.
        let mut unique_key_and_hash_codes = Vec::new();

        for (row_idx, (key, hash_code)) in keys.iter().zip_eq(key_hash_codes.iter()).enumerate() {
            // if the visibility map has already shadowed this row,
            // then we pass
            if let Some(vis_map) = visibility && !vis_map.is_set(row_idx)? {
                continue;
            }
            let vis_map = key_to_vis_maps.entry(key).or_insert_with(|| {
                unique_key_and_hash_codes.push((key, hash_code));
                vec![false; total_num_rows]
            });
            vis_map[row_idx] = true;
        }

        let result = unique_key_and_hash_codes
            .into_iter()
            .map(|(key, hash_code)| {
                (
                    key.clone(),
                    hash_code.clone(),
                    key_to_vis_maps.remove(key).unwrap().into_iter().collect(),
                )
            })
            .collect_vec();

        Ok(result)
    }

    async fn apply_chunk(
        &mut HashAggExecutorExtra::<S> {
            ref key_indices,
            ref agg_calls,
            ref input_pk_indices,
            ref input_schema,
            ref schema,
            ref mut state_tables,
            ..
        }: &mut HashAggExecutorExtra<S>,
        state_map: &mut EvictableHashMap<K, Option<Box<AggState<S>>>>,
        chunk: StreamChunk,
        epoch: u64,
    ) -> StreamExecutorResult<()> {
        let (data_chunk, ops) = chunk.into_parts();

        // Compute hash code here before serializing keys to avoid duplicate hash code computation.
        let hash_codes = data_chunk.get_hash_values(key_indices, CRC32FastBuilder)?;
        let keys = K::build_from_hash_code(key_indices, &data_chunk, hash_codes.clone());
        let (columns, vis) = data_chunk.into_parts();
        let visibility = match vis {
            Vis::Bitmap(b) => Some(b),
            Vis::Compact(_) => None,
        };

        // --- Find unique keys in this batch and generate visibility map for each key ---
        // TODO: this might be inefficient if there are not too many duplicated keys in one batch.
        let unique_keys = Self::get_unique_keys(keys, hash_codes, &visibility)
            .map_err(StreamExecutorError::eval_error)?;

        // --- Retrieve all aggregation inputs in advance ---
        // Previously, this is done in `unique_keys` inner loop, which is very inefficient.
        let all_agg_input_arrays = agg_input_arrays(agg_calls, &columns);
        let pk_input_arrays = pk_input_arrays(input_pk_indices, &columns);

        let input_pk_data_types: PkDataTypes = input_pk_indices
            .iter()
            .map(|idx| input_schema.fields[*idx].data_type.clone())
            .collect();

        // When applying batch, we will send columns of primary keys to the last N columns.
        let all_agg_data = all_agg_input_arrays
            .into_iter()
            .map(|mut input_arrays| {
                input_arrays.extend(pk_input_arrays.iter().cloned());
                input_arrays
            })
            .collect_vec();

        let key_data_types = &schema.data_types()[..key_indices.len()];
        let mut futures = vec![];
        for (key, hash_code, _) in &unique_keys {
            // Retrieve previous state from the KeyedState.
            let states = state_map.put(key.to_owned(), None);

            let key = key.clone();
            // To leverage more parallelism in IO operations, fetching and updating states for every
            // unique keys is created as futures and run in parallel.
            futures.push(async {
                // 1. If previous state didn't exist, the ManagedState will automatically create new
                // ones for them.
                let mut states = {
                    match states {
                        Some(s) => s.unwrap(),
                        None => Box::new(
                            generate_managed_agg_state(
                                Some(
                                    &key.clone()
                                        .deserialize(key_data_types.iter())
                                        .map_err(StreamExecutorError::eval_error)?,
                                ),
                                agg_calls,
                                input_pk_data_types.clone(),
                                epoch,
                                Some(hash_code.clone()),
                                state_tables,
                            )
                            .await?,
                        ),
                    }
                };

                // 2. Mark the state as dirty by filling prev states
                states.may_mark_as_dirty(epoch, state_tables).await?;

                Ok::<(_, Box<AggState<S>>), StreamExecutorError>((key, states))
            });
        }

        let mut buffered = stream::iter(futures).buffer_unordered(10).fuse();

        while let Some(result) = buffered.next().await {
            let (key, state) = result?;
            state_map.put(key, Some(state));
        }
        // Drop the stream manually to teach compiler the async closure above will not use the read
        // ref anymore.
        drop(buffered);

        // Apply batch in single-thread.
        for (key, _, vis_map) in &unique_keys {
            let state = state_map.get_mut(key).unwrap().as_mut().unwrap();
            // 3. Apply batch to each of the state (per agg_call)
            for ((agg_state, data), state_table) in state
                .managed_states
                .iter_mut()
                .zip_eq(all_agg_data.iter())
                .zip_eq(state_tables.iter_mut())
            {
                let data = data.iter().map(|d| &**d).collect_vec();
                agg_state
                    .apply_batch(&ops, Some(vis_map), &data, epoch, state_table)
                    .await?;
            }
        }

        Ok(())
    }

    #[try_stream(ok = StreamChunk, error = StreamExecutorError)]
    async fn flush_data<'a>(
        &mut HashAggExecutorExtra::<S> {
            ref key_indices,
            ref schema,
            ref mut state_tables,
            ..
        }: &'a mut HashAggExecutorExtra<S>,
        state_map: &'a mut EvictableHashMap<K, Option<Box<AggState<S>>>>,
        epoch: u64,
    ) {
        // --- Flush states to the state store ---
        // Some state will have the correct output only after their internal states have been
        // fully flushed.
        let dirty_cnt = {
            let mut dirty_cnt = 0;
            for states in state_map.values_mut() {
                if states.as_ref().unwrap().is_dirty() {
                    dirty_cnt += 1;
                    for (state, state_table) in states
                        .as_mut()
                        .unwrap()
                        .managed_states
                        .iter_mut()
                        .zip_eq(state_tables.iter_mut())
                    {
                        state.flush(state_table).await?;
                    }
                }
            }

            dirty_cnt
        };

        if dirty_cnt == 0 {
            // Nothing to flush.
            return Ok(());
        } else {
            // Batch commit data.
            for state_table in state_tables.iter_mut() {
                state_table.commit(epoch).await?;
            }

            // --- Produce the stream chunk ---
            let mut batches = IterChunks::chunks(state_map.iter_mut(), PROCESSING_WINDOW_SIZE);
            while let Some(batch) = batches.next() {
                // --- Create array builders ---
                // As the datatype is retrieved from schema, it contains both group key and
                // aggregation state outputs.
                let mut builders = schema.create_array_builders(dirty_cnt * 2);
                let mut new_ops = Vec::with_capacity(dirty_cnt);

                // --- Retrieve modified states and put the changes into the builders ---
                for (key, states) in batch {
                    let appended = states
                        .as_mut()
                        .unwrap()
                        .build_changes(
                            &mut builders[key_indices.len()..],
                            &mut new_ops,
                            epoch,
                            state_tables,
                        )
                        .await?;

                    for _ in 0..appended {
                        key.clone()
                            .deserialize_to_builders(&mut builders[..key_indices.len()])
                            .map_err(StreamExecutorError::eval_error)?;
                    }
                }

                let columns: Vec<Column> = builders
                    .into_iter()
                    .map(|builder| -> Result<_> { Ok(Column::new(Arc::new(builder.finish()?))) })
                    .try_collect()
                    .map_err(StreamExecutorError::eval_error)?;

                let chunk = StreamChunk::new(new_ops, columns, None);

                trace!("output_chunk: {:?}", &chunk);
                yield chunk;
            }

            // evict cache to target capacity
            // In current implementation, we need to fetch the RowCount from the state store
            // once a key is deleted and added again. We should find a way to
            // eliminate this extra fetch.
            assert!(!state_map
                .values()
                .any(|state| state.as_ref().unwrap().is_dirty()));
            state_map.evict_to_target_cap();
        }
    }

    #[try_stream(ok = Message, error = StreamExecutorError)]
    async fn execute_inner(self) {
        let HashAggExecutor {
            input, mut extra, ..
        } = self;

        // The cached states. `HashKey -> (prev_value, value)`.
        let mut state_map = EvictableHashMap::new(1 << 16);

        let mut input = input.execute();
        let barrier = expect_first_barrier(&mut input).await?;
        let mut epoch = barrier.epoch.curr;
        yield Message::Barrier(barrier);

        #[for_await]
        for msg in input {
            let msg = msg?;
            match msg {
                Message::Chunk(chunk) => {
                    Self::apply_chunk(&mut extra, &mut state_map, chunk, epoch).await?;
                }
                Message::Barrier(barrier) => {
                    let next_epoch = barrier.epoch.curr;
                    assert_eq!(epoch, barrier.epoch.prev);

                    #[for_await]
                    for chunk in Self::flush_data(&mut extra, &mut state_map, epoch) {
                        yield Message::Chunk(chunk?);
                    }

                    yield Message::Barrier(barrier);
                    epoch = next_epoch;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use assert_matches::assert_matches;
    use futures::StreamExt;
    use itertools::Itertools;
    use piestream_common::array::data_chunk_iter::Row;
    use piestream_common::array::stream_chunk::StreamChunkTestExt;
    use piestream_common::array::{Op, StreamChunk};
    use piestream_common::catalog::{Field, Schema, TableId};
    use piestream_common::error::Result;
    use piestream_common::hash::{calc_hash_key_kind, HashKey, HashKeyDispatcher};
    use piestream_common::types::DataType;
    use piestream_expr::expr::*;
    use piestream_storage::memory::MemoryStateStore;
    use piestream_storage::table::state_table::StateTable;
    use piestream_storage::StateStore;

    use crate::executor::aggregation::{generate_agg_schema, AggArgs, AggCall};
    use crate::executor::test_utils::global_simple_agg::generate_state_table;
    use crate::executor::test_utils::*;
    use crate::executor::{Executor, HashAggExecutor, Message, PkIndices};

    struct HashAggExecutorDispatcher<S: StateStore>(PhantomData<S>);

    struct HashAggExecutorDispatcherArgs<S: StateStore> {
        input: Box<dyn Executor>,
        agg_calls: Vec<AggCall>,
        key_indices: Vec<usize>,
        pk_indices: PkIndices,
        executor_id: u64,
        state_tables: Vec<StateTable<S>>,
    }

    impl<S: StateStore> HashKeyDispatcher for HashAggExecutorDispatcher<S> {
        type Input = HashAggExecutorDispatcherArgs<S>;
        type Output = Result<Box<dyn Executor>>;

        fn dispatch<K: HashKey>(args: Self::Input) -> Self::Output {
            Ok(Box::new(HashAggExecutor::<K, S>::new(
                args.input,
                args.agg_calls,
                args.pk_indices,
                args.executor_id,
                args.key_indices,
                args.state_tables,
            )?))
        }
    }

    fn new_boxed_hash_agg_executor(
        input: Box<dyn Executor>,
        agg_calls: Vec<AggCall>,
        key_indices: Vec<usize>,
        keyspace_gen: Vec<(MemoryStateStore, TableId)>,
        pk_indices: PkIndices,
        executor_id: u64,
    ) -> Box<dyn Executor> {
        let keys = key_indices
            .iter()
            .map(|idx| input.schema().fields[*idx].data_type())
            .collect_vec();
        let agg_schema = generate_agg_schema(input.as_ref(), &agg_calls, Some(&key_indices));
        let state_tables = keyspace_gen
            .iter()
            .zip_eq(agg_calls.iter())
            .map(|(ks, agg_call)| {
                generate_state_table(
                    ks.0.clone(),
                    ks.1,
                    agg_call,
                    &key_indices,
                    &pk_indices,
                    &agg_schema,
                    input.as_ref(),
                )
            })
            .collect();
        let args = HashAggExecutorDispatcherArgs {
            input,
            agg_calls,
            key_indices,
            pk_indices,
            executor_id,
            state_tables,
        };
        let kind = calc_hash_key_kind(&keys);
        HashAggExecutorDispatcher::dispatch_by_kind(kind, args).unwrap()
    }

    // --- Test HashAgg with in-memory KeyedState ---

    #[tokio::test]
    async fn test_local_hash_aggregation_count_in_memory() {
        test_local_hash_aggregation_count(create_in_memory_keyspace_agg(3)).await
    }

    #[tokio::test]
    async fn test_global_hash_aggregation_count_in_memory() {
        test_global_hash_aggregation_count(create_in_memory_keyspace_agg(3)).await
    }

    #[tokio::test]
    async fn test_local_hash_aggregation_min_in_memory() {
        test_local_hash_aggregation_min(create_in_memory_keyspace_agg(2)).await
    }

    #[tokio::test]
    async fn test_local_hash_aggregation_min_append_only_in_memory() {
        test_local_hash_aggregation_min_append_only(create_in_memory_keyspace_agg(2)).await
    }

    async fn test_local_hash_aggregation_count(keyspace: Vec<(MemoryStateStore, TableId)>) {
        let schema = Schema {
            fields: vec![Field::unnamed(DataType::Int64)],
        };
        let (mut tx, source) = MockSource::channel(schema, PkIndices::new());
        tx.push_barrier(1, false);
        tx.push_chunk(StreamChunk::from_pretty(
            " I
            + 1
            + 2
            + 2",
        ));
        tx.push_barrier(2, false);
        tx.push_chunk(StreamChunk::from_pretty(
            " I
            - 1
            - 2 D
            - 2",
        ));
        tx.push_barrier(3, false);

        // This is local hash aggregation, so we add another row count state
        let keys = vec![0];
        let append_only = false;
        let agg_calls = vec![
            AggCall {
                kind: AggKind::RowCount,
                args: AggArgs::None,
                return_type: DataType::Int64,
                append_only,
            },
            AggCall {
                kind: AggKind::Count,
                args: AggArgs::Unary(DataType::Int64, 0),
                return_type: DataType::Int64,
                append_only,
            },
            AggCall {
                kind: AggKind::Count,
                args: AggArgs::None,
                return_type: DataType::Int64,
                append_only,
            },
        ];

        let hash_agg =
            new_boxed_hash_agg_executor(Box::new(source), agg_calls, keys, keyspace, vec![], 1);
        let mut hash_agg = hash_agg.execute();

        // Consume the init barrier
        hash_agg.next().await.unwrap().unwrap();
        // Consume stream chunk
        let msg = hash_agg.next().await.unwrap().unwrap();
        assert_eq!(
            msg.into_chunk().unwrap().sorted_rows(),
            StreamChunk::from_pretty(
                " I I I I
                + 1 1 1 1
                + 2 2 2 2"
            )
            .sorted_rows(),
        );

        assert_matches!(
            hash_agg.next().await.unwrap().unwrap(),
            Message::Barrier { .. }
        );

        let msg = hash_agg.next().await.unwrap().unwrap();
        assert_eq!(
            msg.into_chunk().unwrap().sorted_rows(),
            StreamChunk::from_pretty(
                "  I I I I
                -  1 1 1 1
                U- 2 2 2 2
                U+ 2 1 1 1"
            )
            .sorted_rows(),
        );
    }

    async fn test_global_hash_aggregation_count(keyspace: Vec<(MemoryStateStore, TableId)>) {
        let schema = Schema {
            fields: vec![
                Field::unnamed(DataType::Int64),
                Field::unnamed(DataType::Int64),
                Field::unnamed(DataType::Int64),
            ],
        };

        let (mut tx, source) = MockSource::channel(schema, PkIndices::new());
        tx.push_barrier(1, false);
        tx.push_chunk(StreamChunk::from_pretty(
            " I I I
            + 1 1 1
            + 2 2 2
            + 2 2 2",
        ));
        tx.push_barrier(2, false);
        tx.push_chunk(StreamChunk::from_pretty(
            " I I I
            - 1 1 1
            - 2 2 2 D
            - 2 2 2
            + 3 3 3",
        ));
        tx.push_barrier(3, false);

        // This is local hash aggregation, so we add another sum state
        let key_indices = vec![0];
        let append_only = false;
        let agg_calls = vec![
            AggCall {
                kind: AggKind::RowCount,
                args: AggArgs::None,
                return_type: DataType::Int64,
                append_only,
            },
            AggCall {
                kind: AggKind::Sum,
                args: AggArgs::Unary(DataType::Int64, 1),
                return_type: DataType::Int64,
                append_only,
            },
            // This is local hash aggregation, so we add another sum state
            AggCall {
                kind: AggKind::Sum,
                args: AggArgs::Unary(DataType::Int64, 2),
                return_type: DataType::Int64,
                append_only,
            },
        ];

        let hash_agg = new_boxed_hash_agg_executor(
            Box::new(source),
            agg_calls,
            key_indices,
            keyspace,
            vec![],
            1,
        );
        let mut hash_agg = hash_agg.execute();

        // Consume the init barrier
        hash_agg.next().await.unwrap().unwrap();
        // Consume stream chunk
        let msg = hash_agg.next().await.unwrap().unwrap();
        assert_eq!(
            msg.into_chunk().unwrap().sorted_rows(),
            StreamChunk::from_pretty(
                " I I I I
                + 1 1 1 1
                + 2 2 4 4"
            )
            .sorted_rows(),
        );

        assert_matches!(
            hash_agg.next().await.unwrap().unwrap(),
            Message::Barrier { .. }
        );

        let msg = hash_agg.next().await.unwrap().unwrap();
        assert_eq!(
            msg.into_chunk().unwrap().sorted_rows(),
            StreamChunk::from_pretty(
                "  I I I I
                -  1 1 1 1
                U- 2 2 4 4
                U+ 2 1 2 2
                +  3 1 3 3"
            )
            .sorted_rows(),
        );
    }

    async fn test_local_hash_aggregation_min(keyspace: Vec<(MemoryStateStore, TableId)>) {
        let schema = Schema {
            fields: vec![
                // group key column
                Field::unnamed(DataType::Int64),
                // data column to get minimum
                Field::unnamed(DataType::Int64),
                // primary key column
                Field::unnamed(DataType::Int64),
            ],
        };
        let (mut tx, source) = MockSource::channel(schema, vec![2]); // pk
        tx.push_barrier(1, false);
        tx.push_chunk(StreamChunk::from_pretty(
            " I     I    I
            + 1   233 1001
            + 1 23333 1002
            + 2  2333 1003",
        ));
        tx.push_barrier(2, false);
        tx.push_chunk(StreamChunk::from_pretty(
            " I     I    I
            - 1   233 1001
            - 1 23333 1002 D
            - 2  2333 1003",
        ));
        tx.push_barrier(3, false);

        // This is local hash aggregation, so we add another row count state
        let keys = vec![0];
        let agg_calls = vec![
            AggCall {
                kind: AggKind::RowCount,
                args: AggArgs::None,
                return_type: DataType::Int64,
                append_only: false,
            },
            AggCall {
                kind: AggKind::Min,
                args: AggArgs::Unary(DataType::Int64, 1),
                return_type: DataType::Int64,
                append_only: false,
            },
        ];

        let hash_agg =
            new_boxed_hash_agg_executor(Box::new(source), agg_calls, keys, keyspace, vec![2], 1);
        let mut hash_agg = hash_agg.execute();

        // Consume the init barrier
        hash_agg.next().await.unwrap().unwrap();
        // Consume stream chunk
        let msg = hash_agg.next().await.unwrap().unwrap();
        assert_eq!(
            msg.into_chunk().unwrap().sorted_rows(),
            StreamChunk::from_pretty(
                " I I    I
                + 1 2  233
                + 2 1 2333"
            )
            .sorted_rows(),
        );

        assert_matches!(
            hash_agg.next().await.unwrap().unwrap(),
            Message::Barrier { .. }
        );

        let msg = hash_agg.next().await.unwrap().unwrap();
        assert_eq!(
            msg.into_chunk().unwrap().sorted_rows(),
            StreamChunk::from_pretty(
                "  I I     I
                -  2 1  2333
                U- 1 2   233
                U+ 1 1 23333"
            )
            .sorted_rows(),
        );
    }

    async fn test_local_hash_aggregation_min_append_only(
        keyspace: Vec<(MemoryStateStore, TableId)>,
    ) {
        let schema = Schema {
            fields: vec![
                // group key column
                Field::unnamed(DataType::Int64),
                // data column to get minimum
                Field::unnamed(DataType::Int64),
                // primary key column
                Field::unnamed(DataType::Int64),
            ],
        };
        let (mut tx, source) = MockSource::channel(schema, vec![2]); // pk
        tx.push_barrier(1, false);
        tx.push_chunk(StreamChunk::from_pretty(
            " I  I  I
            + 2 5  1000
            + 1 15 1001
            + 1 8  1002
            + 2 5  1003
            + 2 10 1004
            ",
        ));
        tx.push_barrier(2, false);
        tx.push_chunk(StreamChunk::from_pretty(
            " I  I  I
            + 1 20 1005
            + 1 1  1006
            + 2 10 1007
            + 2 20 1008
            ",
        ));
        tx.push_barrier(3, false);

        // This is local hash aggregation, so we add another row count state
        let keys = vec![0];
        let append_only = true;
        let agg_calls = vec![
            AggCall {
                kind: AggKind::RowCount,
                args: AggArgs::None,
                return_type: DataType::Int64,
                append_only,
            },
            AggCall {
                kind: AggKind::Min,
                args: AggArgs::Unary(DataType::Int64, 1),
                return_type: DataType::Int64,
                append_only,
            },
        ];

        let hash_agg =
            new_boxed_hash_agg_executor(Box::new(source), agg_calls, keys, keyspace, vec![2], 1);
        let mut hash_agg = hash_agg.execute();

        // Consume the init barrier
        hash_agg.next().await.unwrap().unwrap();
        // Consume stream chunk
        let msg = hash_agg.next().await.unwrap().unwrap();
        assert_eq!(
            msg.into_chunk().unwrap().sorted_rows(),
            StreamChunk::from_pretty(
                " I I    I
                + 1 2 8
                + 2 3 5"
            )
            .sorted_rows(),
        );

        assert_matches!(
            hash_agg.next().await.unwrap().unwrap(),
            Message::Barrier { .. }
        );

        let msg = hash_agg.next().await.unwrap().unwrap();
        assert_eq!(
            msg.into_chunk().unwrap().sorted_rows(),
            StreamChunk::from_pretty(
                "  I I  I
                U- 1 2 8
                U+ 1 4 1
                U- 2 3 5 
                U+ 2 5 5
                "
            )
            .sorted_rows(),
        );
    }

    trait SortedRows {
        fn sorted_rows(self) -> Vec<(Op, Row)>;
    }
    impl SortedRows for StreamChunk {
        fn sorted_rows(self) -> Vec<(Op, Row)> {
            let (chunk, ops) = self.into_parts();
            ops.into_iter()
                .zip_eq(chunk.rows().map(Row::from))
                .sorted()
                .collect_vec()
        }
    }
}
