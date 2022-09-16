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

use std::sync::Arc;

use futures::StreamExt;
use futures_async_stream::try_stream;
use itertools::Itertools;
use madsim::collections::HashSet;
use piestream_common::array::{Array, ArrayRef, Op, Row, RowRef, StreamChunk};
use piestream_common::bail;
use piestream_common::catalog::{Schema, TableId};
use piestream_common::hash::HashKey;
use piestream_common::types::{DataType, ToOwnedDatum};
use piestream_expr::expr::RowExpression;
use piestream_storage::StateStore;

use super::barrier_align::*;
use super::error::{StreamExecutorError, StreamExecutorResult};
use super::managed_state::join::*;
use super::monitor::StreamingMetrics;
use super::{BoxedExecutor, BoxedMessageStream, Executor, Message, PkIndices, PkIndicesRef};
use crate::common::StreamChunkBuilder;
use crate::executor::PROCESSING_WINDOW_SIZE;

pub const JOIN_CACHE_SIZE: usize = 1 << 16;

/// The `JoinType` and `SideType` are to mimic a enum, because currently
/// enum is not supported in const generic.
// TODO: Use enum to replace this once [feature(adt_const_params)](https://github.com/rust-lang/rust/issues/95174) get completed.
pub type JoinTypePrimitive = u8;
#[allow(non_snake_case, non_upper_case_globals)]
pub mod JoinType {
    use super::JoinTypePrimitive;
    pub const Inner: JoinTypePrimitive = 0;
    pub const LeftOuter: JoinTypePrimitive = 1;
    pub const RightOuter: JoinTypePrimitive = 2;
    pub const FullOuter: JoinTypePrimitive = 3;
    pub const LeftSemi: JoinTypePrimitive = 4;
    pub const LeftAnti: JoinTypePrimitive = 5;
    pub const RightSemi: JoinTypePrimitive = 6;
    pub const RightAnti: JoinTypePrimitive = 7;
}

pub type SideTypePrimitive = u8;
#[allow(non_snake_case, non_upper_case_globals)]
pub mod SideType {
    use super::SideTypePrimitive;
    pub const Left: SideTypePrimitive = 0;
    pub const Right: SideTypePrimitive = 1;
}

const fn is_outer_side(join_type: JoinTypePrimitive, side_type: SideTypePrimitive) -> bool {
    join_type == JoinType::FullOuter
        || (join_type == JoinType::LeftOuter && side_type == SideType::Left)
        || (join_type == JoinType::RightOuter && side_type == SideType::Right)
}

const fn outer_side_null(join_type: JoinTypePrimitive, side_type: SideTypePrimitive) -> bool {
    join_type == JoinType::FullOuter
        || (join_type == JoinType::LeftOuter && side_type == SideType::Right)
        || (join_type == JoinType::RightOuter && side_type == SideType::Left)
}

/// Send the update only once if the join type is semi/anti and the update is the same side as the
/// join
const fn forward_exactly_once(join_type: JoinTypePrimitive, side_type: SideTypePrimitive) -> bool {
    ((join_type == JoinType::LeftSemi || join_type == JoinType::LeftAnti)
        && side_type == SideType::Left)
        || ((join_type == JoinType::RightSemi || join_type == JoinType::RightAnti)
            && side_type == SideType::Right)
}

const fn only_forward_matched_side(
    join_type: JoinTypePrimitive,
    side_type: SideTypePrimitive,
) -> bool {
    ((join_type == JoinType::LeftSemi || join_type == JoinType::LeftAnti)
        && side_type == SideType::Right)
        || ((join_type == JoinType::RightSemi || join_type == JoinType::RightAnti)
            && side_type == SideType::Left)
}

const fn is_semi(join_type: JoinTypePrimitive) -> bool {
    join_type == JoinType::LeftSemi || join_type == JoinType::RightSemi
}

const fn is_anti(join_type: JoinTypePrimitive) -> bool {
    join_type == JoinType::LeftAnti || join_type == JoinType::RightAnti
}

const fn is_semi_or_anti(join_type: JoinTypePrimitive) -> bool {
    is_semi(join_type) || is_anti(join_type)
}

pub struct JoinParams {
    /// Indices of the join keys
    pub key_indices: Vec<usize>,
    /// Indices of the distribution keys
    pub dist_keys: Vec<usize>,
}

impl JoinParams {
    pub fn new(key_indices: Vec<usize>, dist_keys: Vec<usize>) -> Self {
        Self {
            key_indices,
            dist_keys,
        }
    }
}

struct JoinSide<K: HashKey, S: StateStore> {
    /// Store all data from a one side stream
    ht: JoinHashMap<K, S>,
    /// Indices of the join key columns
    key_indices: Vec<usize>,
    /// The primary key indices of this side, used for state store
    pk_indices: Vec<usize>,
    /// The date type of each columns to join on
    col_types: Vec<DataType>,
    /// The start position for the side in output new columns
    start_pos: usize,
}

impl<K: HashKey, S: StateStore> std::fmt::Debug for JoinSide<K, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JoinSide")
            .field("key_indices", &self.key_indices)
            .field("pk_indices", &self.pk_indices)
            .field("col_types", &self.col_types)
            .field("start_pos", &self.start_pos)
            .finish()
    }
}

impl<K: HashKey, S: StateStore> JoinSide<K, S> {
    // WARNING: Please do not call this until we implement it.
    fn is_dirty(&self) -> bool {
        unimplemented!()
    }

    #[allow(dead_code)]
    fn clear_cache(&mut self) {
        assert!(
            !self.is_dirty(),
            "cannot clear cache while states of hash join are dirty"
        );

        // TODO: not working with rearranged chain
        // self.ht.clear();
    }
}

/// `HashJoinExecutor` takes two input streams and runs equal hash join on them.
/// The output columns are the concatenation of left and right columns.
pub struct HashJoinExecutor<K: HashKey, S: StateStore, const T: JoinTypePrimitive> {
    /// Left input executor.
    input_l: Option<BoxedExecutor>,
    /// Right input executor.
    input_r: Option<BoxedExecutor>,
    /// the data types of the formed new columns
    output_data_types: Vec<DataType>,
    /// the output indices of the join executor
    output_indices: Vec<usize>,
    /// The schema of the hash join executor
    schema: Schema,
    /// The primary key indices of the schema
    pk_indices: PkIndices,
    /// The parameters of the left join executor
    side_l: JoinSide<K, S>,
    /// The parameters of the right join executor
    side_r: JoinSide<K, S>,
    /// Optional non-equi join conditions
    cond: Option<RowExpression>,
    /// Identity string
    identity: String,
    /// Epoch
    epoch: u64,

    #[allow(dead_code)]
    /// Logical Operator Info
    op_info: String,

    /// Whether the logic can be optimized for append-only stream
    append_only_optimize: bool,

    actor_id: u64,
    metrics: Arc<StreamingMetrics>,
}

impl<K: HashKey, S: StateStore, const T: JoinTypePrimitive> std::fmt::Debug
    for HashJoinExecutor<K, S, T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HashJoinExecutor")
            .field("join_type", &T)
            .field("input_left", &self.input_l.as_ref().unwrap().identity())
            .field("input_right", &self.input_r.as_ref().unwrap().identity())
            .field("side_l", &self.side_l)
            .field("side_r", &self.side_r)
            .field("pk_indices", &self.pk_indices)
            .field("schema", &self.schema)
            .field("output_data_types", &self.output_data_types)
            .finish()
    }
}

