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
use piestream_common::types::Datum;
use piestream_storage::table::state_table::StateTable;
use piestream_storage::StateStore;

use crate::executor::aggregation::{create_streaming_agg_state, AggCall, StreamingAggStateImpl};
use crate::executor::error::StreamExecutorResult;

/// A wrapper around [`StreamingAggStateImpl`], which fetches data from the state store and helps
/// update the state. We don't use any trait to wrap around all `ManagedXxxState`, so as to reduce
/// the overhead of creating boxed async future.
pub struct ManagedValueState {
    /// The internal single-value state.
    state: Box<dyn StreamingAggStateImpl>,

    /// Indicates whether this managed state is dirty. If this state is dirty, we cannot evict the
    /// state from memory.
    is_dirty: bool,

    /// Primary key to look up in relational table. For value state, there is only one row.
    /// If None, the pk is empty vector (simple agg). If not None, the pk is group key (hash agg).
    pk: Option<Row>,
}

impl ManagedValueState {
    /// Create a single-value managed state based on `AggCall` and `Keyspace`.
    pub async fn new<S: StateStore>(
        agg_call: AggCall,
        row_count: Option<usize>,
        pk: Option<&Row>,
        state_table: &StateTable<S>,
    ) -> StreamExecutorResult<Self> {
        let data = if row_count != Some(0) {
            // TODO: use the correct epoch
            let epoch = u64::MAX;

            // View the state table as single-value table, and get the value via empty primary key
            // or group key.
            let raw_data = state_table
                .get_row(pk.unwrap_or_else(Row::empty), epoch)
                .await?;

            // According to row layout, the last field of the row is value and we sure the row is
            // not empty.
            raw_data.map(|row| row.0.last().unwrap().clone())
        } else {
            None
        };

        // Create the internal state based on the value we get.
        Ok(Self {
            state: create_streaming_agg_state(
                agg_call.args.arg_types(),
                &agg_call.kind,
                &agg_call.return_type,
                data,
            )?,
            is_dirty: false,
            pk: pk.cloned(),
        })
    }

    /// Apply a batch of data to the state.
    pub async fn apply_batch(
        &mut self,
        ops: Ops<'_>,
        visibility: Option<&Bitmap>,
        data: &[&ArrayImpl],
    ) -> StreamExecutorResult<()> {
        debug_assert!(super::verify_batch(ops, visibility, data));
        self.is_dirty = true;
        self.state.apply_batch(ops, visibility, data)
    }

    /// Get the output of the state. Note that in our case, getting the output is very easy, as the
    /// output is the same as the aggregation state. In other aggregators, like min and max,
    /// `get_output` might involve a scan from the state store.
    pub async fn get_output(&self) -> StreamExecutorResult<Datum> {
        debug_assert!(!self.is_dirty());
        self.state.get_output()
    }

    /// Check if this state needs a flush.
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub async fn flush<S: StateStore>(
        &mut self,
        state_table: &mut StateTable<S>,
    ) -> StreamExecutorResult<()> {
        // If the managed state is not dirty, the caller should not flush. But forcing a flush won't
        // cause incorrect result: it will only produce more I/O.
        debug_assert!(self.is_dirty());

        // Persist value into relational table. The inserted row should concat with pk (pk is in
        // front of value). In this case, the pk is just group key.

        let mut v = vec![];
        v.extend_from_slice(&self.pk.as_ref().unwrap_or_else(Row::empty).0);
        v.push(self.state.get_output()?);

        state_table.insert(Row::new(v))?;

        self.is_dirty = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use piestream_common::array::{I64Array, Op};
    use piestream_common::catalog::{ColumnDesc, ColumnId, TableId};
    use piestream_common::types::{DataType, ScalarImpl};
    use piestream_storage::memory::MemoryStateStore;
    use piestream_storage::table::state_table::StateTable;

    use super::*;
    use crate::executor::aggregation::AggArgs;

    fn create_test_count_state() -> AggCall {
        AggCall {
            kind: piestream_expr::expr::AggKind::Count,
            args: AggArgs::Unary(DataType::Int64, 0),
            return_type: DataType::Int64,
            append_only: false,
        }
    }

    #[tokio::test]
    async fn test_managed_value_state() {
        let mut state_table = StateTable::new(
            MemoryStateStore::new(),
            TableId::from(0x2333),
            vec![ColumnDesc::unnamed(ColumnId::new(0), DataType::Int64)],
            vec![],
            None,
            vec![],
        );
        let mut managed_state =
            ManagedValueState::new(create_test_count_state(), Some(0), None, &state_table)
                .await
                .unwrap();
        assert!(!managed_state.is_dirty());

        // apply a batch and get the output
        managed_state
            .apply_batch(
                &[Op::Insert, Op::Insert, Op::Insert, Op::Insert],
                None,
                &[&I64Array::from_slice(&[Some(0), Some(1), Some(2), None])
                    .unwrap()
                    .into()],
            )
            .await
            .unwrap();
        assert!(managed_state.is_dirty());

        // write to state store
        let epoch: u64 = 0;
        managed_state.flush(&mut state_table).await.unwrap();
        state_table.commit(epoch).await.unwrap();

        // get output
        assert_eq!(
            managed_state.get_output().await.unwrap(),
            Some(ScalarImpl::Int64(3))
        );

        // reload the state and check the output
        let managed_state =
            ManagedValueState::new(create_test_count_state(), None, None, &state_table)
                .await
                .unwrap();
        assert_eq!(
            managed_state.get_output().await.unwrap(),
            Some(ScalarImpl::Int64(3))
        );
    }

    fn create_test_max_agg_append_only() -> AggCall {
        AggCall {
            kind: piestream_expr::expr::AggKind::Max,
            args: AggArgs::Unary(DataType::Int64, 0),
            return_type: DataType::Int64,
            append_only: true,
        }
    }

    #[tokio::test]
    async fn test_managed_value_state_append_only() {
        let pk_index = vec![];
        let mut state_table = StateTable::new(
            MemoryStateStore::new(),
            TableId::from(0x2333),
            vec![ColumnDesc::unnamed(ColumnId::new(0), DataType::Int64)],
            vec![],
            None,
            pk_index,
        );
        let mut managed_state = ManagedValueState::new(
            create_test_max_agg_append_only(),
            Some(0),
            None,
            &state_table,
        )
        .await
        .unwrap();
        assert!(!managed_state.is_dirty());

        // apply a batch and get the output
        managed_state
            .apply_batch(
                &[Op::Insert, Op::Insert, Op::Insert, Op::Insert, Op::Insert],
                None,
                &[
                    &I64Array::from_slice(&[Some(-1), Some(0), Some(2), Some(1), None])
                        .unwrap()
                        .into(),
                ],
            )
            .await
            .unwrap();
        assert!(managed_state.is_dirty());

        // write to state store
        let epoch: u64 = 0;
        managed_state.flush(&mut state_table).await.unwrap();
        state_table.commit(epoch).await.unwrap();

        // get output
        assert_eq!(
            managed_state.get_output().await.unwrap(),
            Some(ScalarImpl::Int64(2))
        );

        // reload the state and check the output
        let managed_state =
            ManagedValueState::new(create_test_max_agg_append_only(), None, None, &state_table)
                .await
                .unwrap();
        assert_eq!(
            managed_state.get_output().await.unwrap(),
            Some(ScalarImpl::Int64(2))
        );
    }
}
