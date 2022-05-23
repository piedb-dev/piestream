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

use std::sync::Arc;

use futures_async_stream::try_stream;
use itertools::Itertools;
use risingwave_common::array::column::Column;
use risingwave_common::array::{ArrayBuilderImpl, ArrayRef, DataChunk};
use risingwave_common::catalog::{Field, Schema};
use risingwave_common::error::{ErrorCode, Result, RwError};
use risingwave_common::util::chunk_coalesce::DEFAULT_CHUNK_BUFFER_SIZE;
use risingwave_expr::expr::{build_from_prost, BoxedExpression};
use risingwave_expr::vector_op::agg::{
    create_sorted_grouper, AggStateFactory, BoxedAggState, BoxedSortedGrouper, EqGroups,
};
use risingwave_pb::batch_plan::plan_node::NodeBody;

use crate::executor::ExecutorBuilder;
use crate::executor2::{BoxedDataChunkStream, BoxedExecutor2, BoxedExecutor2Builder, Executor2};

/// `SortAggExecutor` implements the sort aggregate algorithm, which assumes
/// that the input chunks has already been sorted by group columns.
/// The aggregation will be applied to tuples within the same group.
/// And the output schema is `[group columns, agg result]`.
///
/// As a special case, simple aggregate without groups satisfies the requirement
/// automatically because all tuples should be aggregated together.
pub struct SortAggExecutor2 {
    agg_states: Vec<BoxedAggState>,
    group_keys: Vec<BoxedExpression>,
    sorted_groupers: Vec<BoxedSortedGrouper>,
    child: BoxedExecutor2,
    schema: Schema,
    identity: String,
    output_size_limit: usize, // make unit test easy
}

impl BoxedExecutor2Builder for SortAggExecutor2 {
    fn new_boxed_executor2(source: &ExecutorBuilder) -> Result<BoxedExecutor2> {
        ensure!(source.plan_node().get_children().len() == 1);
        let proto_child =
            source.plan_node().get_children().get(0).ok_or_else(|| {
                ErrorCode::InternalError("SortAgg must have child node".to_string())
            })?;
        let child = source.clone_for_plan(proto_child).build2()?;

        let sort_agg_node = try_match_expand!(
            source.plan_node().get_node_body().unwrap(),
            NodeBody::SortAgg
        )?;

        let agg_states = sort_agg_node
            .get_agg_calls()
            .iter()
            .map(|x| AggStateFactory::new(x)?.create_agg_state())
            .collect::<Result<Vec<BoxedAggState>>>()?;

        let group_keys = sort_agg_node
            .get_group_keys()
            .iter()
            .map(build_from_prost)
            .collect::<Result<Vec<BoxedExpression>>>()?;

        let sorted_groupers = group_keys
            .iter()
            .map(|e| create_sorted_grouper(e.return_type()))
            .collect::<Result<Vec<BoxedSortedGrouper>>>()?;

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
            child,
            schema: Schema { fields },
            identity: source.plan_node().get_identity().clone(),
            output_size_limit: DEFAULT_CHUNK_BUFFER_SIZE,
        }))
    }
}

