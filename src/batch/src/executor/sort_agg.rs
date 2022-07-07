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

use futures_async_stream::try_stream;
use itertools::Itertools;
use piestream_common::array::column::Column;
use piestream_common::array::{ArrayBuilderImpl, ArrayRef, DataChunk};
use piestream_common::catalog::{Field, Schema};
use piestream_common::error::{Result, RwError};
use piestream_common::util::chunk_coalesce::DEFAULT_CHUNK_BUFFER_SIZE;
use piestream_expr::expr::{build_from_prost, BoxedExpression};
use piestream_expr::vector_op::agg::{
    create_sorted_grouper, AggStateFactory, BoxedAggState, BoxedSortedGrouper, EqGroups,
};
use piestream_pb::batch_plan::plan_node::NodeBody;

use crate::executor::{
    BoxedDataChunkStream, BoxedExecutor, BoxedExecutorBuilder, Executor, ExecutorBuilder,
};
use crate::task::BatchTaskContext;

/// `SortAggExecutor` implements the sort aggregate algorithm, which assumes
/// that the input chunks has already been sorted by group columns.
/// The aggregation will be applied to tuples within the same group.
/// And the output schema is `[group columns, agg result]`.
///
/// As a special case, simple aggregate without groups satisfies the requirement
/// automatically because all tuples should be aggregated together.
pub struct SortAggExecutor {
    agg_states: Vec<BoxedAggState>,
    group_keys: Vec<BoxedExpression>,
    sorted_groupers: Vec<BoxedSortedGrouper>,
    child: BoxedExecutor,
    schema: Schema,
    identity: String,
    output_size_limit: usize, // make unit test easy
}

#[async_trait::async_trait]
impl BoxedExecutorBuilder for SortAggExecutor {
    async fn new_boxed_executor<C: BatchTaskContext>(
        source: &ExecutorBuilder<C>,
        mut inputs: Vec<BoxedExecutor>,
    ) -> Result<BoxedExecutor> {
        ensure!(
            inputs.len() == 1,
            "Sort aggregation executor should have only 1 child!"
        );
        let sort_agg_node = try_match_expand!(
            source.plan_node().get_node_body().unwrap(),
            NodeBody::SortAgg
        )?;

        let agg_states: Vec<_> = sort_agg_node
            .get_agg_calls()
            .iter()
            .map(|x| AggStateFactory::new(x)?.create_agg_state())
            .try_collect()?;

        let group_keys: Vec<_> = sort_agg_node
            .get_group_keys()
            .iter()
            .map(build_from_prost)
            .try_collect()?;

        let sorted_groupers: Vec<_> = group_keys
            .iter()
            .map(|e| create_sorted_grouper(e.return_type()))
            .try_collect()?;

        let fields = group_keys
            .iter()
            .map(|e| e.return_type())
            .chain(agg_states.iter().map(|e| e.return_type()))
            .map(Field::unnamed)
            .collect::<Vec<Field>>();

        Ok(Box::new(Self {
            agg_states,
            group_keys,
            sorted_groupers,
            child: inputs.remove(0),
            schema: Schema { fields },
            identity: source.plan_node().get_identity().clone(),
            output_size_limit: DEFAULT_CHUNK_BUFFER_SIZE,
        }))
    }
}