impl<K: HashKey, S: StateStore, const T: JoinTypePrimitive> Executor for HashJoinExecutor<K, S, T> {
    fn execute(self: Box<Self>) -> BoxedMessageStream {
        self.into_stream().boxed()
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn pk_indices(&self) -> PkIndicesRef {
        &self.pk_indices
    }

    fn identity(&self) -> &str {
        self.identity.as_str()
    }
}

struct HashJoinChunkBuilder<const T: JoinTypePrimitive, const SIDE: SideTypePrimitive> {
    stream_chunk_builder: StreamChunkBuilder,
}

impl<const T: JoinTypePrimitive, const SIDE: SideTypePrimitive> HashJoinChunkBuilder<T, SIDE> {
    fn with_match_on_insert(
        &mut self,
        row: &RowRef,
        matched_row: &mut JoinRow,
    ) -> StreamExecutorResult<Option<StreamChunk>> {
        // Left/Right Anti sides
        if is_anti(T) {
            if matched_row.is_zero_degree() && only_forward_matched_side(T, SIDE) {
                Ok(self
                    .stream_chunk_builder
                    .append_row_matched(Op::Delete, &matched_row.row)?)
            } else {
                Ok(None)
            }
        // Left/Right Semi sides
        } else if is_semi(T) {
            if matched_row.is_zero_degree() && only_forward_matched_side(T, SIDE) {
                Ok(self
                    .stream_chunk_builder
                    .append_row_matched(Op::Insert, &matched_row.row)?)
            } else {
                Ok(None)
            }
        // Outer sides
        } else if matched_row.is_zero_degree() && outer_side_null(T, SIDE) {
            // if the matched_row does not have any current matches
            // `StreamChunkBuilder` guarantees that `UpdateDelete` will never
            // issue an output chunk.
            if self
                .stream_chunk_builder
                .append_row_matched(Op::UpdateDelete, &matched_row.row)?
                .is_some()
            {
                bail!("`Op::UpdateDelete` should not yield chunk");
            }
            Ok(self
                .stream_chunk_builder
                .append_row(Op::UpdateInsert, row, &matched_row.row)?)
        // Inner sides
        } else {
            Ok(self
                .stream_chunk_builder
                .append_row(Op::Insert, row, &matched_row.row)?)
        }
    }

    fn with_match_on_delete(
        &mut self,
        row: &RowRef,
        matched_row: &mut JoinRow,
    ) -> StreamExecutorResult<Option<StreamChunk>> {
        Ok(
            // Left/Right Anti sides
            if is_anti(T) {
                if matched_row.is_zero_degree() && only_forward_matched_side(T, SIDE) {
                    self.stream_chunk_builder
                        .append_row_matched(Op::Insert, &matched_row.row)?
                } else {
                    None
                }
            // Left/Right Semi sides
            } else if is_semi(T) {
                if matched_row.is_zero_degree() && only_forward_matched_side(T, SIDE) {
                    self.stream_chunk_builder
                        .append_row_matched(Op::Delete, &matched_row.row)?
                } else {
                    None
                }
            // Outer sides
            } else if matched_row.is_zero_degree() && outer_side_null(T, SIDE) {
                // if the matched_row does not have any current
                // matches
                if self
                    .stream_chunk_builder
                    .append_row_matched(Op::UpdateDelete, &matched_row.row)?
                    .is_some()
                {
                    bail!("`Op::UpdateDelete` should not yield chunk");
                }
                self.stream_chunk_builder
                    .append_row_matched(Op::UpdateInsert, &matched_row.row)?
            // Inner sides
            } else {
                // concat with the matched_row and append the new
                // row
                // FIXME: we always use `Op::Delete` here to avoid
                // violating
                // the assumption for U+ after U-.
                self.stream_chunk_builder
                    .append_row(Op::Delete, row, &matched_row.row)?
            },
        )
    }

    #[inline]
    fn forward_exactly_once_if_matched(
        &mut self,
        op: Op,
        row: &RowRef,
    ) -> StreamExecutorResult<Option<StreamChunk>> {
        // if it's a semi join and the side needs to be maintained.
        Ok(if is_semi(T) && forward_exactly_once(T, SIDE) {
            self.stream_chunk_builder.append_row_update(op, row)?
        } else {
            None
        })
    }

    #[inline]
    fn forward_if_not_matched(
        &mut self,
        op: Op,
        row: &RowRef,
    ) -> StreamExecutorResult<Option<StreamChunk>> {
        // if it's outer join or anti join and the side needs to be maintained.
        Ok(
            if (is_anti(T) && forward_exactly_once(T, SIDE)) || is_outer_side(T, SIDE) {
                self.stream_chunk_builder.append_row_update(op, row)?
            } else {
                None
            },
        )
    }

