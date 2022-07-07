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

use std::iter::once;

use futures::future::try_join_all;
use futures_async_stream::try_stream;
use risingwave_common::array::column::Column;
use risingwave_common::array::{
    ArrayBuilder, DataChunk, I64ArrayBuilder, Op, PrimitiveArrayBuilder, StreamChunk,
};
use risingwave_common::catalog::{Field, Schema, TableId};
use risingwave_common::error::ErrorCode::InternalError;
use risingwave_common::error::{ErrorCode, Result, RwError};
use risingwave_common::types::DataType;
use risingwave_pb::batch_plan::plan_node::NodeBody;
use risingwave_source::SourceManagerRef;

use crate::executor::{
    BoxedDataChunkStream, BoxedExecutor, BoxedExecutorBuilder, Executor, ExecutorBuilder,
};
use crate::task::BatchTaskContext;

/// [`InsertExecutor`] implements table insertion with values from its child executor.
pub struct InsertExecutor {
    /// Target table id.
    table_id: TableId,
    source_manager: SourceManagerRef,

    child: BoxedExecutor,
    schema: Schema,
    identity: String,
}

impl InsertExecutor {
    pub fn new(table_id: TableId, source_manager: SourceManagerRef, child: BoxedExecutor) -> Self {
        Self {
            table_id,
            source_manager,
            child,
            schema: Schema {
                fields: vec![Field::unnamed(DataType::Int64)],
            },
            identity: "InsertExecutor".to_string(),
        }
    }
}