impl Executor for SortAggExecutor {
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

impl SortAggExecutor {
    #[try_stream(boxed, ok = DataChunk, error = RwError)]
    async fn do_execute(mut self: Box<Self>) {
        let mut left_capacity = self.output_size_limit;
        let (mut group_builders, mut agg_builders) =
            SortAggExecutor::create_builders(&self.group_keys, &self.agg_states);

        #[for_await]
        for child_chunk in self.child.execute() {
            let child_chunk = child_chunk?.compact()?;
            let group_columns: Vec<_> = self
                .group_keys
                .iter_mut()
                .map(|expr| expr.eval(&child_chunk))
                .try_collect()?;

            let groups = self
                .sorted_groupers
                .iter()
                .zip_eq(&group_columns)
                .map(|(grouper, array)| grouper.detect_groups(array))
                .collect::<Result<Vec<EqGroups>>>()?;

            let mut groups = EqGroups::intersect(&groups);
            loop {
                let limit = {
                    if left_capacity >= groups.len() {
                        left_capacity -= groups.len();
                        0
                    } else {
                        let old = left_capacity;
                        left_capacity = 0;
                        old
                    }
                };
                groups.set_limit(limit);

                SortAggExecutor::build_sorted_groups(
                    &mut self.sorted_groupers,
                    &group_columns,
                    &mut group_builders,
                    &groups,
                )?;

                SortAggExecutor::build_agg_states(
                    &mut self.agg_states,
                    &child_chunk,
                    &mut agg_builders,
                    &groups,
                )?;

                if left_capacity == 0 {
                    // yield output chunk
                    let columns = group_builders
                        .into_iter()
                        .chain(agg_builders)
                        .map(|b| Ok(Column::new(Arc::new(b.finish()?))))
                        .collect::<Result<Vec<_>>>()?;

                    let output = DataChunk::new(columns, self.output_size_limit);
                    yield output;

                    // reset builders and capactiy to build next output chunk
                    (group_builders, agg_builders) =
                        SortAggExecutor::create_builders(&self.group_keys, &self.agg_states);

                    left_capacity = self.output_size_limit;
                }

                groups.advance_offset();
                if groups.is_empty() {
                    break;
                }
            }
        }

        assert!(left_capacity > 0);
        // process the last group
        self.sorted_groupers
            .iter()
            .zip_eq(&mut group_builders)
            .try_for_each(|(grouper, builder)| grouper.output(builder))?;
        self.agg_states
            .iter()
            .zip_eq(&mut agg_builders)
            .try_for_each(|(state, builder)| state.output(builder))?;

        let columns = group_builders
            .into_iter()
            .chain(agg_builders)
            .map(|b| Ok(Column::new(Arc::new(b.finish()?))))
            .collect::<Result<Vec<_>>>()?;

        let output = DataChunk::new(columns, self.output_size_limit - left_capacity + 1);

        yield output;
    }

    fn build_sorted_groups(
        sorted_groupers: &mut [BoxedSortedGrouper],
        group_columns: &[ArrayRef],
        group_builders: &mut [ArrayBuilderImpl],
        groups: &EqGroups,
    ) -> Result<()> {
        sorted_groupers
            .iter_mut()
            .zip_eq(group_columns)
            .zip_eq(group_builders)
            .try_for_each(|((grouper, column), builder)| {
                grouper.update_and_output_with_sorted_groups(column, builder, groups)
            })
    }

    fn build_agg_states(
        agg_states: &mut [BoxedAggState],
        child_chunk: &DataChunk,
        agg_builders: &mut [ArrayBuilderImpl],
        groups: &EqGroups,
    ) -> Result<()> {
        agg_states
            .iter_mut()
            .zip_eq(agg_builders)
            .try_for_each(|(state, builder)| {
                state.update_and_output_with_sorted_groups(child_chunk, builder, groups)
            })
    }

