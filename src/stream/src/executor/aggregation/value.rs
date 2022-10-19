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

use itertools::Itertools;
use piestream_common::array::stream_chunk::Ops;
use piestream_common::array::ArrayImpl;
use piestream_common::buffer::Bitmap;
use piestream_common::types::Datum;

use crate::executor::aggregation::agg_impl::{create_streaming_agg_impl, StreamingAggImpl};
use crate::executor::aggregation::AggCall;
use crate::executor::error::StreamExecutorResult;

/// A wrapper around [`StreamingAggImpl`], which maintains aggregation result as a value in memory.
/// Agg executors will get the result and store it in result state table.
pub struct ValueState {
    /// Upstream column indices of agg arguments.
    arg_indices: Vec<usize>,

    /// The internal single-value state.
    inner: Box<dyn StreamingAggImpl>,
}

impl ValueState {
    /// Create an instance from [`AggCall`] and previous output.
    pub fn new(agg_call: &AggCall, prev_output: Option<Datum>) -> StreamExecutorResult<Self> {
        // Create the internal state based on the value we get.
        Ok(Self {
            arg_indices: agg_call.args.val_indices().to_vec(),
            inner: create_streaming_agg_impl(
                agg_call.args.arg_types(),
                &agg_call.kind,
                &agg_call.return_type,
                prev_output,
            )?,
        })
    }

    /// Apply a chunk of data to the state.
    pub fn apply_chunk(
        &mut self,
        ops: Ops<'_>,
        visibility: Option<&Bitmap>,
        columns: &[&ArrayImpl],
    ) -> StreamExecutorResult<()> {
        let data = self
            .arg_indices
            .iter()
            .map(|col_idx| columns[*col_idx])
            .collect_vec();
        self.inner.apply_batch(ops, visibility, &data)
    }

    /// Get the output of the state. Note that in our case, getting the output is very easy, as the
    /// output is the same as the aggregation state. In other aggregators, like min and max,
    /// `get_output` might involve a scan from the state store.
    pub fn get_output(&self) -> Datum {
        self.inner
            .get_output()
            .expect("agg call throw an error in streamAgg")
    }
}

#[cfg(test)]
mod tests {
    use piestream_common::array::{I64Array, Op};
    use piestream_common::types::{DataType, ScalarImpl};

    use super::*;
    use crate::executor::aggregation::AggArgs;

    fn create_test_count_agg() -> AggCall {
        AggCall {
            kind: piestream_expr::expr::AggKind::Count,
            args: AggArgs::Unary(DataType::Int64, 0),
            return_type: DataType::Int64,
            order_pairs: vec![],
            append_only: false,
            filter: None,
        }
    }

    #[tokio::test]
    async fn test_managed_value_state_count() {
        let agg_call = create_test_count_agg();
        let mut state = ValueState::new(&agg_call, None).unwrap();

        // apply a batch and get the output
        state
            .apply_chunk(
                &[Op::Insert, Op::Insert, Op::Insert, Op::Insert],
                None,
                &[&I64Array::from_slice(&[Some(0), Some(1), Some(2), None]).into()],
            )
            .unwrap();

        // get output
        let output = state.get_output();
        assert_eq!(output, Some(ScalarImpl::Int64(3)));

        // check recovery
        let mut state = ValueState::new(&agg_call, Some(output)).unwrap();
        assert_eq!(state.get_output(), Some(ScalarImpl::Int64(3)));
        state
            .apply_chunk(
                &[Op::Insert, Op::Insert, Op::Delete, Op::Insert],
                None,
                &[&I64Array::from_slice(&[Some(42), None, Some(2), Some(8)]).into()],
            )
            .unwrap();
        assert_eq!(state.get_output(), Some(ScalarImpl::Int64(4)));
    }

    fn create_test_max_agg_append_only() -> AggCall {
        AggCall {
            kind: piestream_expr::expr::AggKind::Max,
            args: AggArgs::Unary(DataType::Int64, 0),
            return_type: DataType::Int64,
            order_pairs: vec![],
            append_only: true,
            filter: None,
        }
    }

    #[tokio::test]
    async fn test_managed_value_state_append_only_max() {
        let agg_call = create_test_max_agg_append_only();
        let mut state = ValueState::new(&agg_call, None).unwrap();

        // apply a batch and get the output
        state
            .apply_chunk(
                &[Op::Insert, Op::Insert, Op::Insert, Op::Insert, Op::Insert],
                None,
                &[&I64Array::from_slice(&[Some(-1), Some(0), Some(2), Some(1), None]).into()],
            )
            .unwrap();

        // get output
        assert_eq!(state.get_output(), Some(ScalarImpl::Int64(2)));
    }
}
