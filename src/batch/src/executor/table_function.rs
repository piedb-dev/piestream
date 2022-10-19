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

use futures_async_stream::try_stream;
use piestream_common::array::DataChunk;
use piestream_common::catalog::{Field, Schema};
use piestream_common::error::{Result, RwError};
use piestream_expr::table_function::{build_from_prost, BoxedTableFunction};
use piestream_pb::batch_plan::plan_node::NodeBody;

use super::{BoxedExecutor, BoxedExecutorBuilder};
use crate::executor::{BoxedDataChunkStream, Executor, ExecutorBuilder};
use crate::task::BatchTaskContext;

pub struct TableFunctionExecutor {
    schema: Schema,
    identity: String,
    table_function: BoxedTableFunction,
    chunk_size: usize,
}

impl Executor for TableFunctionExecutor {
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

impl TableFunctionExecutor {
    #[try_stream(boxed, ok = DataChunk, error = RwError)]
    async fn do_execute(self: Box<Self>) {
        let dummy_chunk = DataChunk::new_dummy(1);

        let mut builder = self
            .table_function
            .return_type()
            .create_array_builder(self.chunk_size);
        let mut len = 0;
        for array in self.table_function.eval(&dummy_chunk)? {
            len += array.len();
            builder.append_array(&array);
        }
        let ret = DataChunk::new(vec![builder.finish().into()], len);
        yield ret
    }
}

pub struct TableFunctionExecutorBuilder {}

impl TableFunctionExecutorBuilder {}

#[async_trait::async_trait]
impl BoxedExecutorBuilder for TableFunctionExecutorBuilder {
    async fn new_boxed_executor<C: BatchTaskContext>(
        source: &ExecutorBuilder<'_, C>,
        inputs: Vec<BoxedExecutor>,
    ) -> Result<BoxedExecutor> {
        ensure!(
            inputs.is_empty(),
            "GenerateSeriesExecutor should not have child!"
        );
        let node = try_match_expand!(
            source.plan_node().get_node_body().unwrap(),
            NodeBody::TableFunction
        )?;

        let identity = source.plan_node().get_identity().clone();

        let chunk_size = source.context.get_config().developer.batch_chunk_size;

        let table_function = build_from_prost(node.table_function.as_ref().unwrap(), chunk_size)?;

        let fields = vec![Field::unnamed(table_function.return_type())];

        Ok(Box::new(TableFunctionExecutor {
            schema: Schema { fields },
            identity,
            table_function,
            chunk_size,
        }))
    }
}