    fn create_builders(
        group_keys: &[BoxedExpression],
        agg_states: &[BoxedAggState],
    ) -> (Vec<ArrayBuilderImpl>, Vec<ArrayBuilderImpl>) {
        let group_builders = group_keys
            .iter()
            .map(|e| e.return_type().create_array_builder(1))
            .collect();

        let agg_builders = agg_states
            .iter()
            .map(|e| e.return_type().create_array_builder(1))
            .collect();

        (group_builders, agg_builders)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use futures::StreamExt;
    use piestream_common::array::{Array as _, I64Array};
    use piestream_common::catalog::{Field, Schema};
    use piestream_common::test_prelude::DataChunkTestExt;
    use piestream_common::types::DataType;
    use piestream_expr::expr::build_from_prost;
    use piestream_pb::data::data_type::TypeName;
    use piestream_pb::data::DataType as ProstDataType;
    use piestream_pb::expr::agg_call::{Arg, Type};
    use piestream_pb::expr::expr_node::RexNode;
    use piestream_pb::expr::expr_node::Type::InputRef;
    use piestream_pb::expr::{AggCall, ExprNode, InputRefExpr};

    use super::*;
    use crate::executor::test_utils::MockExecutor;

    #[tokio::test]
    async fn execute_count_star_int32() -> Result<()> {
        // mock a child executor
        let schema = Schema {
            fields: vec![
                Field::unnamed(DataType::Int32),
                Field::unnamed(DataType::Int32),
                Field::unnamed(DataType::Int32),
            ],
        };
        let mut child = MockExecutor::new(schema);
        child.add(DataChunk::from_pretty(
            "i i i
             1 1 7
             2 1 8
             3 3 8
             4 3 9",
        ));
        child.add(DataChunk::from_pretty(
            "i i i
             1 3 9
             2 4 9
             3 4 9
             4 5 9",
        ));
        child.add(DataChunk::from_pretty(
            "i i i
             1 5 9
             2 5 9
             3 5 9
             4 5 9",
        ));

        let prost = AggCall {
            r#type: Type::Count as i32,
            args: vec![],
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            distinct: false,
        };

        let count_star = AggStateFactory::new(&prost)?.create_agg_state()?;
        let group_exprs: Vec<BoxedExpression> = vec![];
        let sorted_groupers = vec![];
        let agg_states = vec![count_star];

        // chain group key fields and agg state schema to get output schema for sort agg
        let fields = group_exprs
            .iter()
            .map(|e| e.return_type())
            .chain(agg_states.iter().map(|e| e.return_type()))
            .map(Field::unnamed)
            .collect::<Vec<Field>>();

        let executor = Box::new(SortAggExecutor {
            agg_states,
            group_keys: group_exprs,
            sorted_groupers,
            child: Box::new(child),
            schema: Schema { fields },
            identity: "SortAggExecutor".to_string(),
            output_size_limit: 3,
        });

        let fields = &executor.schema().fields;
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].data_type, DataType::Int64);

        let mut stream = executor.execute();
        let res = stream.next().await.unwrap();
        assert_matches!(res, Ok(_));
        assert_matches!(stream.next().await, None);

        let chunk = res?;
        assert_eq!(chunk.cardinality(), 1);
        let actual = chunk.column_at(0).array();
        let actual_agg: &I64Array = actual.as_ref().into();
        let v = actual_agg.iter().collect::<Vec<Option<i64>>>();

        // check the result
        assert_eq!(v, vec![Some(12)]);
        Ok(())
    }