    #[inline]
    fn take(&mut self) -> StreamExecutorResult<Option<StreamChunk>> {
        Ok(self.stream_chunk_builder.take()?)
    }
}

impl<K: HashKey, S: StateStore, const T: JoinTypePrimitive> HashJoinExecutor<K, S, T> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        input_l: BoxedExecutor,
        input_r: BoxedExecutor,
        params_l: JoinParams,
        params_r: JoinParams,
        pk_indices: PkIndices,
        output_indices: Vec<usize>,
        actor_id: u64,
        executor_id: u64,
        cond: Option<RowExpression>,
        op_info: String,
        store_l: S,
        table_id_l: TableId,
        store_r: S,
        table_id_r: TableId,
        is_append_only: bool,
        metrics: Arc<StreamingMetrics>,
    ) -> Self {
        let side_l_column_n = input_l.schema().len();

        let schema_fields = match T {
            JoinType::LeftSemi | JoinType::LeftAnti => input_l.schema().fields.clone(),
            JoinType::RightSemi | JoinType::RightAnti => input_r.schema().fields.clone(),
            _ => [
                input_l.schema().fields.clone(),
                input_r.schema().fields.clone(),
            ]
            .concat(),
        };

        let original_output_data_types: Vec<_> = schema_fields
            .iter()
            .map(|field| field.data_type.clone())
            .collect();
        let col_l_datatypes = input_l
            .schema()
            .fields
            .iter()
            .map(|field| field.data_type.clone())
            .collect_vec();
        let col_r_datatypes = input_r
            .schema()
            .fields
            .iter()
            .map(|field| field.data_type.clone())
            .collect_vec();

        let pk_indices_l = input_l.pk_indices().to_vec();
        let pk_indices_r = input_r.pk_indices().to_vec();

        // check whether join key contains pk in both side
        let append_only_optimize = if is_append_only {
            let join_key_l = HashSet::<usize>::from_iter(params_l.key_indices.clone());
            let join_key_r = HashSet::<usize>::from_iter(params_r.key_indices.clone());
            let pk_contained_l = pk_indices_l.len()
                == pk_indices_l
                    .iter()
                    .filter(|x| join_key_l.contains(x))
                    .count();
            let pk_contained_r = pk_indices_r.len()
                == pk_indices_r
                    .iter()
                    .filter(|x| join_key_r.contains(x))
                    .count();
            pk_contained_l && pk_contained_r
        } else {
            false
        };

        let original_schema = Schema {
            fields: schema_fields,
        };
        let actual_schema: Schema = output_indices
            .iter()
            .map(|&idx| original_schema[idx].clone())
            .collect();
        Self {
            input_l: Some(input_l),
            input_r: Some(input_r),
            output_data_types: original_output_data_types,
            schema: actual_schema,
            side_l: JoinSide {
                ht: JoinHashMap::new(
                    JOIN_CACHE_SIZE,
                    pk_indices_l.clone(),
                    params_l.key_indices.clone(),
                    col_l_datatypes.clone(),
                    store_l,
                    table_id_l,
                    Some(params_l.dist_keys.clone()),
                    metrics.clone(),
                    actor_id,
                    "left",
                ), // TODO: decide the target cap
                key_indices: params_l.key_indices,
                col_types: col_l_datatypes,
                pk_indices: pk_indices_l,
                start_pos: 0,
            },
            side_r: JoinSide {
                ht: JoinHashMap::new(
                    JOIN_CACHE_SIZE,
                    pk_indices_r.clone(),
                    params_r.key_indices.clone(),
                    col_r_datatypes.clone(),
                    store_r,
                    table_id_r,
                    Some(params_r.dist_keys.clone()),
                    metrics.clone(),
                    actor_id,
                    "right",
                ), // TODO: decide the target cap
                key_indices: params_r.key_indices,
                col_types: col_r_datatypes,
                pk_indices: pk_indices_r,
                start_pos: side_l_column_n,
            },
            pk_indices,
            output_indices,
            cond,
            identity: format!("HashJoinExecutor {:X}", executor_id),
            op_info,
            epoch: 0,
            append_only_optimize,
            actor_id,
            metrics,
        }
    }

    #[try_stream(ok = Message, error = StreamExecutorError)]
    async fn into_stream(mut self) {
        let input_l = self.input_l.take().unwrap();
        let input_r = self.input_r.take().unwrap();
        let aligned_stream = barrier_align(
            input_l.execute(),
            input_r.execute(),
            self.actor_id,
            self.metrics.clone(),
        );
        #[for_await]
        for msg in aligned_stream {
            match msg? {
                AlignedMessage::Left(chunk) => {
                    #[for_await]
                    for chunk in Self::eq_join_oneside::<{ SideType::Left }>(
                        &mut self.side_l,
                        &mut self.side_r,
                        &self.output_data_types,
                        &mut self.cond,
                        chunk,
                        self.append_only_optimize,
                    ) {
                        yield chunk.map(|v| match v {
                            Message::Chunk(chunk) => {
                                Message::Chunk(chunk.reorder_columns(&self.output_indices))
                            }
                            barrier @ Message::Barrier(_) => barrier,
                        })?;
                    }
                }
                AlignedMessage::Right(chunk) => {
                    #[for_await]
                    for chunk in Self::eq_join_oneside::<{ SideType::Right }>(
                        &mut self.side_l,
                        &mut self.side_r,
                        &self.output_data_types,
                        &mut self.cond,
                        chunk,
                        self.append_only_optimize,
                    ) {
                        yield chunk.map(|v| match v {
                            Message::Chunk(chunk) => {
                                Message::Chunk(chunk.reorder_columns(&self.output_indices))
                            }
                            barrier @ Message::Barrier(_) => barrier,
                        })?;
                    }
                }
                AlignedMessage::Barrier(barrier) => {
                    self.flush_data().await?;
                    let epoch = barrier.epoch.curr;
                    self.side_l.ht.update_epoch(epoch);
                    self.side_r.ht.update_epoch(epoch);
                    self.epoch = epoch;
                    yield Message::Barrier(barrier);
                }
            }
        }
    }

    async fn flush_data(&mut self) -> StreamExecutorResult<()> {
        self.side_l.ht.flush().await?;
        self.side_r.ht.flush().await?;

        // evict the LRU cache
        // assert!(!self.side_l.is_dirty());
        self.side_l.ht.evict_to_target_cap();
        // assert!(!self.side_r.is_dirty());
        self.side_r.ht.evict_to_target_cap();
        Ok(())
    }

    /// the data the hash table and match the coming
    /// data chunk with the executor state
    async fn hash_eq_match<'a>(
        key: &K,
        ht: &mut JoinHashMap<K, S>,
    ) -> StreamExecutorResult<Option<HashValueType>> {
        if key.has_null() {
            Ok(None)
        } else {
            ht.remove_state(key).await
        }
    }

    fn row_concat(
        row_update: &RowRef<'_>,
        update_start_pos: usize,
        row_matched: &Row,
        matched_start_pos: usize,
    ) -> Row {
        let mut new_row = vec![None; row_update.size() + row_matched.size()];

        for (i, datum_ref) in row_update.values().enumerate() {
            new_row[i + update_start_pos] = datum_ref.to_owned_datum();
        }
        for i in 0..row_matched.size() {
            new_row[i + matched_start_pos] = row_matched[i].clone();
        }
        Row(new_row)
    }

    fn bool_from_array_ref(array_ref: ArrayRef) -> bool {
        let bool_array = array_ref.as_ref().as_bool();
        bool_array.value_at(0).unwrap_or_else(|| {
            panic!(
                "Some thing wrong with the expression result. Bool array: {:?}",
                bool_array
            )
        })
    }

    #[try_stream(ok = Message, error = StreamExecutorError)]
    async fn eq_join_oneside<'a, const SIDE: SideTypePrimitive>(
        mut side_l: &'a mut JoinSide<K, S>,
        mut side_r: &'a mut JoinSide<K, S>,
        output_data_types: &'a [DataType],
        cond: &'a mut Option<RowExpression>,
        chunk: StreamChunk,
        append_only_optimize: bool,
    ) {
        let chunk = chunk.compact()?;
        let (data_chunk, ops) = chunk.into_parts();

        let (side_update, side_match) = if SIDE == SideType::Left {
            (&mut side_l, &mut side_r)
        } else {
            (&mut side_r, &mut side_l)
        };

        let (update_start_pos, matched_start_pos) = if is_semi_or_anti(T) {
            (0, 0)
        } else {
            (side_update.start_pos, side_match.start_pos)
        };

        let mut hashjoin_chunk_builder = HashJoinChunkBuilder::<T, SIDE> {
            stream_chunk_builder: StreamChunkBuilder::new(
                PROCESSING_WINDOW_SIZE,
                output_data_types,
                update_start_pos,
                matched_start_pos,
            )?,
        };

        let mut check_join_condition = |row_update: &RowRef<'_>,
                                        row_matched: &Row|
         -> StreamExecutorResult<bool> {
            // TODO(yuhao-su): We should find a better way to eval the
            // expression without concat
            // two rows.
            let mut cond_match = true;
            // if there are non-equi expressions
            if let Some(ref mut cond) = cond {
                let new_row =
                    Self::row_concat(row_update, update_start_pos, row_matched, matched_start_pos);

                cond_match = Self::bool_from_array_ref(cond.eval(&new_row, output_data_types)?);
            }
            Ok(cond_match)
        };

        let keys = K::build(&side_update.key_indices, &data_chunk)?;
        for (idx, (row, op)) in data_chunk.rows().zip_eq(ops.iter()).enumerate() {
            let key = &keys[idx];
            let value = row.to_owned_row();
            let pk = row.row_by_indices(&side_update.pk_indices);
            let matched_rows = Self::hash_eq_match(key, &mut side_match.ht).await?;
            match *op {
                Op::Insert | Op::UpdateInsert => {
                    let mut degree = 0;
                    let mut append_only_matched_rows = Vec::with_capacity(1);
                    if let Some(mut matched_rows) = matched_rows {
                        for matched_row in matched_rows.values_mut() {
                            if check_join_condition(&row, &matched_row.row)? {
                                degree += 1;
                                if !forward_exactly_once(T, SIDE) {
                                    if let Some(chunk) = hashjoin_chunk_builder
                                        .with_match_on_insert(&row, matched_row)?
                                    {
                                        yield Message::Chunk(chunk);
                                    }
                                }
                                side_match.ht.inc_degree(matched_row)?;
                            }
                            // If the stream is append-only and the join key covers pk in both side,
                            // then we can remove matched rows since pk is unique and will not be
                            // inserted again
                            if append_only_optimize {
                                append_only_matched_rows.push(matched_row.clone());
                            }
                        }
                        if degree == 0 {
                            if let Some(chunk) =
                                hashjoin_chunk_builder.forward_if_not_matched(*op, &row)?
                            {
                                yield Message::Chunk(chunk);
                            }
                        } else if let Some(chunk) =
                            hashjoin_chunk_builder.forward_exactly_once_if_matched(*op, &row)?
                        {
                            yield Message::Chunk(chunk);
                        }
                        // Insert back the state taken from ht.
                        side_match.ht.insert_state(key, matched_rows);
                    } else if let Some(chunk) =
                        hashjoin_chunk_builder.forward_if_not_matched(*op, &row)?
                    {
                        yield Message::Chunk(chunk);
                    }

                    if append_only_optimize {
                        if !append_only_matched_rows.is_empty() {
                            // Since join key contains pk and pk is unique, there should be only
                            // one row if matched
                            debug_assert_eq!(1, append_only_matched_rows.len());
                            let row = append_only_matched_rows.remove(0);
                            let pk = row.row_by_indices(&side_match.pk_indices);
                            side_match.ht.delete(key, pk, row)?;
                        } else {
                            side_update
                                .ht
                                .insert(key, pk, JoinRow::new(value, degree))?;
                        }
                    } else {
                        side_update
                            .ht
                            .insert(key, pk, JoinRow::new(value, degree))?;
                    }
                }
                Op::Delete | Op::UpdateDelete => {
                    let mut degree = 0;
                    if let Some(mut matched_rows) = matched_rows {
                        for matched_row in matched_rows.values_mut() {
                            if check_join_condition(&row, &matched_row.row)? {
                                degree += 1;
                                side_match.ht.dec_degree(matched_row)?;
                                if !forward_exactly_once(T, SIDE) {
                                    if let Some(chunk) = hashjoin_chunk_builder
                                        .with_match_on_delete(&row, matched_row)?
                                    {
                                        yield Message::Chunk(chunk);
                                    }
                                }
                            }
                        }
                        if degree == 0 {
                            if let Some(chunk) =
                                hashjoin_chunk_builder.forward_if_not_matched(*op, &row)?
                            {
                                yield Message::Chunk(chunk);
                            }
                        } else if let Some(chunk) =
                            hashjoin_chunk_builder.forward_exactly_once_if_matched(*op, &row)?
                        {
                            yield Message::Chunk(chunk);
                        }
                        // Insert back the state taken from ht.
                        side_match.ht.insert_state(key, matched_rows);
                    } else if let Some(chunk) =
                        hashjoin_chunk_builder.forward_if_not_matched(*op, &row)?
                    {
                        yield Message::Chunk(chunk);
                    }
                    side_update
                        .ht
                        .delete(key, pk, JoinRow::new(value, degree))?;
                }
            }
        }
        if let Some(chunk) = hashjoin_chunk_builder.take()? {
            yield Message::Chunk(chunk);
        }
    }
}

