// Copyright 2022 Piedb Data
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

use anyhow::anyhow;
use futures_async_stream::try_stream;
use piestream_common::array::ArrayImpl::Bool;
use piestream_common::array::{Array, DataChunk};
use piestream_common::catalog::Schema;
use piestream_common::error::{Result, RwError};
use piestream_common::util::chunk_coalesce::DataChunkBuilder;
use piestream_expr::expr::{build_from_prost, BoxedExpression};
use piestream_pb::batch_plan::plan_node::NodeBody;

use crate::error::BatchError;
use crate::executor::{
    BoxedDataChunkStream, BoxedExecutor, BoxedExecutorBuilder, Executor, ExecutorBuilder,
};
use crate::task::BatchTaskContext;

pub struct FilterExecutor {
    expr: BoxedExpression,
    child: BoxedExecutor,
    identity: String,
}

impl Executor for FilterExecutor {
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

impl FilterExecutor {
    #[try_stream(boxed, ok = DataChunk, error = RwError)]
    async fn do_execute(self: Box<Self>) {
        let mut data_chunk_builder =
            DataChunkBuilder::with_default_size(self.child.schema().data_types());

        #[for_await]
        for data_chunk in self.child.execute() {
            let data_chunk = data_chunk?.compact();
            let vis_array = self.expr.eval(&data_chunk)?;

            if let Bool(vis) = vis_array.as_ref() {
                #[for_await]
                for data_chunk in data_chunk_builder
                    .trunc_data_chunk(data_chunk.with_visibility(vis.iter().collect()))
                {
                    yield data_chunk;
                }
            } else {
                return Err(
                    BatchError::Internal(anyhow!("Filter can only receive bool array")).into(),
                );
            }
        }

        if let Some(chunk) = data_chunk_builder.consume_all() {
            yield chunk;
        }
    }
}

#[async_trait::async_trait]
impl BoxedExecutorBuilder for FilterExecutor {
    async fn new_boxed_executor<C: BatchTaskContext>(
        source: &ExecutorBuilder<'_, C>,
        inputs: Vec<BoxedExecutor>,
    ) -> Result<BoxedExecutor> {
        let [input]: [_; 1] = inputs.try_into().unwrap();

        let filter_node = try_match_expand!(
            source.plan_node().get_node_body().unwrap(),
            NodeBody::Filter
        )?;

        let expr_node = filter_node.get_search_condition()?;
        let expr = build_from_prost(expr_node)?;
        Ok(Box::new(Self::new(
            expr,
            input,
            source.plan_node().get_identity().clone(),
        )))
    }
}

impl FilterExecutor {
    pub fn new(expr: BoxedExpression, input: BoxedExecutor, identity: String) -> Self {
        Self {
            expr,
            child: input,
            identity,
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use futures::stream::StreamExt;
    use piestream_common::array::{Array, DataChunk, ListValue};
    use piestream_common::catalog::{Field, Schema};
    use piestream_common::test_prelude::DataChunkTestExt;
    use piestream_common::types::{DataType, Scalar};
    use piestream_expr::expr::build_from_prost;
    use piestream_pb::data::data_type::TypeName;
    use piestream_pb::expr::expr_node::Type::InputRef;
    use piestream_pb::expr::expr_node::{RexNode, Type};
    use piestream_pb::expr::{ConstantValue, ExprNode, FunctionCall, InputRefExpr};

    use crate::executor::test_utils::MockExecutor;
    use crate::executor::{Executor, FilterExecutor};

    #[tokio::test]
    async fn test_list_filter_executor() {
        use piestream_common::array::{
            ArrayBuilder, ArrayMeta, ListArrayBuilder, ListRef, ListValue,
        };
        use piestream_common::types::Scalar;

        let mut builder = ListArrayBuilder::with_meta(
            4,
            ArrayMeta::List {
                datatype: Box::new(DataType::Int32),
            },
        );

        // Add 4 ListValues to ArrayBuilder
        (1..=4).for_each(|i| {
            builder.append(Some(ListRef::ValueRef {
                val: &ListValue::new(vec![Some(i.to_scalar_value())]),
            }));
        });

        // Use builder to obtain a single (List) column DataChunk
        let chunk = DataChunk::new(vec![builder.finish().into()], 4);

        // Initialize mock executor
        let mut mock_executor = MockExecutor::new(Schema {
            fields: vec![Field::unnamed(DataType::List {
                datatype: Box::new(DataType::Int32),
            })],
        });
        mock_executor.add(chunk);

        // Initialize filter executor
        let expr = make_filter_expression(Type::GreaterThan);
        let filter_executor = Box::new(FilterExecutor {
            expr: build_from_prost(&expr).unwrap(),
            child: Box::new(mock_executor),
            identity: "FilterExecutor".to_string(),
        });

        let fields = &filter_executor.schema().fields;

        assert!(fields.iter().all(|f| f.data_type
            == DataType::List {
                datatype: Box::new(DataType::Int32)
            }));

        let mut stream = filter_executor.execute();

        let res = stream.next().await.unwrap();

        assert_matches!(res, Ok(_));
        if let Ok(res) = res {
            let col1 = res.column_at(0);
            let array = col1.array();
            let col1 = array.as_list();
            assert_eq!(col1.len(), 2);
            // Assert that values 3 and 4 are bigger than 2
            assert_eq!(
                col1.value_at(0),
                Some(ListRef::ValueRef {
                    val: &ListValue::new(vec![Some(3.to_scalar_value())]),
                })
            );
            assert_eq!(
                col1.value_at(1),
                Some(ListRef::ValueRef {
                    val: &ListValue::new(vec![Some(4.to_scalar_value())]),
                })
            );
        }
        let res = stream.next().await;
        assert_matches!(res, None);
    }

    fn make_filter_expression(kind: Type) -> ExprNode {
        use piestream_common::types::ScalarImpl;
        let lhs = ExprNode {
            expr_type: InputRef as i32,
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::List as i32,
                field_type: vec![piestream_pb::data::DataType {
                    type_name: TypeName::Int32 as i32,
                    is_nullable: true,
                    ..Default::default()
                }],
                ..Default::default()
            }),
            rex_node: Some(RexNode::InputRef(InputRefExpr { column_idx: 0 })),
        };
        let rhs = ExprNode {
            expr_type: Type::ConstantValue as i32,
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::List as i32,
                field_type: vec![piestream_pb::data::DataType {
                    type_name: TypeName::Int32 as i32,
                    ..Default::default()
                }],
                ..Default::default()
            }),
            rex_node: Some(RexNode::Constant(ConstantValue {
                body: ScalarImpl::List(ListValue::new(vec![Some(2.to_scalar_value())]))
                    .to_protobuf(),
            })),
        };
        let function_call = FunctionCall {
            children: vec![lhs, rhs],
        };
        let return_type = piestream_pb::data::DataType {
            type_name: piestream_pb::data::data_type::TypeName::Boolean as i32,
            ..Default::default()
        };
        ExprNode {
            expr_type: kind as i32,
            return_type: Some(return_type),
            rex_node: Some(RexNode::FuncCall(function_call)),
        }
    }

    #[tokio::test]
    async fn test_filter_executor() {
        let schema = Schema {
            fields: vec![
                Field::unnamed(DataType::Int32),
                Field::unnamed(DataType::Int32),
            ],
        };
        let mut mock_executor = MockExecutor::new(schema);
        mock_executor.add(DataChunk::from_pretty(
            "i i
             2 1
             2 2
             4 1
             3 3",
        ));
        let expr = make_expression(Type::Equal);
        let filter_executor = Box::new(FilterExecutor {
            expr: build_from_prost(&expr).unwrap(),
            child: Box::new(mock_executor),
            identity: "FilterExecutor".to_string(),
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
        let return_type = piestream_pb::data::DataType {
            type_name: piestream_pb::data::data_type::TypeName::Boolean as i32,
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
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::Int32 as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::InputRef(InputRefExpr { column_idx: idx })),
        }
    }
}