    #[tokio::test]
    async fn execute_count_star_int32_grouped() -> Result<()> {
        // mock a child executor
        let schema = Schema {
            fields: vec![
                Field::unnamed(DataType::Int32),
                Field::unnamed(DataType::Int32),
                Field::unnamed(DataType::Int32),
            ],
        };
        let mut child = MockExecutor::new(schema);
        child.add(DataChunk::from_pretty(
            "i i i
             1 1 7
             2 1 8
             3 3 8
             4 3 9
             5 4 9",
        ));
        child.add(DataChunk::from_pretty(
            "i i i
             1 4 9
             2 4 9
             3 4 9
             4 5 9
             5 6 9
             6 7 9
             7 7 9
             8 8 9",
        ));
        child.add(DataChunk::from_pretty(
            "i i i
             1 8 9
             2 8 9
             3 8 9
             4 8 9
             5 8 9",
        ));

        let prost = AggCall {
            r#type: Type::Count as i32,
            args: vec![],
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            distinct: false,
        };

        let count_star = AggStateFactory::new(&prost)?.create_agg_state()?;
        let group_exprs: Vec<_> = (1..=2)
            .map(|idx| {
                build_from_prost(&ExprNode {
                    expr_type: InputRef as i32,
                    return_type: Some(ProstDataType {
                        type_name: TypeName::Int32 as i32,
                        ..Default::default()
                    }),
                    rex_node: Some(RexNode::InputRef(InputRefExpr { column_idx: idx })),
                })
            })
            .try_collect()?;

        let sorted_groupers = group_exprs
            .iter()
            .map(|e| create_sorted_grouper(e.return_type()))
            .collect::<Result<Vec<BoxedSortedGrouper>>>()?;

        let agg_states = vec![count_star];

        // chain group key fields and agg state schema to get output schema for sort agg
        let fields = group_exprs
            .iter()
            .map(|e| e.return_type())
            .chain(agg_states.iter().map(|e| e.return_type()))
            .map(Field::unnamed)
            .collect::<Vec<Field>>();

        let executor = Box::new(SortAggExecutor {
            agg_states,
            group_keys: group_exprs,
            sorted_groupers,
            child: Box::new(child),
            schema: Schema { fields },
            identity: "SortAggExecutor".to_string(),
            output_size_limit: 3,
        });

        let fields = &executor.schema().fields;
        assert_eq!(fields[0].data_type, DataType::Int32);
        assert_eq!(fields[1].data_type, DataType::Int32);
        assert_eq!(fields[2].data_type, DataType::Int64);

        let mut stream = executor.execute();
        let res = stream.next().await.unwrap();
        assert_matches!(res, Ok(_));

        let chunk = res?;
        assert_eq!(chunk.cardinality(), 3);
        let actual = chunk.column_at(2).array();
        let actual_agg: &I64Array = actual.as_ref().into();
        let v = actual_agg.iter().collect::<Vec<Option<i64>>>();

        // check the result
        assert_eq!(v, vec![Some(1), Some(1), Some(1)]);
        check_group_key_column(&chunk, 0, vec![Some(1), Some(1), Some(3)]);
        check_group_key_column(&chunk, 1, vec![Some(7), Some(8), Some(8)]);

        let res = stream.next().await.unwrap();
        assert_matches!(res, Ok(_));

        let chunk = res?;
        assert_eq!(chunk.cardinality(), 3);
        let actual = chunk.column_at(2).array();
        let actual_agg: &I64Array = actual.as_ref().into();
        let v = actual_agg.iter().collect::<Vec<Option<i64>>>();

        assert_eq!(v, vec![Some(1), Some(4), Some(1)]);
        check_group_key_column(&chunk, 0, vec![Some(3), Some(4), Some(5)]);
        check_group_key_column(&chunk, 1, vec![Some(9), Some(9), Some(9)]);

        // check the result
        let res = stream.next().await.unwrap();
        assert_matches!(res, Ok(_));

        let chunk = res?;
        assert_eq!(chunk.cardinality(), 3);
        let actual = chunk.column_at(2).array();
        let actual_agg: &I64Array = actual.as_ref().into();
        let v = actual_agg.iter().collect::<Vec<Option<i64>>>();

        // check the result
        assert_eq!(v, vec![Some(1), Some(2), Some(6)]);
        check_group_key_column(&chunk, 0, vec![Some(6), Some(7), Some(8)]);
        check_group_key_column(&chunk, 1, vec![Some(9), Some(9), Some(9)]);

        assert_matches!(stream.next().await, None);
        Ok(())
    }

    #[tokio::test]
    async fn execute_sum_int32() -> Result<()> {
        let schema = Schema {
            fields: vec![Field::unnamed(DataType::Int32)],
        };
        let mut child = MockExecutor::new(schema);
        child.add(DataChunk::from_pretty(
            " i
              1
              2
              3
              4
              5
              6
              7
              8
              9
             10",
        ));

        let prost = AggCall {
            r#type: Type::Sum as i32,
            args: vec![Arg {
                input: Some(InputRefExpr { column_idx: 0 }),
                r#type: Some(ProstDataType {
                    type_name: TypeName::Int32 as i32,
                    ..Default::default()
                }),
            }],
            return_type: Some(ProstDataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            distinct: false,
        };

        let sum_agg = AggStateFactory::new(&prost)?.create_agg_state()?;

        let group_exprs: Vec<BoxedExpression> = vec![];
        let agg_states = vec![sum_agg];
        let fields = group_exprs
            .iter()
            .map(|e| e.return_type())
            .chain(agg_states.iter().map(|e| e.return_type()))
            .map(Field::unnamed)
            .collect::<Vec<Field>>();
        let executor = Box::new(SortAggExecutor {
            agg_states,
            group_keys: vec![],
            sorted_groupers: vec![],
            child: Box::new(child),
            schema: Schema { fields },
            identity: "SortAggExecutor".to_string(),
            output_size_limit: 4,
        });

        let mut stream = executor.execute();
        let res = stream.next().await.unwrap();
        assert_matches!(res, Ok(_));
        assert_matches!(stream.next().await, None);

        let actual = res?.column_at(0).array();
        let actual: &I64Array = actual.as_ref().into();
        let v = actual.iter().collect::<Vec<Option<i64>>>();
        assert_eq!(v, vec![Some(55)]);

