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

use piestream_common::array::stream_chunk::Ops;
use piestream_common::array::{ArrayImpl, Row};
use piestream_common::buffer::Bitmap;
use piestream_common::catalog::Schema;
use piestream_common::types::Datum;
use piestream_expr::expr::AggKind;
use piestream_storage::table::streaming_table::state_table::StateTable;
use piestream_storage::StateStore;

use super::table_state::{
    GenericExtremeState, ManagedArrayAggState, ManagedStringAggState, ManagedTableState,
};
use super::AggCall;
use crate::common::StateTableColumnMapping;
use crate::executor::{PkIndices, StreamExecutorResult};

/// Aggregation state as a materialization of input chunks.
///
/// For example, in `string_agg`, several useful columns are picked from input chunks and
/// stored in the state table when applying chunks, and the aggregation result is calculated
/// when need to get output.
pub struct MaterializedInputState<S: StateStore> {
    inner: Box<dyn ManagedTableState<S>>,
}

impl<S: StateStore> MaterializedInputState<S> {
    /// Create an instance from [`AggCall`].
    pub fn new(
        agg_call: &AggCall,
        group_key: Option<&Row>,
        pk_indices: &PkIndices,
        col_mapping: StateTableColumnMapping,
        row_count: usize,
        extreme_cache_size: usize,
        input_schema: &Schema,
    ) -> Self {
        Self {
            inner: match agg_call.kind {
                AggKind::Max | AggKind::Min | AggKind::FirstValue => {
                    Box::new(GenericExtremeState::new(
                        agg_call,
                        group_key,
                        pk_indices,
                        col_mapping,
                        row_count,
                        extreme_cache_size,
                        input_schema,
                    ))
                }
                AggKind::StringAgg => Box::new(ManagedStringAggState::new(
                    agg_call,
                    group_key,
                    pk_indices,
                    col_mapping,
                    row_count,
                )),
                AggKind::ArrayAgg => Box::new(ManagedArrayAggState::new(
                    agg_call,
                    group_key,
                    pk_indices,
                    col_mapping,
                    row_count,
                )),
                _ => panic!(
                    "Agg kind `{}` is not expected to have materialized input state",
                    agg_call.kind
                ),
            },
        }
    }

    /// Apply a chunk of data to the state.
    pub async fn apply_chunk(
        &mut self,
        ops: Ops<'_>,
        visibility: Option<&Bitmap>,
        columns: &[&ArrayImpl],
        state_table: &mut StateTable<S>,
    ) -> StreamExecutorResult<()> {
        self.inner
            .apply_chunk(ops, visibility, columns, state_table)
            .await
    }

    /// Get the output of the state.
    pub async fn get_output(&mut self, state_table: &StateTable<S>) -> StreamExecutorResult<Datum> {
        self.inner.get_output(state_table).await
    }
}