impl Executor for InsertExecutor {
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

impl InsertExecutor {
    #[try_stream(boxed, ok = DataChunk, error = RwError)]
    async fn do_execute(self: Box<Self>) {
        let source_desc = self.source_manager.get_source(&self.table_id)?;
        let source = source_desc.source.as_table_v2().expect("not table source");

        let mut notifiers = Vec::new();

        #[for_await]
        for data_chunk in self.child.execute() {
            let data_chunk = data_chunk?;
            let len = data_chunk.cardinality();
            assert!(data_chunk.visibility().is_none());

            // add row-id column as first column
            let row_ids = source_desc.next_row_id_batch(len);
            let mut builder = I64ArrayBuilder::new(len);
            row_ids
                .into_iter()
                .for_each(|row_id| builder.append(Some(row_id)).unwrap());

            let rowid_column = once(Column::from(builder.finish().unwrap()));
            let child_columns = data_chunk.into_parts().0.into_iter();

            // Materialize plan is assembled manually with Rust frontend, so we put the row
            // id column to the first.
            let columns = rowid_column.chain(child_columns).collect();
            let chunk = StreamChunk::new(vec![Op::Insert; len], columns, None);

            let notifier = source.write_chunk(chunk)?;
            notifiers.push(notifier);
        }

        // Wait for all chunks to be taken / written.
        let rows_inserted = try_join_all(notifiers)
            .await
            .map_err(|_| {
                RwError::from(ErrorCode::InternalError(
                    "failed to wait chunks to be written".to_owned(),
                ))
            })?
            .into_iter()
            .sum::<usize>();

        // create ret value
        {
            let mut array_builder = PrimitiveArrayBuilder::<i64>::new(1);
            array_builder.append(Some(rows_inserted as i64))?;

            let array = array_builder.finish()?;
            let ret_chunk = DataChunk::new(vec![array.into()], 1);

            yield ret_chunk
        }
    }
}

#[async_trait::async_trait]
impl BoxedExecutorBuilder for InsertExecutor {
    async fn new_boxed_executor<C: BatchTaskContext>(
        source: &ExecutorBuilder<C>,
        mut inputs: Vec<BoxedExecutor>,
    ) -> Result<BoxedExecutor> {
        ensure!(inputs.len() == 1, "Insert executor should 1 child!");
        let insert_node = try_match_expand!(
            source.plan_node().get_node_body().unwrap(),
            NodeBody::Insert
        )?;

        let table_id = TableId::from(&insert_node.table_source_ref_id);

        Ok(Box::new(Self::new(
            table_id,
            source
                .context()
                .source_manager_ref()
                .ok_or_else(|| InternalError("Source manager not found".to_string()))?,
            inputs.remove(0),
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Bound;
    use std::sync::Arc;

    use futures::StreamExt;
    use risingwave_common::array::{Array, ArrayImpl, I32Array, StructArray};
    use risingwave_common::catalog::{schema_test_utils, ColumnDesc, ColumnId};
    use risingwave_common::column_nonnull;
    use risingwave_common::types::DataType;
    use risingwave_source::{MemSourceManager, SourceManager, StreamSourceReader};
    use risingwave_storage::memory::MemoryStateStore;
    use risingwave_storage::store::ReadOptions;
    use risingwave_storage::*;

    use super::*;
    use crate::executor::test_utils::MockExecutor;
    use crate::*;

    #[tokio::test]
    async fn test_insert_executor() -> Result<()> {
        let source_manager = Arc::new(MemSourceManager::default());
        let store = MemoryStateStore::new();

        // Make struct field
        let struct_field = Field::unnamed(DataType::Struct {
            fields: vec![DataType::Int32, DataType::Int32, DataType::Int32].into(),
        });

        // Schema for mock executor.
        let mut schema = schema_test_utils::ii();
        schema.fields.push(struct_field.clone());
        let mut mock_executor = MockExecutor::new(schema.clone());

        // Schema of the table
        let mut schema = schema_test_utils::iii();
        schema.fields.push(struct_field);
        let table_columns: Vec<_> = schema
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| ColumnDesc {
                data_type: f.data_type.clone(),
                column_id: ColumnId::from(i as i32), // use column index as column id
                name: f.name.clone(),
                field_descs: vec![],
                type_name: "".to_string(),
            })
            .collect();

        let col1 = column_nonnull! { I32Array, [1, 3, 5, 7, 9] };
        let col2 = column_nonnull! { I32Array, [2, 4, 6, 8, 10] };
        let array = StructArray::from_slices(
            &[true, false, false, false, false],
            vec![
                array! { I32Array, [Some(1),None,None,None,None] }.into(),
                array! { I32Array, [Some(2),None,None,None,None] }.into(),
                array! { I32Array, [Some(3),None,None,None,None] }.into(),
            ],
            vec![DataType::Int32, DataType::Int32, DataType::Int32],
        )
        .map(|x| Arc::new(x.into()))
        .unwrap();
        let col3 = Column::new(array);
        let data_chunk: DataChunk = DataChunk::new(vec![col1, col2, col3], 5);
        mock_executor.add(data_chunk.clone());

        // Create the table.
        let table_id = TableId::new(0);
        source_manager.create_table_source(&table_id, table_columns.to_vec())?;

        // Create reader
        let source_desc = source_manager.get_source(&table_id)?;
        let source = source_desc.source.as_table_v2().unwrap();
        let mut reader = source
            .stream_reader(vec![0.into(), 1.into(), 2.into(), 3.into()])
            .await?;

        // Insert
        let insert_executor = Box::new(InsertExecutor::new(
            table_id,
            source_manager.clone(),
            Box::new(mock_executor),
        ));
        let handle = tokio::spawn(async move {
            let fields = &insert_executor.schema().fields;
            assert_eq!(fields[0].data_type, DataType::Int64);

            let mut stream = insert_executor.execute();
            let result = stream.next().await.unwrap().unwrap();

            assert_eq!(
                result
                    .column_at(0)
                    .array()
                    .as_int64()
                    .iter()
                    .collect::<Vec<_>>(),
                vec![Some(5)] // inserted rows
            );
        });

        // Read
        let chunk = reader.next().await?;

        // Row id column
        assert!(chunk.chunk.columns()[0]
            .array()
            .as_int64()
            .iter()
            .all(|x| x.is_some()));

        assert_eq!(
            chunk.chunk.columns()[1]
                .array()
                .as_int32()
                .iter()
                .collect::<Vec<_>>(),
            vec![Some(1), Some(3), Some(5), Some(7), Some(9)]
        );

        assert_eq!(
            chunk.chunk.columns()[2]
                .array()
                .as_int32()
                .iter()
                .collect::<Vec<_>>(),
            vec![Some(2), Some(4), Some(6), Some(8), Some(10)]
        );

        let array: ArrayImpl = StructArray::from_slices(
            &[true, false, false, false, false],
            vec![
                array! { I32Array, [Some(1),None,None,None,None] }.into(),
                array! { I32Array, [Some(2),None,None,None,None] }.into(),
                array! { I32Array, [Some(3),None,None,None,None] }.into(),
            ],
            vec![DataType::Int32, DataType::Int32, DataType::Int32],
        )
        .unwrap()
        .into();
        assert_eq!(*chunk.chunk.columns()[3].array(), array);

        // There's nothing in store since `TableSourceV2` has no side effect.
        // Data will be materialized in associated streaming task.
        let epoch = u64::MAX;
        let full_range = (Bound::<Vec<u8>>::Unbounded, Bound::<Vec<u8>>::Unbounded);
        let store_content = store
            .scan(
                full_range,
                None,
                ReadOptions {
                    epoch,
                    table_id: Default::default(),
                    ttl: None,
                },
            )
            .await?;
        assert!(store_content.is_empty());

        handle.await.unwrap();

        Ok(())
    }
}
