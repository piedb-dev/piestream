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

use std::collections::VecDeque;

use assert_matches::assert_matches;
use futures_async_stream::{for_await, try_stream};
use itertools::Itertools;
use risingwave_common::array::DataChunk;
use risingwave_common::catalog::Schema;
use risingwave_common::error::RwError;

use crate::executor2::{BoxedDataChunkStream, BoxedExecutor2, Executor2};

/// Mock the input of executor.
/// You can bind one or more `MockExecutor` as the children of the executor to test,
/// (`HashAgg`, e.g), so that allow testing without instantiating real `SeqScan`s and real storage.
pub struct MockExecutor {
    chunks: VecDeque<DataChunk>,
    schema: Schema,
    identity: String,
}

impl MockExecutor {
    pub fn new(schema: Schema) -> Self {
        Self {
            chunks: VecDeque::new(),
            schema,
            identity: "MockExecutor".to_string(),
        }
    }

    pub fn with_chunk(chunk: DataChunk, schema: Schema) -> Self {
        let mut ret = Self::new(schema);
        ret.add(chunk);
        ret
    }

    pub fn add(&mut self, chunk: DataChunk) {
        self.chunks.push_back(chunk);
    }
}

impl Executor2 for MockExecutor {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn identity(&self) -> &str {
        &self.identity
    }

    fn execute(self: Box<Self>) -> BoxedDataChunkStream {
        self.do_execute()
    }
}

impl MockExecutor {
    #[try_stream(boxed, ok = DataChunk, error = RwError)]
    async fn do_execute(self: Box<Self>) {
        for data_chunk in self.chunks {
            yield data_chunk;
        }
    }
}

/// if the input from two child executor is same(considering order),
/// it will also check the columns structure of chunks from child executor
/// use for executor unit test.
///
/// if want diff ignoring order, add a `order_by` executor in manual currently, when the `schema`
/// method of `executor` is ready, an order-ignored version will be added.
pub async fn diff_executor_output(actual: BoxedExecutor2, expect: BoxedExecutor2) {
    let mut expect_cardinality = 0;
    let mut actual_cardinality = 0;
    let mut expects = vec![];
    let mut actuals = vec![];

    #[for_await]
    for chunk in expect.execute() {
        assert_matches!(chunk, Ok(_));
        let chunk = chunk.unwrap().compact().unwrap();
        expect_cardinality += chunk.cardinality();
        expects.push(chunk);
    }

    #[for_await]
    for chunk in actual.execute() {
        assert_matches!(chunk, Ok(_));
        let chunk = chunk.unwrap().compact().unwrap();
        actual_cardinality += chunk.cardinality();
        actuals.push(chunk);
    }

    assert_eq!(actual_cardinality, expect_cardinality);
    if actual_cardinality == 0 {
        return;
    }
    let expect = DataChunk::rechunk(expects.as_slice(), actual_cardinality)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let actual = DataChunk::rechunk(actuals.as_slice(), actual_cardinality)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let col_num = expect.columns().len();
    assert_eq!(col_num, actual.columns().len());
    expect
        .columns()
        .iter()
        .zip_eq(actual.columns().iter())
        .for_each(|(c1, c2)| assert_eq!(c1.array().to_protobuf(), c2.array().to_protobuf()));

    is_data_chunk_eq(&expect, &actual)
}

fn is_data_chunk_eq(left: &DataChunk, right: &DataChunk) {
    assert!(left.visibility().is_none());
    assert!(right.visibility().is_none());

    assert_eq!(
        left.cardinality(),
        right.cardinality(),
        "two chunks cardinality is different"
    );

    left.rows()
        .zip_eq(right.rows())
        .for_each(|(row1, row2)| assert_eq!(row1, row2));
}