#[cfg(test)]
mod tests {
    use piestream_common::array::stream_chunk::StreamChunkTestExt;
    use piestream_common::array::*;
    use piestream_common::catalog::{Field, Schema, TableId};
    use piestream_common::hash::{Key128, Key64};
    use piestream_expr::expr::expr_binary_nonnull::new_binary_expr;
    use piestream_expr::expr::{InputRefExpression, RowExpression};
    use piestream_pb::expr::expr_node::Type;
    use piestream_storage::memory::MemoryStateStore;

    use super::*;
    use crate::executor::test_utils::{MessageSender, MockSource};
    use crate::executor::{Barrier, Epoch, Message};

    fn create_cond() -> RowExpression {
        let left_expr = InputRefExpression::new(DataType::Int64, 1);
        let right_expr = InputRefExpression::new(DataType::Int64, 3);
        let cond = new_binary_expr(
            Type::LessThan,
            DataType::Boolean,
            Box::new(left_expr),
            Box::new(right_expr),
        );
        RowExpression::new(cond)
    }

    fn create_executor<const T: JoinTypePrimitive>(
        with_condition: bool,
    ) -> (MessageSender, MessageSender, BoxedMessageStream) {
        let schema = Schema {
            fields: vec![
                Field::unnamed(DataType::Int64), // join key
                Field::unnamed(DataType::Int64),
            ],
        };
        let (tx_l, source_l) = MockSource::channel(schema.clone(), vec![0, 1]);
        let (tx_r, source_r) = MockSource::channel(schema, vec![0, 1]);
        let params_l = JoinParams::new(vec![0], vec![]);
        let params_r = JoinParams::new(vec![0], vec![]);
        let cond = with_condition.then(create_cond);
        let mem_state = MemoryStateStore::new();

        let schema_len = match T {
            JoinType::LeftSemi | JoinType::LeftAnti => source_l.schema().len(),
            JoinType::RightSemi | JoinType::RightAnti => source_r.schema().len(),
            _ => source_l.schema().len() + source_r.schema().len(),
        };
        let executor = HashJoinExecutor::<Key64, MemoryStateStore, T>::new(
            Box::new(source_l),
            Box::new(source_r),
            params_l,
            params_r,
            vec![1],
            (0..schema_len).into_iter().collect_vec(),
            1,
            1,
            cond,
            "HashJoinExecutor".to_string(),
            mem_state.clone(),
            TableId::new(0),
            mem_state,
            TableId::new(1),
            false,
            Arc::new(StreamingMetrics::unused()),
        );
        (tx_l, tx_r, Box::new(executor).execute())
    }

