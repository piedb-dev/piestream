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

use futures_async_stream::try_stream;
use risingwave_common::array::column::Column;
use risingwave_common::array::DataChunk;
use risingwave_common::catalog::{Field, Schema};
use risingwave_common::error::{ErrorCode, Result, RwError};
use risingwave_expr::expr::{build_from_prost, BoxedExpression};
use risingwave_pb::batch_plan::plan_node::NodeBody;

use crate::executor::ExecutorBuilder;
use crate::executor2::{BoxedDataChunkStream, BoxedExecutor2, BoxedExecutor2Builder, Executor2};

pub struct ProjectExecutor2 {
    expr: Vec<BoxedExpression>,
    child: BoxedExecutor2,
    schema: Schema,
    identity: String,
}

impl Executor2 for ProjectExecutor2 {
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

impl ProjectExecutor2 {
    #[try_stream(boxed, ok = DataChunk, error = RwError)]
    async fn do_execute(mut self: Box<Self>) {
        #[for_await]
        for data_chunk in self.child.execute() {
            let data_chunk = data_chunk?;
            // let data_chunk = data_chunk.compact()?;
            let arrays: Vec<Column> = self
                .expr
                .iter_mut()
                .map(|expr| expr.eval(&data_chunk).map(Column::new))
                .collect::<Result<Vec<_>>>()?;
            let ret = if arrays.is_empty() {
                DataChunk::new_dummy(data_chunk.cardinality())
            } else {
                DataChunk::builder().columns(arrays).build()
            };
            yield ret
        }
    }
}

impl BoxedExecutor2Builder for ProjectExecutor2 {
    fn new_boxed_executor2(source: &ExecutorBuilder) -> Result<BoxedExecutor2> {
        ensure!(source.plan_node().get_children().len() == 1);

        let project_node = try_match_expand!(
            source.plan_node().get_node_body().unwrap(),
            NodeBody::Project
        )?;

        let proto_child = source.plan_node.get_children().get(0).ok_or_else(|| {
            RwError::from(ErrorCode::InternalError(String::from(
                "Child interpreting error",
            )))
        })?;
        let child_node = source.clone_for_plan(proto_child).build2()?;

        let project_exprs = project_node
            .get_select_list()
            .iter()
            .map(build_from_prost)
            .collect::<Result<Vec<BoxedExpression>>>()?;

        let fields = project_exprs
            .iter()
            .map(|expr| Field::unnamed(expr.return_type()))
            .collect::<Vec<Field>>();

        Ok(Box::new(Self {
            expr: project_exprs,
            child: child_node,
            schema: Schema { fields },
            identity: source.plan_node().get_identity().clone(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use futures::stream::StreamExt;
    use risingwave_common::array::{Array, I32Array};
    use risingwave_common::catalog::{Field, Schema};
    use risingwave_common::test_prelude::*;
    use risingwave_common::types::DataType;
    use risingwave_expr::expr::{InputRefExpression, LiteralExpression};

    use super::*;
    use crate::executor::test_utils::MockExecutor;
    use crate::executor2::{Executor2, ValuesExecutor2};
    use crate::*;

    #[tokio::test]
    async fn test_project_executor() -> Result<()> {
        let chunk = DataChunk::from_pretty(
            "
            i     i
            1     7
            2     8
            33333 66666
            4     4
            5     3
        ",
        );

        let expr1 = InputRefExpression::new(DataType::Int32, 0);
        let expr_vec = vec![Box::new(expr1) as BoxedExpression];

        let schema = schema_unnamed! { DataType::Int32, DataType::Int32 };
        let mut mock_executor = MockExecutor::new(schema);
        mock_executor.add(chunk);

        let fields = expr_vec
            .iter()
            .map(|expr| Field::unnamed(expr.return_type()))
            .collect::<Vec<Field>>();

        let proj_executor = Box::new(ProjectExecutor2 {
            expr: expr_vec,
            child: Box::new(mock_executor),
            schema: Schema { fields },
            identity: "ProjectExecutor".to_string(),
        });

        let fields = &proj_executor.schema().fields;
        assert_eq!(fields[0].data_type, DataType::Int32);

        let mut stream = proj_executor.execute();
        let result_chunk = stream.next().await.unwrap().unwrap();
        assert_eq!(result_chunk.dimension(), 1);
        assert_eq!(
            result_chunk
                .column_at(0)
                .array()
                .as_int32()
                .iter()
                .collect::<Vec<_>>(),
            vec![Some(1), Some(2), Some(33333), Some(4), Some(5)]
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_project_dummy_chunk() {
        let literal = LiteralExpression::new(DataType::Int32, Some(1_i32.into()));

        let values_executor2: Box<dyn Executor2> = Box::new(ValuesExecutor2::new(
            vec![vec![]], // One single row with no column.
            Schema::default(),
            "ValuesExecutor".to_string(),
            1024,
        ));

        let proj_executor = Box::new(ProjectExecutor2 {
            expr: vec![Box::new(literal)],
            child: values_executor2,
            schema: schema_unnamed!(DataType::Int32),
            identity: "ProjectExecutor2".to_string(),
        });
        let mut stream = proj_executor.execute();
        let chunk = stream.next().await.unwrap().unwrap();
        assert_eq!(
            *chunk.column_at(0).array(),
            array_nonnull!(I32Array, [1]).into()
        );
    }
}
