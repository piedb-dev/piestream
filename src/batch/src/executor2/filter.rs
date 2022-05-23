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
use risingwave_common::array::ArrayImpl::Bool;
use risingwave_common::array::DataChunk;
use risingwave_common::catalog::Schema;
use risingwave_common::error::ErrorCode::InternalError;
use risingwave_common::error::{Result, RwError};
use risingwave_common::util::chunk_coalesce::{DataChunkBuilder, SlicedDataChunk};
use risingwave_expr::expr::{build_from_prost, BoxedExpression};
use risingwave_pb::batch_plan::plan_node::NodeBody;

use crate::executor::ExecutorBuilder;
use crate::executor2::{BoxedDataChunkStream, BoxedExecutor2, BoxedExecutor2Builder, Executor2};

pub struct FilterExecutor2 {
    expr: BoxedExpression,
    child: BoxedExecutor2,
    identity: String,
}

impl Executor2 for FilterExecutor2 {
    fn schema(&self) -> &Schema {
        self.child.schema()
    }

    fn identity(&self) -> &str {
        &self.identity
    }

    fn execute(self: Box<Self>) -> BoxedDataChunkStream {
        self.do_execute()
    }
}

impl FilterExecutor2 {
    #[try_stream(boxed, ok = DataChunk, error = RwError)]
    async fn do_execute(self: Box<Self>) {
        let mut data_chunk_builder =
            DataChunkBuilder::with_default_size(self.child.schema().data_types());

        #[for_await]
        for data_chunk in self.child.execute() {
            let data_chunk = data_chunk?.compact()?;
            let vis_array = self.expr.eval(&data_chunk)?;

            if let Bool(vis) = vis_array.as_ref() {
                let mut sliced_data_chunk =
                    SlicedDataChunk::new_checked(data_chunk.with_visibility(vis.try_into()?))?;

                loop {
                    let (left_data, output) = data_chunk_builder.append_chunk(sliced_data_chunk)?;
                    match (left_data, output) {
                        (Some(left_data), Some(output)) => {
                            sliced_data_chunk = left_data;
                            yield output;
                        }
                        (None, Some(output)) => {
                            yield output;
                            break;
                        }
                        (None, None) => {
                            break;
                        }
                        _ => {
                            return Err(
                                InternalError("Data chunk builder error".to_string()).into()
                            );
                        }
                    }
                }
            } else {
                return Err(InternalError("Filter can only receive bool array".to_string()).into());
            }
        }

        if let Some(chunk) = data_chunk_builder.consume_all()? {
            yield chunk;
        }
    }
}

impl BoxedExecutor2Builder for FilterExecutor2 {
    fn new_boxed_executor2(source: &ExecutorBuilder) -> Result<BoxedExecutor2> {
        ensure!(source.plan_node().get_children().len() == 1);

        let filter_node = try_match_expand!(
            source.plan_node().get_node_body().unwrap(),
            NodeBody::Filter
        )?;

        let expr_node = filter_node.get_search_condition()?;
        let expr = build_from_prost(expr_node)?;
        if let Some(child_plan) = source.plan_node.get_children().get(0) {
            let child = source.clone_for_plan(child_plan).build2()?;
            debug!("Child schema: {:?}", child.schema());

            return Ok(Box::new(Self {
                expr,
                child,
                identity: source.plan_node().get_identity().clone(),
            }));
        }
        Err(InternalError("Filter must have one children".to_string()).into())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use assert_matches::assert_matches;
    use futures::stream::StreamExt;
    use risingwave_common::array::column::Column;
    use risingwave_common::array::{Array, DataChunk, PrimitiveArray};
    use risingwave_common::catalog::{Field, Schema};
    use risingwave_common::error::Result;
    use risingwave_common::types::DataType;
    use risingwave_expr::expr::build_from_prost;
    use risingwave_pb::data::data_type::TypeName;
    use risingwave_pb::expr::expr_node::Type::InputRef;
    use risingwave_pb::expr::expr_node::{RexNode, Type};
    use risingwave_pb::expr::{ExprNode, FunctionCall, InputRefExpr};

    use crate::executor::test_utils::MockExecutor;
    use crate::executor2::{Executor2, FilterExecutor2};

    #[tokio::test]
    async fn test_filter_executor() {
        let col1 = create_column(&[Some(2), Some(2), Some(4), Some(3)]).unwrap();
        let col2 = create_column(&[Some(1), Some(2), Some(1), Some(3)]).unwrap();
        let data_chunk = DataChunk::builder().columns([col1, col2].to_vec()).build();
        let schema = Schema {
            fields: vec![
                Field::unnamed(DataType::Int32),
                Field::unnamed(DataType::Int32),
            ],
        };
        let mut mock_executor = MockExecutor::new(schema);
        mock_executor.add(data_chunk);
        let expr = make_expression(Type::Equal);
        let filter_executor = Box::new(FilterExecutor2 {
            expr: build_from_prost(&expr).unwrap(),
            child: Box::new(mock_executor),
            identity: "FilterExecutor2".to_string(),
        });
        let fields = &filter_executor.schema().fields;
        assert_eq!(fields[0].data_type, DataType::Int32);
        assert_eq!(fields[1].data_type, DataType::Int32);
        let mut stream = filter_executor.execute();
        let res = stream.next().await.unwrap();
        assert_matches!(res, Ok(_));
        if let Ok(res) = res {
            let col1 = res.column_at(0);
            let array = col1.array();
            let col1 = array.as_int32();
            assert_eq!(col1.len(), 2);
            assert_eq!(col1.value_at(0), Some(2));
            assert_eq!(col1.value_at(1), Some(3));
        }
        let res = stream.next().await;
        assert_matches!(res, None);
    }

    fn make_expression(kind: Type) -> ExprNode {
        let lhs = make_inputref(0);
        let rhs = make_inputref(1);
        let function_call = FunctionCall {
            children: vec![lhs, rhs],
        };
        let return_type = risingwave_pb::data::DataType {
            type_name: risingwave_pb::data::data_type::TypeName::Boolean as i32,
            ..Default::default()
        };
        ExprNode {
            expr_type: kind as i32,
            return_type: Some(return_type),
            rex_node: Some(RexNode::FuncCall(function_call)),
        }
    }

    fn make_inputref(idx: i32) -> ExprNode {
        ExprNode {
            expr_type: InputRef as i32,
            return_type: Some(risingwave_pb::data::DataType {
                type_name: TypeName::Int32 as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::InputRef(InputRefExpr { column_idx: idx })),
        }
    }

    fn create_column(vec: &[Option<i32>]) -> Result<Column> {
        let array = PrimitiveArray::from_slice(vec).map(|x| Arc::new(x.into()))?;
        Ok(Column::new(array))
    }
}