    fn create_append_only_executor<const T: JoinTypePrimitive>(
        with_condition: bool,
    ) -> (MessageSender, MessageSender, BoxedMessageStream) {
        let schema = Schema {
            fields: vec![
                Field::unnamed(DataType::Int64),
                Field::unnamed(DataType::Int64),
                Field::unnamed(DataType::Int64),
            ],
        };
        let (tx_l, source_l) = MockSource::channel(schema.clone(), vec![0]);
        let (tx_r, source_r) = MockSource::channel(schema, vec![0]);
        let params_l = JoinParams::new(vec![0, 1], vec![]);
        let params_r = JoinParams::new(vec![0, 1], vec![]);
        let cond = with_condition.then(create_cond);
        let mem_store = MemoryStateStore::new();
        let schema_len = match T {
            JoinType::LeftSemi | JoinType::LeftAnti => source_l.schema().len(),
            JoinType::RightSemi | JoinType::RightAnti => source_r.schema().len(),
            _ => source_l.schema().len() + source_r.schema().len(),
        };
        let executor = HashJoinExecutor::<Key128, MemoryStateStore, T>::new(
            Box::new(source_l),
            Box::new(source_r),
            params_l,
            params_r,
            vec![1],
            (0..schema_len).into_iter().collect_vec(),
            1,
            1,
            cond,
            "HashJoinExecutor".to_string(),
            mem_store.clone(),
            TableId::new(0),
            mem_store,
            TableId::new(1),
            true,
            Arc::new(StreamingMetrics::unused()),
        );
        (tx_l, tx_r, Box::new(executor).execute())
    }

    #[tokio::test]
    async fn test_streaming_hash_inner_join() {
        let chunk_l1 = StreamChunk::from_pretty(
            "  I I
             + 1 4
             + 2 5
             + 3 6",
        );
        let chunk_l2 = StreamChunk::from_pretty(
            "  I I
             + 3 8
             - 3 8",
        );
        let chunk_r1 = StreamChunk::from_pretty(
            "  I I
             + 2 7
             + 4 8
             + 6 9",
        );
        let chunk_r2 = StreamChunk::from_pretty(
            "  I  I
             + 3 10
             + 6 11",
        );
        let (mut tx_l, mut tx_r, mut hash_join) = create_executor::<{ JoinType::Inner }>(false);

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 1st left chunk
        tx_l.push_chunk(chunk_l1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty("I I I I")
        );

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 2nd left chunk
        tx_l.push_chunk(chunk_l2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty("I I I I")
        );

        // push the 1st right chunk
        tx_r.push_chunk(chunk_r1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I
                + 2 5 2 7"
            )
        );