        assert_matches!(stream.next().await, None);
        Ok(())
    }

    #[tokio::test]
    async fn execute_sum_int32_grouped() -> Result<()> {
        // mock a child executor
        let schema = Schema {
            fields: vec![
                Field::unnamed(DataType::Int32),
                Field::unnamed(DataType::Int32),
                Field::unnamed(DataType::Int32),
            ],
        };
        let mut child = MockExecutor::new(schema);
        child.add(DataChunk::from_pretty(
            "i i i
             1 1 7
             2 1 8
             3 3 8
             4 3 9",
        ));
        child.add(DataChunk::from_pretty(
            "i i i
             1 3 9
             2 4 9
             3 4 9
             4 5 9",
        ));
        child.add(DataChunk::from_pretty(
            "i i i
             1 5 9
             2 5 9
             3 5 9
             4 5 9",
        ));

        let prost = AggCall {
            r#type: Type::Sum as i32,
            args: vec![Arg {
                input: Some(InputRefExpr { column_idx: 0 }),
                r#type: Some(ProstDataType {
                    type_name: TypeName::Int32 as i32,
                    ..Default::default()
                }),
            }],
            return_type: Some(ProstDataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            distinct: false,
        };

        let sum_agg = AggStateFactory::new(&prost)?.create_agg_state()?;
        let group_exprs: Vec<_> = (1..=2)
            .map(|idx| {
                build_from_prost(&ExprNode {
                    expr_type: InputRef as i32,
                    return_type: Some(ProstDataType {
                        type_name: TypeName::Int32 as i32,
                        ..Default::default()
                    }),
                    rex_node: Some(RexNode::InputRef(InputRefExpr { column_idx: idx })),
                })
            })
            .try_collect()?;

        let sorted_groupers: Vec<_> = group_exprs
            .iter()
            .map(|e| create_sorted_grouper(e.return_type()))
            .try_collect()?;

        let agg_states = vec![sum_agg];

        // chain group key fields and agg state schema to get output schema for sort agg
        let fields = group_exprs
            .iter()
            .map(|e| e.return_type())
            .chain(agg_states.iter().map(|e| e.return_type()))
            .map(Field::unnamed)
            .collect::<Vec<Field>>();

        let output_size_limit = 4;
        let executor = Box::new(SortAggExecutor {
            agg_states,
            group_keys: group_exprs,
            sorted_groupers,
            child: Box::new(child),
            schema: Schema { fields },
            identity: "SortAggExecutor".to_string(),
            output_size_limit,
        });

        let fields = &executor.schema().fields;
        assert_eq!(fields[0].data_type, DataType::Int32);
        assert_eq!(fields[1].data_type, DataType::Int32);
        assert_eq!(fields[2].data_type, DataType::Int64);

        let mut stream = executor.execute();
        let res = stream.next().await.unwrap();
        assert_matches!(res, Ok(_));

        let chunk = res?;
        let actual = chunk.column_at(2).array();
        let actual_agg: &I64Array = actual.as_ref().into();
        let v = actual_agg.iter().collect::<Vec<Option<i64>>>();

        // check the result
        assert_eq!(v, vec![Some(1), Some(2), Some(3), Some(5)]);
        check_group_key_column(&chunk, 0, vec![Some(1), Some(1), Some(3), Some(3)]);
        check_group_key_column(&chunk, 1, vec![Some(7), Some(8), Some(8), Some(9)]);

        let res = stream.next().await.unwrap();
        assert_matches!(res, Ok(_));

        let chunk = res?;
        let actual2 = chunk.column_at(2).array();
        let actual_agg2: &I64Array = actual2.as_ref().into();
        let v = actual_agg2.iter().collect::<Vec<Option<i64>>>();

        // check the result
        assert_eq!(v, vec![Some(5), Some(14)]);
        check_group_key_column(&chunk, 0, vec![Some(4), Some(5)]);
        check_group_key_column(&chunk, 1, vec![Some(9), Some(9)]);

        assert_matches!(stream.next().await, None);
        Ok(())
    }

    #[tokio::test]
    async fn execute_sum_int32_grouped_execeed_limit() -> Result<()> {
        // mock a child executor
        let schema = Schema {
            fields: vec![
                Field::unnamed(DataType::Int32),
                Field::unnamed(DataType::Int32),
                Field::unnamed(DataType::Int32),
            ],
        };
        let mut child = MockExecutor::new(schema);
        child.add(DataChunk::from_pretty(
            " i  i  i
              1  1  7
              2  1  8
              3  3  8
              4  3  8
              5  4  9
              6  4  9
              7  5  9
              8  5  9
              9  6 10
             10  6 10",
        ));
        child.add(DataChunk::from_pretty(
            " i  i  i
              1  6 10
              2  7 12",
        ));

        let prost = AggCall {
            r#type: Type::Sum as i32,
            args: vec![Arg {
                input: Some(InputRefExpr { column_idx: 0 }),
                r#type: Some(ProstDataType {
                    type_name: TypeName::Int32 as i32,
                    ..Default::default()
                }),
            }],
            return_type: Some(ProstDataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            distinct: false,
        };

        let sum_agg = AggStateFactory::new(&prost)?.create_agg_state()?;
        let group_exprs: Vec<_> = (1..=2)
            .map(|idx| {
                build_from_prost(&ExprNode {
                    expr_type: InputRef as i32,
                    return_type: Some(ProstDataType {
                        type_name: TypeName::Int32 as i32,
                        ..Default::default()
                    }),
                    rex_node: Some(RexNode::InputRef(InputRefExpr { column_idx: idx })),
                })
            })
            .try_collect()?;

        let sorted_groupers = group_exprs
            .iter()
            .map(|e| create_sorted_grouper(e.return_type()))
            .collect::<Result<Vec<BoxedSortedGrouper>>>()?;

        let agg_states = vec![sum_agg];

        // chain group key fields and agg state schema to get output schema for sort agg
        let fields = group_exprs
            .iter()
            .map(|e| e.return_type())
            .chain(agg_states.iter().map(|e| e.return_type()))
            .map(Field::unnamed)
            .collect::<Vec<Field>>();

        let executor = Box::new(SortAggExecutor {
            agg_states,
            group_keys: group_exprs,
            sorted_groupers,
            child: Box::new(child),
            schema: Schema { fields },
            identity: "SortAggExecutor".to_string(),
            output_size_limit: 3,
        });

        let fields = &executor.schema().fields;
        assert_eq!(fields[0].data_type, DataType::Int32);
        assert_eq!(fields[1].data_type, DataType::Int32);
        assert_eq!(fields[2].data_type, DataType::Int64);

        // check first chunk
        let mut stream = executor.execute();
        let res = stream.next().await.unwrap();
        assert_matches!(res, Ok(_));

        let chunk = res?;
        let actual = chunk.column_at(2).array();
        let actual_agg: &I64Array = actual.as_ref().into();
        let v = actual_agg.iter().collect::<Vec<Option<i64>>>();
        assert_eq!(v, vec![Some(1), Some(2), Some(7)]);
        check_group_key_column(&chunk, 0, vec![Some(1), Some(1), Some(3)]);
        check_group_key_column(&chunk, 1, vec![Some(7), Some(8), Some(8)]);

        // check second chunk
        let res = stream.next().await.unwrap();
        assert_matches!(res, Ok(_));

        let chunk = res?;
        let actual2 = chunk.column_at(2).array();
        let actual_agg2: &I64Array = actual2.as_ref().into();
        let v = actual_agg2.iter().collect::<Vec<Option<i64>>>();
        assert_eq!(v, vec![Some(11), Some(15), Some(20)]);
        check_group_key_column(&chunk, 0, vec![Some(4), Some(5), Some(6)]);
        check_group_key_column(&chunk, 1, vec![Some(9), Some(9), Some(10)]);

        // check third chunk
        let res = stream.next().await.unwrap();
        assert_matches!(res, Ok(_));

        let chunk = res?;
        let actual2 = chunk.column_at(2).array();
        let actual_agg2: &I64Array = actual2.as_ref().into();
        let v = actual_agg2.iter().collect::<Vec<Option<i64>>>();

        assert_eq!(v, vec![Some(2)]);
        check_group_key_column(&chunk, 0, vec![Some(7)]);
        check_group_key_column(&chunk, 1, vec![Some(12)]);

        assert_matches!(stream.next().await, None);
        Ok(())
    }

    fn check_group_key_column(actual: &DataChunk, col_idx: usize, expect: Vec<Option<i32>>) {
        assert_eq!(
            actual
                .column_at(col_idx)
                .array()
                .as_int32()
                .iter()
                .collect::<Vec<_>>(),
            expect
        );
    }
}
