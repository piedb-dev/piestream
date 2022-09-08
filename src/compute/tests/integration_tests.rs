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

#![feature(generators)]
#![feature(proc_macro_hygiene, stmt_expr_attributes)]

use std::sync::Arc;

use futures::stream::StreamExt;
use futures_async_stream::try_stream;
use itertools::Itertools;
use piestream_batch::executor::monitor::BatchMetrics;
use piestream_batch::executor::{
    BoxedDataChunkStream, BoxedExecutor, DeleteExecutor, Executor as BatchExecutor, InsertExecutor,
    RowSeqScanExecutor, ScanType,
};
use piestream_common::array::{Array, DataChunk, F64Array, I64Array, Row};
use piestream_common::catalog::{ColumnDesc, ColumnId, Field, OrderedColumnDesc, Schema, TableId};
use piestream_common::column_nonnull;
use piestream_common::error::{Result, RwError};
use piestream_common::test_prelude::DataChunkTestExt;
use piestream_common::types::{DataType, IntoOrdered};
use piestream_common::util::sort_util::{OrderPair, OrderType};
use piestream_pb::data::data_type::TypeName;
use piestream_pb::plan_common::ColumnDesc as ProstColumnDesc;
use piestream_source::{MemSourceManager, SourceManager};
use piestream_storage::memory::MemoryStateStore;
use piestream_storage::table::state_table::StateTable;
use piestream_storage::table::storage_table::StorageTable;
use piestream_storage::Keyspace;
use piestream_stream::executor::monitor::StreamingMetrics;
use piestream_stream::executor::{
    Barrier, Executor, MaterializeExecutor, Message, PkIndices, SourceExecutor,
};
use tokio::sync::mpsc::unbounded_channel;

struct SingleChunkExecutor {
    chunk: Option<DataChunk>,
    schema: Schema,
    identity: String,
}

impl SingleChunkExecutor {
    pub fn new(chunk: DataChunk, schema: Schema) -> Self {
        Self {
            chunk: Some(chunk),
            schema,
            identity: "SingleChunkExecutor".to_string(),
        }
    }
}