        // push the 2nd right chunk
        tx_r.push_chunk(chunk_r2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I
                + 3 6 3 10"
            )
        );
    }

    #[tokio::test]
    async fn test_streaming_hash_left_semi_join() {
        let chunk_l1 = StreamChunk::from_pretty(
            "  I I
             + 1 4
             + 2 5
             + 3 6",
        );
        let chunk_l2 = StreamChunk::from_pretty(
            "  I I
             + 3 8
             - 3 8",
        );
        let chunk_r1 = StreamChunk::from_pretty(
            "  I I
             + 2 7
             + 4 8
             + 6 9",
        );
        let chunk_r2 = StreamChunk::from_pretty(
            "  I  I
             + 3 10
             + 6 11",
        );
        let chunk_l3 = StreamChunk::from_pretty(
            "  I I
             + 6 10",
        );
        let chunk_r3 = StreamChunk::from_pretty(
            "  I  I
             - 6 11",
        );
        let chunk_r4 = StreamChunk::from_pretty(
            "  I  I
             - 6 9",
        );
        let (mut tx_l, mut tx_r, mut hash_join) = create_executor::<{ JoinType::LeftSemi }>(false);

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 1st left chunk
        tx_l.push_chunk(chunk_l1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(chunk.into_chunk().unwrap(), StreamChunk::from_pretty("I I"));

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 2nd left chunk
        tx_l.push_chunk(chunk_l2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(chunk.into_chunk().unwrap(), StreamChunk::from_pretty("I I"));

        // push the 1st right chunk
        tx_r.push_chunk(chunk_r1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I
                + 2 5"
            )
        );

        // push the 2nd right chunk
        tx_r.push_chunk(chunk_r2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I
                + 3 6"
            )
        );

        // push the 3rd left chunk (tests forward_exactly_once)
        tx_l.push_chunk(chunk_l3);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I
                + 6 10"
            )
        );

        // push the 3rd right chunk
        // (tests that no change if there are still matches)
        tx_r.push_chunk(chunk_r3);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(chunk.into_chunk().unwrap(), StreamChunk::from_pretty("I I"));

        // push the 3rd left chunk
        // (tests that deletion occurs when there are no more matches)
        tx_r.push_chunk(chunk_r4);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I
                - 6 10"
            )
        );
    }

    #[tokio::test]
    async fn test_streaming_hash_inner_join_append_only() {
        let chunk_l1 = StreamChunk::from_pretty(
            "  I I I
             + 1 4 1
             + 2 5 2
             + 3 6 3",
        );
        let chunk_l2 = StreamChunk::from_pretty(
            "  I I I
             + 4 9 4
             + 5 10 5",
        );
        let chunk_r1 = StreamChunk::from_pretty(
            "  I I I
             + 2 5 1
             + 4 9 2
             + 6 9 3",
        );
        let chunk_r2 = StreamChunk::from_pretty(
            "  I I I
             + 1 4 4
             + 3 6 5",
        );

        let (mut tx_l, mut tx_r, mut hash_join) =
            create_append_only_executor::<{ JoinType::Inner }>(false);

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 1st left chunk
        tx_l.push_chunk(chunk_l1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty("I I I I I I")
        );

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 2nd left chunk
        tx_l.push_chunk(chunk_l2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty("I I I I I I")
        );

        // push the 1st right chunk
        tx_r.push_chunk(chunk_r1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I I I
                + 2 5 2 2 5 1
                + 4 9 4 4 9 2"
            )
        );

        // push the 2nd right chunk
        tx_r.push_chunk(chunk_r2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I I I
                + 1 4 1 1 4 4
                + 3 6 3 3 6 5"
            )
        );
    }

    #[tokio::test]
    async fn test_streaming_hash_left_semi_join_append_only() {
        let chunk_l1 = StreamChunk::from_pretty(
            "  I I I
             + 1 4 1
             + 2 5 2
             + 3 6 3",
        );
        let chunk_l2 = StreamChunk::from_pretty(
            "  I I I
             + 4 9 4
             + 5 10 5",
        );
        let chunk_r1 = StreamChunk::from_pretty(
            "  I I I
             + 2 5 1
             + 4 9 2
             + 6 9 3",
        );
        let chunk_r2 = StreamChunk::from_pretty(
            "  I I I
             + 1 4 4
             + 3 6 5",
        );

        let (mut tx_l, mut tx_r, mut hash_join) =
            create_append_only_executor::<{ JoinType::LeftSemi }>(false);

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 1st left chunk
        tx_l.push_chunk(chunk_l1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty("I I I")
        );

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 2nd left chunk
        tx_l.push_chunk(chunk_l2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty("I I I")
        );

        // push the 1st right chunk
        tx_r.push_chunk(chunk_r1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I
                + 2 5 2
                + 4 9 4"
            )
        );

        // push the 2nd right chunk
        tx_r.push_chunk(chunk_r2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I
                + 1 4 1
                + 3 6 3"
            )
        );
    }

    #[tokio::test]
    async fn test_streaming_hash_right_semi_join_append_only() {
        let chunk_l1 = StreamChunk::from_pretty(
            "  I I I
             + 1 4 1
             + 2 5 2
             + 3 6 3",
        );
        let chunk_l2 = StreamChunk::from_pretty(
            "  I I I
             + 4 9 4
             + 5 10 5",
        );
        let chunk_r1 = StreamChunk::from_pretty(
            "  I I I
             + 2 5 1
             + 4 9 2
             + 6 9 3",
        );
        let chunk_r2 = StreamChunk::from_pretty(
            "  I I I
             + 1 4 4
             + 3 6 5",
        );

        let (mut tx_l, mut tx_r, mut hash_join) =
            create_append_only_executor::<{ JoinType::RightSemi }>(false);

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 1st left chunk
        tx_l.push_chunk(chunk_l1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty("I I I")
        );

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 2nd left chunk
        tx_l.push_chunk(chunk_l2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty("I I I")
        );

        // push the 1st right chunk
        tx_r.push_chunk(chunk_r1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I
                + 2 5 1
                + 4 9 2"
            )
        );

        // push the 2nd right chunk
        tx_r.push_chunk(chunk_r2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I
                + 1 4 4
                + 3 6 5"
            )
        );
    }

    #[tokio::test]
    async fn test_streaming_hash_right_semi_join() {
        let chunk_r1 = StreamChunk::from_pretty(
            "  I I
             + 1 4
             + 2 5
             + 3 6",
        );
        let chunk_r2 = StreamChunk::from_pretty(
            "  I I
             + 3 8
             - 3 8",
        );
        let chunk_l1 = StreamChunk::from_pretty(
            "  I I
             + 2 7
             + 4 8
             + 6 9",
        );
        let chunk_l2 = StreamChunk::from_pretty(
            "  I  I
             + 3 10
             + 6 11",
        );
        let chunk_r3 = StreamChunk::from_pretty(
            "  I I
             + 6 10",
        );
        let chunk_l3 = StreamChunk::from_pretty(
            "  I  I
             - 6 11",
        );
        let chunk_l4 = StreamChunk::from_pretty(
            "  I  I
             - 6 9",
        );
        let (mut tx_l, mut tx_r, mut hash_join) = create_executor::<{ JoinType::RightSemi }>(false);

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 1st right chunk
        tx_r.push_chunk(chunk_r1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(chunk.into_chunk().unwrap(), StreamChunk::from_pretty("I I"));

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 2nd right chunk
        tx_r.push_chunk(chunk_r2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(chunk.into_chunk().unwrap(), StreamChunk::from_pretty("I I"));

        // push the 1st left chunk
        tx_l.push_chunk(chunk_l1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I
                + 2 5"
            )
        );

        // push the 2nd left chunk
        tx_l.push_chunk(chunk_l2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I
                + 3 6"
            )
        );

        // push the 3rd right chunk (tests forward_exactly_once)
        tx_r.push_chunk(chunk_r3);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I
                + 6 10"
            )
        );

        // push the 3rd left chunk
        // (tests that no change if there are still matches)
        tx_l.push_chunk(chunk_l3);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(chunk.into_chunk().unwrap(), StreamChunk::from_pretty("I I"));

        // push the 3rd right chunk
        // (tests that deletion occurs when there are no more matches)
        tx_l.push_chunk(chunk_l4);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I
                - 6 10"
            )
        );
    }

    #[tokio::test]
    async fn test_streaming_hash_left_anti_join() {
        let chunk_l1 = StreamChunk::from_pretty(
            "  I I
             + 1 4
             + 2 5
             + 3 6",
        );
        let chunk_l2 = StreamChunk::from_pretty(
            "  I I
             + 3 8
             - 3 8",
        );
        let chunk_r1 = StreamChunk::from_pretty(
            "  I I
             + 2 7
             + 4 8
             + 6 9",
        );
        let chunk_r2 = StreamChunk::from_pretty(
            "  I  I
             + 3 10
             + 6 11
             + 1 2
             + 1 3",
        );
        let chunk_l3 = StreamChunk::from_pretty(
            "  I I
             + 9 10",
        );
        let chunk_r3 = StreamChunk::from_pretty(
            "  I I
             - 1 2",
        );
        let chunk_r4 = StreamChunk::from_pretty(
            "  I I
             - 1 3",
        );
        let (mut tx_l, mut tx_r, mut hash_join) = create_executor::<{ JoinType::LeftAnti }>(false);

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 1st left chunk
        tx_l.push_chunk(chunk_l1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I
                + 1 4
                + 2 5
                + 3 6",
            )
        );

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 2nd left chunk
        tx_l.push_chunk(chunk_l2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                "  I I
                 + 3 8
                 - 3 8",
            )
        );

        // push the 1st right chunk
        tx_r.push_chunk(chunk_r1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I
                - 2 5"
            )
        );

        // push the 2nd right chunk
        tx_r.push_chunk(chunk_r2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I
                - 3 6
                - 1 4"
            )
        );

        // push the 3rd left chunk (tests forward_exactly_once)
        tx_l.push_chunk(chunk_l3);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I
                + 9 10"
            )
        );

        // push the 3rd right chunk
        // (tests that no change if there are still matches)
        tx_r.push_chunk(chunk_r3);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(chunk.into_chunk().unwrap(), StreamChunk::from_pretty("I I"));

        // push the 4th right chunk
        // (tests that insertion occurs when there are no more matches)
        tx_r.push_chunk(chunk_r4);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I
                + 1 4"
            )
        );
    }

    #[tokio::test]
    async fn test_streaming_hash_right_anti_join() {
        let chunk_r1 = StreamChunk::from_pretty(
            "  I I
             + 1 4
             + 2 5
             + 3 6",
        );
        let chunk_r2 = StreamChunk::from_pretty(
            "  I I
             + 3 8
             - 3 8",
        );
        let chunk_l1 = StreamChunk::from_pretty(
            "  I I
             + 2 7
             + 4 8
             + 6 9",
        );
        let chunk_l2 = StreamChunk::from_pretty(
            "  I  I
             + 3 10
             + 6 11
             + 1 2
             + 1 3",
        );
        let chunk_r3 = StreamChunk::from_pretty(
            "  I I
             + 9 10",
        );
        let chunk_l3 = StreamChunk::from_pretty(
            "  I I
             - 1 2",
        );
        let chunk_l4 = StreamChunk::from_pretty(
            "  I I
             - 1 3",
        );
        let (mut tx_r, mut tx_l, mut hash_join) = create_executor::<{ JoinType::LeftAnti }>(false);

        // push the init barrier for left and right
        tx_r.push_barrier(1, false);
        tx_l.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 1st right chunk
        tx_r.push_chunk(chunk_r1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I
                + 1 4
                + 2 5
                + 3 6",
            )
        );

        // push the init barrier for left and right
        tx_r.push_barrier(1, false);
        tx_l.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 2nd right chunk
        tx_r.push_chunk(chunk_r2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                "  I I
                 + 3 8
                 - 3 8",
            )
        );

        // push the 1st left chunk
        tx_l.push_chunk(chunk_l1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I
                - 2 5"
            )
        );

        // push the 2nd left chunk
        tx_l.push_chunk(chunk_l2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I
                - 3 6
                - 1 4"
            )
        );

        // push the 3rd right chunk (tests forward_exactly_once)
        tx_r.push_chunk(chunk_r3);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I
                + 9 10"
            )
        );

        // push the 3rd left chunk
        // (tests that no change if there are still matches)
        tx_l.push_chunk(chunk_l3);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(chunk.into_chunk().unwrap(), StreamChunk::from_pretty("I I"));

        // push the 4th left chunk
        // (tests that insertion occurs when there are no more matches)
        tx_l.push_chunk(chunk_l4);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I
                + 1 4"
            )
        );
    }

    #[tokio::test]
    async fn test_streaming_hash_inner_join_with_barrier() {
        let chunk_l1 = StreamChunk::from_pretty(
            "  I I
             + 1 4
             + 2 5
             + 3 6",
        );
        let chunk_l2 = StreamChunk::from_pretty(
            "  I I
             + 6 8
             + 3 8",
        );
        let chunk_r1 = StreamChunk::from_pretty(
            "  I I
             + 2 7
             + 4 8
             + 6 9",
        );
        let chunk_r2 = StreamChunk::from_pretty(
            "  I  I
             + 3 10
             + 6 11",
        );
        let (mut tx_l, mut tx_r, mut hash_join) = create_executor::<{ JoinType::Inner }>(false);

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 1st left chunk
        tx_l.push_chunk(chunk_l1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty("I I I I")
        );

        // push a barrier to left side
        tx_l.push_barrier(2, false);

        // push the 2nd left chunk
        tx_l.push_chunk(chunk_l2);

        // join the first right chunk
        tx_r.push_chunk(chunk_r1);

        // Consume stream chunk
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I
                + 2 5 2 7"
            )
        );

        // push a barrier to right side
        tx_r.push_barrier(2, false);

        // get the aligned barrier here
        let expected_epoch = Epoch::new_test_epoch(2);
        assert!(matches!(
            hash_join.next().await.unwrap().unwrap(),
            Message::Barrier(Barrier {
                epoch,
                mutation: None,
                ..
            }) if epoch == expected_epoch
        ));

        // join the 2nd left chunk
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I
                + 6 8 6 9"
            )
        );

        // push the 2nd right chunk
        tx_r.push_chunk(chunk_r2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I
                + 3 6 3 10
                + 3 8 3 10
                + 6 8 6 11"
            )
        );
    }

    #[tokio::test]
    async fn test_streaming_hash_inner_join_with_null_and_barrier() {
        let chunk_l1 = StreamChunk::from_pretty(
            "  I I
             + 1 4
             + 2 .
             + 3 .",
        );
        let chunk_l2 = StreamChunk::from_pretty(
            "  I I
             + 6 .
             + 3 8",
        );
        let chunk_r1 = StreamChunk::from_pretty(
            "  I I
             + 2 7
             + 4 8
             + 6 9",
        );
        let chunk_r2 = StreamChunk::from_pretty(
            "  I  I
             + 3 10
             + 6 11",
        );
        let (mut tx_l, mut tx_r, mut hash_join) = create_executor::<{ JoinType::Inner }>(false);

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 1st left chunk
        tx_l.push_chunk(chunk_l1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty("I I I I")
        );

        // push a barrier to left side
        tx_l.push_barrier(2, false);

        // push the 2nd left chunk
        tx_l.push_chunk(chunk_l2);

        // join the first right chunk
        tx_r.push_chunk(chunk_r1);

        // Consume stream chunk
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I
                + 2 . 2 7"
            )
        );

        // push a barrier to right side
        tx_r.push_barrier(2, false);

        // get the aligned barrier here
        let expected_epoch = Epoch::new_test_epoch(2);
        assert!(matches!(
            hash_join.next().await.unwrap().unwrap(),
            Message::Barrier(Barrier {
                epoch,
                mutation: None,
                ..
            }) if epoch == expected_epoch
        ));

        // join the 2nd left chunk
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I
                + 6 . 6 9"
            )
        );

        // push the 2nd right chunk
        tx_r.push_chunk(chunk_r2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I
                + 3 . 3 10
                + 3 8 3 10
                + 6 . 6 11"
            )
        );
    }

    #[tokio::test]
    async fn test_streaming_hash_left_join() {
        let chunk_l1 = StreamChunk::from_pretty(
            "  I I
             + 1 4
             + 2 5
             + 3 6",
        );
        let chunk_l2 = StreamChunk::from_pretty(
            "  I I
             + 3 8
             - 3 8",
        );
        let chunk_r1 = StreamChunk::from_pretty(
            "  I I
             + 2 7
             + 4 8
             + 6 9",
        );
        let chunk_r2 = StreamChunk::from_pretty(
            "  I  I
             + 3 10
             + 6 11",
        );
        let (mut tx_l, mut tx_r, mut hash_join) = create_executor::<{ JoinType::LeftOuter }>(false);

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 1st left chunk
        tx_l.push_chunk(chunk_l1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I
                + 1 4 . .
                + 2 5 . .
                + 3 6 . ."
            )
        );

        // push the 2nd left chunk
        tx_l.push_chunk(chunk_l2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I
                + 3 8 . .
                - 3 8 . ."
            )
        );

        // push the 1st right chunk
        tx_r.push_chunk(chunk_r1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                "  I I I I
                U- 2 5 . .
                U+ 2 5 2 7"
            )
        );

        // push the 2nd right chunk
        tx_r.push_chunk(chunk_r2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                "  I I I I
                U- 3 6 . .
                U+ 3 6 3 10"
            )
        );
    }

    #[tokio::test]
    async fn test_streaming_hash_right_join() {
        let chunk_l1 = StreamChunk::from_pretty(
            "  I I
             + 1 4
             + 2 5
             + 3 6",
        );
        let chunk_l2 = StreamChunk::from_pretty(
            "  I I
             + 3 8
             - 3 8",
        );
        let chunk_r1 = StreamChunk::from_pretty(
            "  I I
             + 2 7
             + 4 8
             + 6 9",
        );
        let chunk_r2 = StreamChunk::from_pretty(
            "  I  I
             + 5 10
             - 5 10",
        );
        let (mut tx_l, mut tx_r, mut hash_join) =
            create_executor::<{ JoinType::RightOuter }>(false);

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 1st left chunk
        tx_l.push_chunk(chunk_l1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty("I I I I")
        );

        // push the 2nd left chunk
        tx_l.push_chunk(chunk_l2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty("I I I I")
        );

        // push the 1st right chunk
        tx_r.push_chunk(chunk_r1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I
                + 2 5 2 7
                + . . 4 8
                + . . 6 9"
            )
        );

        // push the 2nd right chunk
        tx_r.push_chunk(chunk_r2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I
                + . . 5 10
                - . . 5 10"
            )
        );
    }

    #[tokio::test]
    async fn test_streaming_hash_left_join_append_only() {
        let chunk_l1 = StreamChunk::from_pretty(
            "  I I I
             + 1 4 1
             + 2 5 2
             + 3 6 3",
        );
        let chunk_l2 = StreamChunk::from_pretty(
            "  I I I
             + 4 9 4
             + 5 10 5",
        );
        let chunk_r1 = StreamChunk::from_pretty(
            "  I I I
             + 2 5 1
             + 4 9 2
             + 6 9 3",
        );
        let chunk_r2 = StreamChunk::from_pretty(
            "  I I I
             + 1 4 4
             + 3 6 5",
        );

        let (mut tx_l, mut tx_r, mut hash_join) =
            create_append_only_executor::<{ JoinType::LeftOuter }>(false);

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 1st left chunk
        tx_l.push_chunk(chunk_l1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I I I
                + 1 4 1 . . .
                + 2 5 2 . . .
                + 3 6 3 . . ."
            )
        );

        // push the 2nd left chunk
        tx_l.push_chunk(chunk_l2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I I I
                + 4 9 4 . . .
                + 5 10 5 . . ."
            )
        );

        // push the 1st right chunk
        tx_r.push_chunk(chunk_r1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                "  I I I I I I
                U- 2 5 2 . . .
                U+ 2 5 2 2 5 1
                U- 4 9 4 . . .
                U+ 4 9 4 4 9 2"
            )
        );

        // push the 2nd right chunk
        tx_r.push_chunk(chunk_r2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                "  I I I I I I
                U- 1 4 1 . . .
                U+ 1 4 1 1 4 4
                U- 3 6 3 . . .
                U+ 3 6 3 3 6 5"
            )
        );
    }

    #[tokio::test]
    async fn test_streaming_hash_right_join_append_only() {
        let chunk_l1 = StreamChunk::from_pretty(
            "  I I I
             + 1 4 1
             + 2 5 2
             + 3 6 3",
        );
        let chunk_l2 = StreamChunk::from_pretty(
            "  I I I
             + 4 9 4
             + 5 10 5",
        );
        let chunk_r1 = StreamChunk::from_pretty(
            "  I I I
             + 2 5 1
             + 4 9 2
             + 6 9 3",
        );
        let chunk_r2 = StreamChunk::from_pretty(
            "  I I I
             + 1 4 4
             + 3 6 5
             + 7 7 6",
        );

        let (mut tx_l, mut tx_r, mut hash_join) =
            create_append_only_executor::<{ JoinType::RightOuter }>(false);

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 1st left chunk
        tx_l.push_chunk(chunk_l1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty("I I I I I I")
        );

        // push the 2nd left chunk
        tx_l.push_chunk(chunk_l2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty("I I I I I I")
        );

        // push the 1st right chunk
        tx_r.push_chunk(chunk_r1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                "  I I I I I I
                + 2 5 2 2 5 1
                + 4 9 4 4 9 2
                + . . . 6 9 3"
            )
        );

        // push the 2nd right chunk
        tx_r.push_chunk(chunk_r2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                "  I I I I I I
                + 1 4 1 1 4 4
                + 3 6 3 3 6 5
                + . . . 7 7 6"
            )
        );
    }

    #[tokio::test]
    async fn test_streaming_hash_full_outer_join() {
        let chunk_l1 = StreamChunk::from_pretty(
            "  I I
             + 1 4
             + 2 5
             + 3 6",
        );
        let chunk_l2 = StreamChunk::from_pretty(
            "  I I
             + 3 8
             - 3 8",
        );
        let chunk_r1 = StreamChunk::from_pretty(
            "  I I
             + 2 7
             + 4 8
             + 6 9",
        );
        let chunk_r2 = StreamChunk::from_pretty(
            "  I  I
             + 5 10
             - 5 10",
        );
        let (mut tx_l, mut tx_r, mut hash_join) = create_executor::<{ JoinType::FullOuter }>(false);

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 1st left chunk
        tx_l.push_chunk(chunk_l1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I
                + 1 4 . .
                + 2 5 . .
                + 3 6 . ."
            )
        );

        // push the 2nd left chunk
        tx_l.push_chunk(chunk_l2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I
                + 3 8 . .
                - 3 8 . ."
            )
        );

        // push the 1st right chunk
        tx_r.push_chunk(chunk_r1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                "  I I I I
                U- 2 5 . .
                U+ 2 5 2 7
                +  . . 4 8
                +  . . 6 9"
            )
        );

        // push the 2nd right chunk
        tx_r.push_chunk(chunk_r2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I
                + . . 5 10
                - . . 5 10"
            )
        );
    }

    #[tokio::test]
    async fn test_streaming_hash_full_outer_join_with_nonequi_condition() {
        let chunk_l1 = StreamChunk::from_pretty(
            "  I I
             + 1 4
             + 2 5
             + 3 6
             + 3 7",
        );
        let chunk_l2 = StreamChunk::from_pretty(
            "  I I
             + 3 8
             - 3 8
             - 1 4", // delete row to cause an empty JoinHashEntry
        );
        let chunk_r1 = StreamChunk::from_pretty(
            "  I I
             + 2 6
             + 4 8
             + 3 4",
        );
        let chunk_r2 = StreamChunk::from_pretty(
            "  I  I
             + 5 10
             - 5 10
             + 1 2",
        );
        let (mut tx_l, mut tx_r, mut hash_join) = create_executor::<{ JoinType::FullOuter }>(true);

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 1st left chunk
        tx_l.push_chunk(chunk_l1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I
                + 1 4 . .
                + 2 5 . .
                + 3 6 . .
                + 3 7 . ."
            )
        );

        // push the 2nd left chunk
        tx_l.push_chunk(chunk_l2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I
                + 3 8 . .
                - 3 8 . .
                - 1 4 . ."
            )
        );

        // push the 1st right chunk
        tx_r.push_chunk(chunk_r1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                "  I I I I
                U- 2 5 . .
                U+ 2 5 2 6
                +  . . 4 8
                +  . . 3 4" /* regression test (#2420): 3 4 should be forwarded only once
                             * despite matching on eq join on 2
                             * entries */
            )
        );

        // push the 2nd right chunk
        tx_r.push_chunk(chunk_r2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I
                + . . 5 10
                - . . 5 10
                + . . 1 2" /* regression test (#2420): 1 2 forwarded even if matches on an empty
                            * join entry */
            )
        );
    }

    #[tokio::test]
    async fn test_streaming_hash_inner_join_with_nonequi_condition() {
        let chunk_l1 = StreamChunk::from_pretty(
            "  I I
             + 1 4
             + 2 10
             + 3 6",
        );
        let chunk_l2 = StreamChunk::from_pretty(
            "  I I
             + 3 8
             - 3 8",
        );
        let chunk_r1 = StreamChunk::from_pretty(
            "  I I
             + 2 7
             + 4 8
             + 6 9",
        );
        let chunk_r2 = StreamChunk::from_pretty(
            "  I  I
             + 3 10
             + 6 11",
        );
        let (mut tx_l, mut tx_r, mut hash_join) = create_executor::<{ JoinType::Inner }>(true);

        // push the init barrier for left and right
        tx_l.push_barrier(1, false);
        tx_r.push_barrier(1, false);
        hash_join.next().await.unwrap().unwrap();

        // push the 1st left chunk
        tx_l.push_chunk(chunk_l1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty("I I I I")
        );

        // push the 2nd left chunk
        tx_l.push_chunk(chunk_l2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty("I I I I")
        );

        // push the 1st right chunk
        tx_r.push_chunk(chunk_r1);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty("I I I I")
        );

        // push the 2nd right chunk
        tx_r.push_chunk(chunk_r2);
        let chunk = hash_join.next().await.unwrap().unwrap();
        assert_eq!(
            chunk.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I I I I
                + 3 6 3 10"
            )
        );
    }
}
