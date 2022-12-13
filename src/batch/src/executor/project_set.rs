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

use either::{for_both, Either};
use futures_async_stream::try_stream;
use itertools::Itertools;
use piestream_common::array::column::Column;
use piestream_common::array::{ArrayBuilder, DataChunk, I64ArrayBuilder};
use piestream_common::catalog::{Field, Schema};
use piestream_common::error::{Result, RwError};
use piestream_common::types::DataType;
use piestream_common::util::chunk_coalesce::DEFAULT_CHUNK_BUFFER_SIZE;
use piestream_expr::table_function::ProjectSetSelectItem;
use piestream_pb::batch_plan::plan_node::NodeBody;

use crate::executor::{
    BoxedDataChunkStream, BoxedExecutor, BoxedExecutorBuilder, Executor, ExecutorBuilder,
};
use crate::task::BatchTaskContext;

pub struct ProjectSetExecutor {
    select_list: Vec<ProjectSetSelectItem>,
    child: BoxedExecutor,
    schema: Schema,
    identity: String,
}

impl Executor for ProjectSetExecutor {
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

impl ProjectSetExecutor {
    //1.输入表达式生成的DataChunk最大列数为N
    //2.输入表达式生成的DataChunk最大行数为M
    //3.生成一个row_id array数组，用于返回第一列
    //最终返回DataChunk 列数等于N+1 行数为M*N
    #[try_stream(boxed, ok = DataChunk, error = RwError)]
    async fn do_execute(self: Box<Self>) {
        let data_types = self
            .select_list
            .iter()
            .map(|i| i.return_type())
            .collect_vec();
        assert!(!self.select_list.is_empty());

        #[for_await]
        for data_chunk in self.child.execute() {
            let data_chunk = data_chunk?;
            println!("*********data_chunk={:?}", data_chunk);
            // First column will be `projected_row_id`, which represents the index in the
            // output table
            let mut projected_row_id_builder = I64ArrayBuilder::new(DEFAULT_CHUNK_BUFFER_SIZE);
            let mut builders = data_types
                .iter()
                .map(|ty| ty.create_array_builder(DEFAULT_CHUNK_BUFFER_SIZE))
                .collect_vec();

            let results: Vec<_> = self
                .select_list
                .iter()
                .map(|select_item| select_item.eval(&data_chunk))
                .try_collect()?;

            let mut lens = results
                .iter()
                .map(|result| {let a=for_both!(result, r=>r.len()); println!("111**************na={:?}", a); a})
                .dedup();
            //println!("***********lens={:?}", lens);
            let cardinality = lens.next().unwrap();
            //println!("***********cardinality={:?}", cardinality);
            assert!(
                lens.next().is_none(),
                "ProjectSet has mismatched output cardinalities among select list."
            );

            // each iteration corresponds to the outputs of one input row
            for row_idx in 0..cardinality {
                let items = results
                    .iter()
                    .map(|result| match result {
                        Either::Left(arrays) => Either::Left(arrays[row_idx].clone()),
                        Either::Right(array) => Either::Right(array.value_at(row_idx)),
                    })
                    .collect_vec();
                println!("items={:?}", items);
                // The maximum length of the results of table functions will be the output length.
                let max_tf_len = items
                    .iter()
                    .map(|i| i.as_ref().map_left(|arr| arr.len()).left_or(0))
                    .max()
                    .unwrap();
                println!("max_tf_len={:?}", max_tf_len);

                for i in 0..max_tf_len {
                    projected_row_id_builder.append(Some(i as i64));
                }

                for (item, builder) in items.into_iter().zip_eq(builders.iter_mut()) {
                    match item {
                        Either::Left(array_ref) => {
                            builder.append_array(&array_ref);
                            for _ in 0..(max_tf_len - array_ref.len()) {
                                builder.append_null();
                            }
                        }
                        Either::Right(datum_ref) => {
                            for _ in 0..max_tf_len {
                                builder.append_datum_ref(datum_ref);
                            }
                        }
                    }
                }
            }
            let mut columns = Vec::with_capacity(self.select_list.len() + 1);
            let projected_row_id: Column = projected_row_id_builder.finish().into();
            let cardinality = projected_row_id.len();
            columns.push(projected_row_id);
            for builder in builders {
                columns.push(builder.finish().into())
            }

            let chunk = DataChunk::new(columns, cardinality);

            yield chunk;
        }
    }
}

#[async_trait::async_trait]
impl BoxedExecutorBuilder for ProjectSetExecutor {
    async fn new_boxed_executor<C: BatchTaskContext>(
        source: &ExecutorBuilder<'_, C>,
        inputs: Vec<BoxedExecutor>,
    ) -> Result<BoxedExecutor> {
        let [child]: [_; 1] = inputs.try_into().unwrap();

        let project_set_node = try_match_expand!(
            source.plan_node().get_node_body().unwrap(),
            NodeBody::ProjectSet
        )?;

        let select_list: Vec<_> = project_set_node
            .get_select_list()
            .iter()
            .map(ProjectSetSelectItem::from_prost)
            .try_collect()?;

        let mut fields = vec![Field::with_name(DataType::Int64, "projected_row_id")];
        fields.extend(
            select_list
                .iter()
                .map(|expr| Field::unnamed(expr.return_type())),
        );

        Ok(Box::new(Self {
            select_list,
            child,
            schema: Schema { fields },
            identity: source.plan_node().get_identity().clone(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use futures::stream::StreamExt;
    use futures_async_stream::for_await;
    use piestream_common::catalog::{Field, Schema};
    use piestream_common::test_prelude::*;
    use piestream_common::types::DataType;
    use piestream_expr::expr::{Expression, InputRefExpression, LiteralExpression};
    use piestream_expr::table_function::repeat_tf;

    use super::*;
    use crate::executor::test_utils::MockExecutor;
    use crate::executor::{Executor, ValuesExecutor};
    use crate::*;

    #[tokio::test]
    async fn test_project_set_executor() -> Result<()> {
        let chunk = DataChunk::from_pretty(
            "i     i
             1     7
             2     8
             33333 66666
             4     4
             5     3",
        );

        //InputRefExpression获取DataChunk下标为idx的Int32类型数组
        let expr1 = InputRefExpression::new(DataType::Int32, 0);
        let expr2 = repeat_tf(
            LiteralExpression::new(DataType::Int32, Some(1_i32.into())).boxed(),
            2,
        );
        let expr3 = repeat_tf(
            LiteralExpression::new(DataType::Int32, Some(2_i32.into())).boxed(),
            3,
        );
        let select_list: Vec<ProjectSetSelectItem> =
            vec![expr1.boxed().into(), expr2.into(), expr3.into()];

        let schema = schema_unnamed! { DataType::Int32, DataType::Int32 };
        //executor增加chunk
        let mut mock_executor = MockExecutor::new(schema);
        mock_executor.add(chunk);

        let fields = select_list
            .iter()
            .map(|expr| Field::unnamed(expr.return_type()))
            .collect::<Vec<Field>>();
        println!("fields={:?}", fields);

        let proj_executor = Box::new(ProjectSetExecutor {
            select_list,
            child: Box::new(mock_executor),
            schema: Schema { fields },
            identity: "ProjectSetExecutor".to_string(),
        });

        let fields = &proj_executor.schema().fields;
        //println!("proj_executor.fields={:?}", fields);
        assert_eq!(fields[0].data_type, DataType::Int32);

        let expected = vec![DataChunk::from_pretty(
            "I i     i i
             0 1     1 2
             1 1     1 2
             2 1     . 2
             0 2     1 2
             1 2     1 2
             2 2     . 2
             0 33333 1 2
             1 33333 1 2
             2 33333 . 2
             0 4     1 2
             1 4     1 2
             2 4     . 2
             0 5     1 2
             1 5     1 2
             2 5     . 2",
        )];

        //1.输入表达式生成的DataChunk最大列数为N
        //2.输入表达式生成的DataChunk最大行数为M
        //3.生成一个row_id array数组，用于返回第一列
        //最终返回DataChunk 列数等于N+1 行数为M*N
        #[for_await]
        for (i, result_chunk) in proj_executor.execute().enumerate() {
            let result_chunk = result_chunk?;
            println!("result_chunk={:?}", result_chunk);
            assert_eq!(result_chunk, expected[i]);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_project_set_dummy_chunk() {
        let literal = LiteralExpression::new(DataType::Int32, Some(1_i32.into()));
        let tf = repeat_tf(
            LiteralExpression::new(DataType::Int32, Some(2_i32.into())).boxed(),
            2,
        );

        let values_executor2: Box<dyn Executor> = Box::new(ValuesExecutor::new(
            vec![vec![]], // One single row with no column.
            Schema::default(),
            "ValuesExecutor".to_string(),
            1024,
        ));

        let proj_executor = Box::new(ProjectSetExecutor {
            select_list: vec![literal.boxed().into(), tf.into()],
            child: values_executor2,
            schema: schema_unnamed!(DataType::Int32, DataType::Int32),
            identity: "ProjectSetExecutor2".to_string(),
        });
        let mut stream = proj_executor.execute();
        let chunk = stream.next().await.unwrap().unwrap();
        assert_eq!(
            chunk,
            DataChunk::from_pretty(
                "I i i
                 0 1 2
                 1 1 2",
            ),
        );
    }
}
