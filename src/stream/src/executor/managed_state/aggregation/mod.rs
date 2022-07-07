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

//! Aggregators with state store support

pub use extreme::*;
use piestream_common::array::stream_chunk::Ops;
use piestream_common::array::{ArrayImpl, Row};
use piestream_common::buffer::Bitmap;
use piestream_common::hash::HashCode;
use piestream_common::types::Datum;
use piestream_expr::expr::AggKind;
use piestream_storage::table::state_table::StateTable;
use piestream_storage::StateStore;
pub use value::*;

use crate::executor::aggregation::AggCall;
use crate::executor::error::{StreamExecutorError, StreamExecutorResult};
use crate::executor::PkDataTypes;

mod extreme;

#[allow(dead_code)]
mod string_agg;
mod value;

/// Verify if the data going through the state is valid by checking if `ops.len() ==
/// visibility.len() == data[x].len()`.
pub fn verify_batch(
    ops: piestream_common::array::stream_chunk::Ops<'_>,
    visibility: Option<&piestream_common::buffer::Bitmap>,
    data: &[&piestream_common::array::ArrayImpl],
) -> bool {
    let mut all_lengths = vec![ops.len()];
    if let Some(visibility) = visibility {
        all_lengths.push(visibility.len());
    }
    all_lengths.extend(data.iter().map(|x| x.len()));
    all_lengths.iter().min() == all_lengths.iter().max()
}

/// All managed state for aggregation. The managed state will manage the cache and integrate
/// the state with the underlying state store. Managed states can only be evicted from outer cache
/// when they are not dirty.
pub enum ManagedStateImpl<S: StateStore> {
    /// States as single scalar value e.g. `COUNT`, `SUM`
    Value(ManagedValueState),

    /// States as table structure e.g. `MAX`, `STRING_AGG`
    Table(Box<dyn ManagedTableState<S>>),
}

impl<S: StateStore> ManagedStateImpl<S> {
    pub async fn apply_batch(
        &mut self,
        ops: Ops<'_>,
        visibility: Option<&Bitmap>,
        data: &[&ArrayImpl],
        epoch: u64,
        state_table: &mut StateTable<S>,
    ) -> StreamExecutorResult<()> {
        match self {
            Self::Value(state) => state.apply_batch(ops, visibility, data).await,
            Self::Table(state) => {
                state
                    .apply_batch(ops, visibility, data, epoch, state_table)
                    .await
            }
        }
    }

    /// Get the output of the state. Must flush before getting output.
    pub async fn get_output(
        &mut self,
        epoch: u64,
        state_table: &StateTable<S>,
    ) -> StreamExecutorResult<Datum> {
        match self {
            Self::Value(state) => state.get_output().await,
            Self::Table(state) => state.get_output(epoch, state_table).await,
        }
    }

    /// Check if this state needs a flush.
    pub fn is_dirty(&self) -> bool {
        match self {
            Self::Value(state) => state.is_dirty(),
            Self::Table(state) => state.is_dirty(),
        }
    }

    /// Flush the internal state to a write batch.
    pub async fn flush(&mut self, state_table: &mut StateTable<S>) -> StreamExecutorResult<()> {
        match self {
            Self::Value(state) => state.flush(state_table).await,
            Self::Table(state) => state.flush(state_table),
        }
    }

    /// Create a managed state from `agg_call`.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_managed_state(
        agg_call: AggCall,
        row_count: Option<usize>,
        pk_data_types: PkDataTypes,
        is_row_count: bool,
        key_hash_code: Option<HashCode>,
        pk: Option<&Row>,
        state_table: &StateTable<S>,
    ) -> StreamExecutorResult<Self> {
        match agg_call.kind {
            AggKind::Max | AggKind::Min => {
                assert!(
                    row_count.is_some(),
                    "should set row_count for value states other than AggKind::RowCount"
                );

                // optimization: use single-value state for append-only min/max
                if agg_call.append_only {
                    Ok(Self::Value(
                        ManagedValueState::new(agg_call, row_count, pk, state_table).await?,
                    ))
                } else {
                    Ok(Self::Table(
                        create_streaming_extreme_state(
                            agg_call,
                            row_count.unwrap(),
                            // TODO: estimate a good cache size instead of hard-coding
                            Some(1024),
                            pk_data_types,
                            key_hash_code,
                            pk,
                        )
                        .await?,
                    ))
                }
            }
            AggKind::StringAgg => {
                // TODO, It seems with `order by`, `StringAgg` needs more stuff from `AggCall`
                Err(StreamExecutorError::not_implemented(
                    "It seems with `order by`, `StringAgg` needs more stuff from `AggCall`",
                    None,
                ))
            }
            // TODO: for append-only lists, we can create `ManagedValueState` instead of
            // `ManagedExtremeState`.
            AggKind::Avg | AggKind::Count | AggKind::Sum | AggKind::ApproxCountDistinct => {
                assert!(
                    is_row_count || row_count.is_some(),
                    "should set row_count for value states other than AggKind::RowCount"
                );
                Ok(Self::Value(
                    ManagedValueState::new(agg_call, row_count, pk, state_table).await?,
                ))
            }
            AggKind::RowCount => {
                assert!(is_row_count);
                Ok(Self::Value(
                    ManagedValueState::new(agg_call, row_count, pk, state_table).await?,
                ))
            }
            AggKind::SingleValue => Ok(Self::Value(
                ManagedValueState::new(agg_call, row_count, pk, state_table).await?,
            )),
        }
    }
}