impl Executor2 for SortAggExecutor2 {
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

impl SortAggExecutor2 {
    #[try_stream(boxed, ok = DataChunk, error = RwError)]
    async fn do_execute(mut self: Box<Self>) {
        let mut left_capacity = self.output_size_limit;
        let (mut group_builders, mut agg_builders) =
            SortAggExecutor2::create_builders(&self.group_keys, &self.agg_states);

        #[for_await]
        for child_chunk in self.child.execute() {
            let child_chunk = child_chunk?.compact()?;
            let group_columns = self
                .group_keys
                .iter_mut()
                .map(|expr| expr.eval(&child_chunk))
                .collect::<Result<Vec<_>>>()?;

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

                SortAggExecutor2::build_sorted_groups(
                    &mut self.sorted_groupers,
                    &group_columns,
                    &mut group_builders,
                    &groups,
                )?;

                SortAggExecutor2::build_agg_states(
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

                    let output = DataChunk::builder().columns(columns).build();
                    yield output;

                    // reset builders and capactiy to build next output chunk
                    (group_builders, agg_builders) =
                        SortAggExecutor2::create_builders(&self.group_keys, &self.agg_states);

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

        let output = match columns.is_empty() {
            // Zero group column means SimpleAgg, which always returns 1 row.
            true => DataChunk::new_dummy(1),
            false => DataChunk::builder().columns(columns).build(),
        };

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
            .collect::<Result<Vec<_>>>();

        let agg_builders = agg_states
            .iter()
            .map(|e| e.return_type().create_array_builder(1))
            .collect::<Result<Vec<_>>>();

        (group_builders.unwrap(), agg_builders.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use futures::StreamExt;
    use risingwave_common::array::{Array as _, I32Array, I64Array};
    use risingwave_common::array_nonnull;
    use risingwave_common::catalog::{Field, Schema};
    use risingwave_common::types::DataType;
    use risingwave_expr::expr::build_from_prost;
    use risingwave_pb::data::data_type::TypeName;
    use risingwave_pb::data::DataType as ProstDataType;
    use risingwave_pb::expr::agg_call::{Arg, Type};
    use risingwave_pb::expr::expr_node::RexNode;
    use risingwave_pb::expr::expr_node::Type::InputRef;
    use risingwave_pb::expr::{AggCall, ExprNode, InputRefExpr};

    use super::*;
    use crate::executor::test_utils::MockExecutor;

    #[tokio::test]
    #[allow(clippy::many_single_char_names)]
    async fn execute_count_star_int32() -> Result<()> {
        use risingwave_common::array::ArrayImpl;
        let anchor: Arc<ArrayImpl> = Arc::new(array_nonnull! { I32Array, [1, 2, 3, 4] }.into());

        let chunk = DataChunk::builder()
            .columns(vec![
                Column::new(anchor.clone()),
                Column::new(Arc::new(array_nonnull! { I32Array, [1, 1, 3, 3] }.into())),
                Column::new(Arc::new(array_nonnull! { I32Array, [7, 8, 8, 9] }.into())),
            ])
            .build();

        // mock a child executor
        let schema = Schema {
            fields: vec![
                Field::unnamed(DataType::Int32),
                Field::unnamed(DataType::Int32),
                Field::unnamed(DataType::Int32),
            ],
        };
        let mut child = MockExecutor::new(schema);
        child.add(chunk);

        let chunk = DataChunk::builder()
            .columns(vec![
                Column::new(anchor.clone()),
                Column::new(Arc::new(array_nonnull! { I32Array, [3, 4, 4, 5] }.into())),
                Column::new(Arc::new(array_nonnull! { I32Array, [9, 9, 9, 9] }.into())),
            ])
            .build();
        child.add(chunk);

        let chunk = DataChunk::builder()
            .columns(vec![
                Column::new(anchor.clone()),
                Column::new(Arc::new(array_nonnull! { I32Array, [5, 5, 5, 5] }.into())),
                Column::new(Arc::new(array_nonnull! { I32Array, [9, 9, 9, 9] }.into())),
            ])
            .build();
        child.add(chunk);

        let prost = AggCall {
            r#type: Type::Count as i32,
            args: vec![],
            return_type: Some(risingwave_pb::data::DataType {
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

        let executor = Box::new(SortAggExecutor2 {
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
    #[allow(clippy::many_single_char_names)]
    async fn execute_count_star_int32_grouped() -> Result<()> {
        use risingwave_common::array::ArrayImpl;
        let anchor: Arc<ArrayImpl> = Arc::new(array_nonnull! { I32Array, [1, 2, 3, 4, 5] }.into());

        let chunk = DataChunk::builder()
            .columns(vec![
                Column::new(anchor.clone()),
                Column::new(Arc::new(
                    array_nonnull! { I32Array, [1, 1, 3, 3, 4] }.into(),
                )),
                Column::new(Arc::new(
                    array_nonnull! { I32Array, [7, 8, 8, 9, 9] }.into(),
                )),
            ])
            .build();

        // mock a child executor
        let schema = Schema {
            fields: vec![
                Field::unnamed(DataType::Int32),
                Field::unnamed(DataType::Int32),
                Field::unnamed(DataType::Int32),
            ],
        };
        let mut child = MockExecutor::new(schema);
        child.add(chunk);

        let chunk = DataChunk::builder()
            .columns(vec![
                Column::new(Arc::new(
                    array_nonnull! { I32Array, [1, 2, 3, 4, 5, 6, 7, 8] }.into(),
                )),
                Column::new(Arc::new(
                    array_nonnull! { I32Array, [4, 4, 4, 5, 6, 7, 7, 8] }.into(),
                )),
                Column::new(Arc::new(
                    array_nonnull! { I32Array, [9, 9, 9, 9, 9, 9, 9, 9] }.into(),
                )),
            ])
            .build();
        child.add(chunk);

        let chunk = DataChunk::builder()
            .columns(vec![
                Column::new(anchor.clone()),
                Column::new(Arc::new(
                    array_nonnull! { I32Array, [8, 8, 8, 8, 8] }.into(),
                )),
                Column::new(Arc::new(
                    array_nonnull! { I32Array, [9, 9, 9, 9, 9] }.into(),
                )),
            ])
            .build();
        child.add(chunk);

        let prost = AggCall {
            r#type: Type::Count as i32,
            args: vec![],
            return_type: Some(risingwave_pb::data::DataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            distinct: false,
        };

        let count_star = AggStateFactory::new(&prost)?.create_agg_state()?;
        let group_exprs = (1..=2)
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
            .collect::<Result<Vec<BoxedExpression>>>()?;

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

        let executor = Box::new(SortAggExecutor2 {
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
    #[allow(clippy::many_single_char_names)]
    async fn execute_sum_int32() -> Result<()> {
        let a = Arc::new(array_nonnull! { I32Array, [1,2,3,4,5,6,7,8,9,10] }.into());
        let chunk = DataChunk::builder().columns(vec![Column::new(a)]).build();
        let schema = Schema {
            fields: vec![Field::unnamed(DataType::Int32)],
        };
        let mut child = MockExecutor::new(schema);
        child.add(chunk);

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
        let executor = Box::new(SortAggExecutor2 {
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
    #[allow(clippy::many_single_char_names)]
    async fn execute_sum_int32_grouped() -> Result<()> {
        use risingwave_common::array::ArrayImpl;
        let anchor: Arc<ArrayImpl> = Arc::new(array_nonnull! { I32Array, [1, 2, 3, 4] }.into());

        let chunk = DataChunk::builder()
            .columns(vec![
                Column::new(anchor.clone()),
                Column::new(Arc::new(array_nonnull! { I32Array, [1, 1, 3, 3] }.into())),
                Column::new(Arc::new(array_nonnull! { I32Array, [7, 8, 8, 9] }.into())),
            ])
            .build();

        // mock a child executor
        let schema = Schema {
            fields: vec![
                Field::unnamed(DataType::Int32),
                Field::unnamed(DataType::Int32),
                Field::unnamed(DataType::Int32),
            ],
        };
        let mut child = MockExecutor::new(schema);
        child.add(chunk);

        let chunk = DataChunk::builder()
            .columns(vec![
                Column::new(anchor.clone()),
                Column::new(Arc::new(array_nonnull! { I32Array, [3, 4, 4, 5] }.into())),
                Column::new(Arc::new(array_nonnull! { I32Array, [9, 9, 9, 9] }.into())),
            ])
            .build();
        child.add(chunk);

        let chunk = DataChunk::builder()
            .columns(vec![
                Column::new(anchor.clone()),
                Column::new(Arc::new(array_nonnull! { I32Array, [5, 5, 5, 5] }.into())),
                Column::new(Arc::new(array_nonnull! { I32Array, [9, 9, 9, 9] }.into())),
            ])
            .build();
        child.add(chunk);

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
        let group_exprs = (1..=2)
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
            .collect::<Result<Vec<BoxedExpression>>>()?;

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

        let output_size_limit = anchor.len();
        let executor = Box::new(SortAggExecutor2 {
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
    #[allow(clippy::many_single_char_names)]
    async fn execute_sum_int32_grouped_execeed_limit() -> Result<()> {
        let chunk = DataChunk::builder()
            .columns(vec![
                Column::new(Arc::new(
                    array_nonnull! { I32Array, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] }.into(),
                )),
                Column::new(Arc::new(
                    array_nonnull! { I32Array, [1, 1, 3, 3, 4, 4, 5, 5, 6, 6] }.into(),
                )),
                Column::new(Arc::new(
                    array_nonnull! { I32Array, [7, 8, 8, 8, 9, 9, 9, 9, 10, 10] }.into(),
                )),
            ])
            .build();

        // mock a child executor
        let schema = Schema {
            fields: vec![
                Field::unnamed(DataType::Int32),
                Field::unnamed(DataType::Int32),
                Field::unnamed(DataType::Int32),
            ],
        };
        let mut child = MockExecutor::new(schema);
        child.add(chunk);

        let chunk = DataChunk::builder()
            .columns(vec![
                Column::new(Arc::new(array_nonnull! { I32Array, [1, 2] }.into())),
                Column::new(Arc::new(array_nonnull! { I32Array, [6, 7] }.into())),
                Column::new(Arc::new(array_nonnull! { I32Array, [10, 12] }.into())),
            ])
            .build();
        child.add(chunk);

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
        let group_exprs = (1..=2)
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
            .collect::<Result<Vec<BoxedExpression>>>()?;

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

        let executor = Box::new(SortAggExecutor2 {
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