impl BatchExecutor for SingleChunkExecutor {
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

impl SingleChunkExecutor {
    #[try_stream(boxed, ok = DataChunk, error = RwError)]
    async fn do_execute(self: Box<Self>) {
        yield self.chunk.unwrap()
    }
}

/// This test checks whether batch task and streaming task work together for `TableV2` creation,
/// insertion, deletion, and materialization.
#[tokio::test]
async fn test_table_v2_materialize() -> Result<()> {
    use piestream_pb::data::DataType;

    let memory_state_store = MemoryStateStore::new();
    let source_manager = Arc::new(MemSourceManager::default());
    let source_table_id = TableId::default();
    let table_columns: Vec<ColumnDesc> = vec![
        // row id
        ProstColumnDesc {
            column_type: Some(DataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            column_id: 0,
            ..Default::default()
        }
        .into(),
        // data
        ProstColumnDesc {
            column_type: Some(DataType {
                type_name: TypeName::Double as i32,
                ..Default::default()
            }),
            column_id: 1,
            ..Default::default()
        }
        .into(),
    ];
    //创建table模式的source[非流式]
    source_manager.create_table_source(&source_table_id, table_columns)?;

    // Ensure the source exists
    let source_desc = source_manager.get_source(&source_table_id)?;
    let get_schema = |column_ids: &[ColumnId]| {
        let mut fields = Vec::with_capacity(column_ids.len());
        for &column_id in column_ids {
            let column_desc = source_desc
                .columns
                .iter()
                .find(|c| c.column_id == column_id)
                .unwrap();
            fields.push(Field::unnamed(column_desc.data_type.clone()));
        }
        Schema::new(fields)
    };

    // Create a `SourceExecutor` to read the changes
    let all_column_ids = vec![ColumnId::from(0), ColumnId::from(1)];
    //获取schema
    let all_schema = get_schema(&all_column_ids);
    let (barrier_tx, barrier_rx) = unbounded_channel();
    let keyspace = Keyspace::table_root(MemoryStateStore::new(), &TableId::from(0x2333));
    let stream_source = SourceExecutor::new(
        0x3f3f3f,
        source_table_id,
        source_desc.clone(),
        keyspace,
        all_column_ids.clone(),
        all_schema.clone(),
        PkIndices::from([0]),
        barrier_rx,
        1,
        1,
        "SourceExecutor".to_string(),
        Arc::new(StreamingMetrics::unused()),
        u64::MAX,
    )?;

    // Create a `Materialize` to write the changes to storage

    let mut materialize = MaterializeExecutor::new_for_test(
        Box::new(stream_source),
        memory_state_store.clone(),
        source_table_id,
        vec![OrderPair::new(0, OrderType::Ascending)],
        all_column_ids.clone(),
        2,
    )
    .boxed()
    .execute();

    // 1.
    // Test insertion
    //

    // Add some data using `InsertExecutor`, assuming we are inserting into the "mv"
    let chunk = DataChunk::from_pretty(
        "F
         1.14
         5.14",
    );
    let insert_inner: BoxedExecutor = Box::new(SingleChunkExecutor::new(chunk, all_schema.clone()));
    let insert = Box::new(InsertExecutor::new(
        source_table_id,
        source_manager.clone(),
        insert_inner,
    ));

    tokio::spawn(async move {
        let mut stream = insert.execute();
        let _ = stream.next().await.unwrap()?;
        Ok::<_, RwError>(())
    });

    let column_descs = all_column_ids
        .into_iter()
        .zip_eq(all_schema.fields.iter().cloned())
        .map(|(column_id, field)| ColumnDesc {
            data_type: field.data_type,
            column_id,
            name: field.name,
            field_descs: vec![],
            type_name: "".to_string(),
        })
        .collect_vec();

    // Since we have not polled `Materialize`, we cannot scan anything from this table
    let table = StorageTable::new_for_test(
        memory_state_store.clone(),
        source_table_id,
        column_descs.clone(),
        vec![OrderType::Ascending],
        vec![0],
    );

    let ordered_column_descs: Vec<OrderedColumnDesc> = column_descs
        .iter()
        .take(1)
        .map(|d| OrderedColumnDesc {
            column_desc: d.clone(),
            order: OrderType::Ascending,
        })
        .collect();

    let scan = Box::new(RowSeqScanExecutor::new(
        table.schema().clone(),
        ScanType::TableScan(
            table
                .batch_dedup_pk_iter(u64::MAX, &ordered_column_descs)
                .await?,
        ),
        1024,
        "RowSeqExecutor2".to_string(),
        Arc::new(BatchMetrics::unused()),
    ));
    let mut stream = scan.execute();
    let result = stream.next().await;

    assert!(result.is_none());
    // Send a barrier to start materialized view
    let curr_epoch = 1919;
    barrier_tx
        .send(Barrier::new_test_barrier(curr_epoch))
        .unwrap();

    assert!(matches!(
        materialize.next().await.unwrap()?,
        Message::Barrier(Barrier {
            epoch,
            mutation: None,
            ..
        }) if epoch.curr == curr_epoch
    ));

    // Poll `Materialize`, should output the same insertion stream chunk
    let message = materialize.next().await.unwrap()?;
    let mut col_row_ids = vec![];
    match message {
        Message::Chunk(c) => {
            let col_row_id = c.columns()[0].array_ref().as_int64();
            col_row_ids.push(col_row_id.value_at(0).unwrap());
            col_row_ids.push(col_row_id.value_at(1).unwrap());

            let col_data = c.columns()[1].array_ref().as_float64();
            assert_eq!(col_data.value_at(0).unwrap(), 1.14.into_ordered());
            assert_eq!(col_data.value_at(1).unwrap(), 5.14.into_ordered());
        }
        Message::Barrier(_) => panic!(),
    }

    // Send a barrier and poll again, should write changes to storage
    let curr_epoch = 1919;
    barrier_tx
        .send(Barrier::new_test_barrier(curr_epoch))
        .unwrap();

    assert!(matches!(
        materialize.next().await.unwrap()?,
        Message::Barrier(Barrier {
            epoch,
            mutation: None,
            ..
        }) if epoch.curr == curr_epoch
    ));

    // Scan the table again, we are able to get the data now!
    let scan = Box::new(RowSeqScanExecutor::new(
        table.schema().clone(),
        ScanType::TableScan(
            table
                .batch_dedup_pk_iter(u64::MAX, &ordered_column_descs)
                .await?,
        ),
        1024,
        "RowSeqScanExecutor2".to_string(),
        Arc::new(BatchMetrics::unused()),
    ));

    let mut stream = scan.execute();
    let result = stream.next().await.unwrap().unwrap();

    let col_data = result.columns()[1].array_ref().as_float64();
    assert_eq!(col_data.len(), 2);
    assert_eq!(col_data.value_at(0).unwrap(), 1.14.into_ordered());
    assert_eq!(col_data.value_at(1).unwrap(), 5.14.into_ordered());

    // 2.
    // Test deletion
    //

    // Delete some data using `DeleteExecutor`, assuming we are inserting into the "mv"
    let columns = vec![
        column_nonnull! { I64Array, [ col_row_ids[0]] }, // row id column
        column_nonnull! { F64Array, [1.14] },
    ];
    let chunk = DataChunk::new(columns.clone(), 1);
    let delete_inner: BoxedExecutor = Box::new(SingleChunkExecutor::new(chunk, all_schema.clone()));
    let delete = Box::new(DeleteExecutor::new(
        source_table_id,
        source_manager.clone(),
        delete_inner,
    ));

    tokio::spawn(async move {
        let mut stream = delete.execute();
        let _ = stream.next().await.unwrap()?;
        Ok::<_, RwError>(())
    });

    // Poll `Materialize`, should output the same deletion stream chunk
    let message = materialize.next().await.unwrap()?;
    match message {
        Message::Chunk(c) => {
            let col_row_id = c.columns()[0].array_ref().as_int64();
            assert_eq!(col_row_id.value_at(0).unwrap(), col_row_ids[0]);

            let col_data = c.columns()[1].array_ref().as_float64();
            assert_eq!(col_data.value_at(0).unwrap(), 1.14.into_ordered());
        }
        Message::Barrier(_) => panic!(),
    }

    // Send a barrier and poll again, should write changes to storage
    barrier_tx
        .send(Barrier::new_test_barrier(curr_epoch + 1))
        .unwrap();

    assert!(matches!(
        materialize.next().await.unwrap()?,
        Message::Barrier(Barrier {
            epoch,
            mutation: None,
            ..
        }) if epoch.curr == curr_epoch + 1
    ));

    // Scan the table again, we are able to see the deletion now!
    let scan = Box::new(RowSeqScanExecutor::new(
        table.schema().clone(),
        ScanType::TableScan(
            table
                .batch_dedup_pk_iter(u64::MAX, &ordered_column_descs)
                .await?,
        ),
        1024,
        "RowSeqScanExecutor2".to_string(),
        Arc::new(BatchMetrics::unused()),
    ));

    let mut stream = scan.execute();
    let result = stream.next().await.unwrap().unwrap();
    let col_data = result.columns()[1].array_ref().as_float64();
    assert_eq!(col_data.len(), 1);
    assert_eq!(col_data.value_at(0).unwrap(), 5.14.into_ordered());

    Ok(())
}

#[tokio::test]
async fn test_row_seq_scan() -> Result<()> {
    // In this test we test if the memtable can be correctly scanned for K-V pair insertions.
    let memory_state_store = MemoryStateStore::new();

    let schema = Schema::new(vec![
        Field::unnamed(DataType::Int32), // pk
        Field::unnamed(DataType::Int32),
        Field::unnamed(DataType::Int64),
    ]);
    let _column_ids = vec![ColumnId::from(0), ColumnId::from(1), ColumnId::from(2)];

    //字段
    let column_descs = vec![
        ColumnDesc::unnamed(ColumnId::from(0), schema[0].data_type.clone()),
        ColumnDesc::unnamed(ColumnId::from(1), schema[1].data_type.clone()),
        ColumnDesc::unnamed(ColumnId::from(2), schema[2].data_type.clone()),
    ];

    //状态表
    let mut state = StateTable::new(
        memory_state_store.clone(),
        TableId::from(0x42),
        column_descs.clone(),
        vec![OrderType::Ascending],
        None,
        vec![0_usize],
    );
    let table = state.storage_table().clone();

    let epoch: u64 = 0;

    state
        .insert(Row(vec![
            Some(1_i32.into()),
            Some(4_i32.into()),
            Some(7_i64.into()),
        ]))
        .unwrap();
    state
        .insert(Row(vec![
            Some(2_i32.into()),
            Some(5_i32.into()),
            Some(8_i64.into()),
        ]))
        .unwrap();
    state.commit(epoch).await.unwrap();

    //设置pk
    let pk_descs: Vec<OrderedColumnDesc> = column_descs
        .iter()
        .take(1)
        .map(|d| OrderedColumnDesc {
            column_desc: d.clone(),
            order: OrderType::Ascending,
        })
        .collect();

    println!("pk_descs={:?}", pk_descs);
    //扫表
    let executor = Box::new(RowSeqScanExecutor::new(
        table.schema().clone(),
        ScanType::TableScan(
            table
                .batch_dedup_pk_iter(u64::MAX, &pk_descs)
                .await
                .unwrap(),
        ),
        1,
        "RowSeqScanExecutor2".to_string(),
        Arc::new(BatchMetrics::unused()),
    ));

    assert_eq!(executor.schema().fields().len(), 3);

    //获取流对象
    let mut stream = executor.execute();
    let res_chunk = stream.next().await.unwrap().unwrap();

    assert_eq!(res_chunk.dimension(), 3);
    assert_eq!(
        res_chunk
            .column_at(0)
            .array()
            .as_int32()
            .iter()
            .collect::<Vec<_>>(),
        vec![Some(1)]
    );
    assert_eq!(
        res_chunk
            .column_at(1)
            .array()
            .as_int32()
            .iter()
            .collect::<Vec<_>>(),
        vec![Some(4)]
    );

    let res_chunk2 = stream.next().await.unwrap().unwrap();
    assert_eq!(res_chunk2.dimension(), 3);
    assert_eq!(
        res_chunk2
            .column_at(0)
            .array()
            .as_int32()
            .iter()
            .collect::<Vec<_>>(),
        vec![Some(2)]
    );
    assert_eq!(
        res_chunk2
            .column_at(1)
            .array()
            .as_int32()
            .iter()
            .collect::<Vec<_>>(),
        vec![Some(5)]
    );
    Ok(())
}
