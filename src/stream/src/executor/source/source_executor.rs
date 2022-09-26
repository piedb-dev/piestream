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

use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use either::Either;
use futures::stream::{select_with_strategy, PollNext};
use futures::{pin_mut, Stream, StreamExt};
use futures_async_stream::try_stream;
use piestream_common::array::column::Column;
use piestream_common::array::{ArrayBuilder, ArrayImpl, I64ArrayBuilder, StreamChunk};
use piestream_common::catalog::{ColumnId, Schema, TableId};
use piestream_common::error::{internal_error, Result, RwError, ToRwResult};
use piestream_connector::{ConnectorState, SplitImpl, SplitMetaData};
use piestream_source::connector_source::SourceContext;
use piestream_source::*;
use piestream_storage::{Keyspace, StateStore};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio::sync::{Mutex, Notify};

use crate::executor::error::StreamExecutorError;
use crate::executor::monitor::StreamingMetrics;
use crate::executor::source::state::SourceStateHandler;
use crate::executor::*;

/// [`SourceExecutor`] is a streaming source, from piestream's batch table, or external systems
/// such as Kafka.
pub struct SourceExecutor<S: StateStore> {
    actor_id: ActorId,
    source_id: TableId,
    source_desc: SourceDesc,

    column_ids: Vec<ColumnId>,
    schema: Schema,
    pk_indices: PkIndices,

    /// Identity string
    identity: String,

    /// Receiver of barrier channel.
    barrier_receiver: Option<UnboundedReceiver<Barrier>>,

    // monitor
    metrics: Arc<StreamingMetrics>,

    /// Split info for stream source
    stream_source_splits: Vec<SplitImpl>,

    source_identify: String,

    split_state_store: SourceStateHandler<S>,

    state_cache: HashMap<String, SplitImpl>,

    /// Expected barrier latency
    expected_barrier_latency_ms: u64,
}

impl<S: StateStore> SourceExecutor<S> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        actor_id: ActorId,
        source_id: TableId,
        source_desc: SourceDesc,
        keyspace: Keyspace<S>,
        column_ids: Vec<ColumnId>,
        schema: Schema,
        pk_indices: PkIndices,
        barrier_receiver: UnboundedReceiver<Barrier>,
        executor_id: u64,
        _operator_id: u64,
        _op_info: String,
        streaming_metrics: Arc<StreamingMetrics>,
        expected_barrier_latency_ms: u64,
    ) -> Result<Self> {
        Ok(Self {
            actor_id,
            source_id,
            source_desc,
            column_ids,
            schema,
            pk_indices,
            barrier_receiver: Some(barrier_receiver),
            identity: format!("SourceExecutor {:X}", executor_id),
            metrics: streaming_metrics,
            stream_source_splits: vec![],
            source_identify: "Table_".to_string() + &source_id.table_id().to_string(),
            split_state_store: SourceStateHandler::new(keyspace),
            state_cache: HashMap::new(),
            expected_barrier_latency_ms,
        })
    }

    /// Generate a row ID column.
    fn gen_row_id_column(&mut self, len: usize) -> Column {
        let mut builder = I64ArrayBuilder::new(len);
        let row_ids = self.source_desc.next_row_id_batch(len);

        for row_id in row_ids {
            builder.append(Some(row_id)).unwrap();
        }

        Column::new(Arc::new(ArrayImpl::from(builder.finish().unwrap())))
    }

    fn refill_row_id_column(&mut self, chunk: StreamChunk) -> StreamChunk {
        let row_id_index = self.source_desc.row_id_index;
        let row_id_column_id = self.source_desc.columns[row_id_index as usize].column_id;

        if let Some(idx) = self
            .column_ids
            .iter()
            .position(|column_id| *column_id == row_id_column_id)
        {
            let (ops, mut columns, bitmap) = chunk.into_inner();
            columns[idx] = self.gen_row_id_column(columns[idx].array().len());
            return StreamChunk::new(ops, columns, bitmap);
        }
        chunk
    }
}

struct SourceReader {
    /// The reader for stream source.
    stream_reader: Arc<Mutex<Option<Box<SourceStreamReaderImpl>>>>,
    /// The reader for barrier.
    barrier_receiver: UnboundedReceiver<Barrier>,
    /// Expected barrier latency in ms. If there are no barrier within the expected barrier
    /// latency, source will stall.
    expected_barrier_latency_ms: u64,
}

impl SourceReader {
    #[try_stream(ok = StreamChunkWithState, error = RwError)]
    async fn stream_reader(
        stream_reader: Arc<Mutex<Option<Box<SourceStreamReaderImpl>>>>,
        #[expect(unused_variables)] notifier: Arc<Notify>,
        #[expect(unused_variables)] expected_barrier_latency_ms: u64,
        mut abort_notifier: UnboundedReceiver<()>,
    ) {
        'outer: loop {
            let mut reader = stream_reader.lock().await.take().unwrap();
            let chunk_result: Option<Result<StreamChunkWithState>>;

            {
                let chunk_future = reader.next();
                let abort_future = abort_notifier.recv();

                pin_mut!(chunk_future);
                pin_mut!(abort_future);

                //一个任务返回即返回
                match futures::future::select(chunk_future, abort_future).await {
                    futures::future::Either::Left((chunk, _)) => {
                        chunk_result = Some(chunk);
                    }
                    futures::future::Either::Right(_) => {
                        chunk_result = None;
                    }
                }
            }

            let mut reader_guard = stream_reader.lock().await;
            if reader_guard.is_none() {
                *reader_guard = Some(reader);
            } else {
                continue;
            }
            drop(reader_guard);
            if let Some(chunk) = chunk_result {
                match chunk {
                    Ok(c) => yield c,
                    Err(e) => {
                        error!("hang up stream reader due to polling error: {}", e);
                        //等价c++ go to命令
                        break 'outer;
                    }
                }
            };
        }

        futures::future::pending().await
    }

    #[try_stream(ok = Message, error = RwError)]
    async fn barrier_receiver(mut rx: UnboundedReceiver<Barrier>, notifier: Arc<Notify>) {
        while let Some(barrier) = rx.recv().await {
            yield Message::Barrier(barrier);
            //发送通知
            notifier.notify_one();
        }
        return Err(internal_error(
            "barrier reader closed unexpectedly".to_string(),
        ));
    }

    fn into_stream(
        self,
        abort_notifier: UnboundedReceiver<()>,
    ) -> impl Stream<Item = Either<Result<Message>, Result<StreamChunkWithState>>> {
        let notifier = Arc::new(Notify::new());

        //构建接收barrier stream对象
        let barrier_receiver = Self::barrier_receiver(self.barrier_receiver, notifier.clone());
        //构建读取流对像
        let stream_reader = Self::stream_reader(
            self.stream_reader,
            notifier,
            self.expected_barrier_latency_ms,
            abort_notifier,
        );
        //数据流与barrier汇合，PollNext::Left 优先遍历left数据，即barrier
        select_with_strategy(
            barrier_receiver.map(Either::Left),
            stream_reader.map(Either::Right),
            |_: &mut ()| PollNext::Left, // prefer barrier
        )
    }
}

impl<S: StateStore> SourceExecutor<S> {
    fn get_diff(&self, rhs: ConnectorState) -> ConnectorState {
        // rhs can not be None because we do not support split number reduction

        let split_change = rhs.unwrap();
        let mut target_state: Vec<SplitImpl> = Vec::with_capacity(split_change.len());
        let mut no_change_flag = true;
        for sc in &split_change {
            // SplitImpl is identified by its id, target_state always follows offsets in cache
            // here we introduce a hypothesis that every split is polled at least once in one epoch
            match self.state_cache.get(&sc.id()) {
                Some(s) => target_state.push(s.clone()),
                None => {
                    no_change_flag = false;
                    target_state.push(sc.clone())
                }
            }
        }

        if no_change_flag {
            None
        } else {
            Some(target_state)
        }
    }

    async fn take_snapshot(&mut self, epoch: u64) -> Result<()> {
        let cache = self
            .state_cache
            .iter()
            .map(|(_, split_impl)| split_impl.to_owned())
            .collect_vec();

        if !cache.is_empty() {
            self.split_state_store
                .take_snapshot(cache, epoch)
                .await
                .to_rw_result()?;
        }

        Ok(())
    }

    async fn build_stream_source_reader(
        &mut self,
        state: ConnectorState,
    ) -> Result<Box<SourceStreamReaderImpl>> {
        let reader = match self.source_desc.source.as_ref() {
            SourceImpl::TableV2(t) => t
                .stream_reader(self.column_ids.clone())
                .await
                .map(SourceStreamReaderImpl::TableV2),
            SourceImpl::Connector(c) => c
                .stream_reader(
                    state,
                    self.column_ids.clone(),
                    self.source_desc.metrics.clone(),
                    SourceContext::new(self.actor_id as u32, self.source_id),
                )
                .await
                .map(SourceStreamReaderImpl::Connector),
        }?;

        Ok(Box::new(reader))
    }

    #[try_stream(ok = Message, error = StreamExecutorError)]
    async fn into_stream(mut self) {
        let mut barrier_receiver = self.barrier_receiver.take().unwrap();
        let barrier = barrier_receiver.recv().await.unwrap();

        if let Some(mutation) = barrier.mutation.as_ref() {
            //通过actor_id获取到splits
            if let Mutation::AddOutput(add_output) = mutation.as_ref() {
                if let Some(splits) = add_output.splits.get(&self.actor_id) {
                    self.stream_source_splits = splits.clone();
                }
            }
        }

        let epoch = barrier.epoch.prev;

        let mut boot_state = self.stream_source_splits.clone();
        if !boot_state.is_empty() {
            for ele in &mut boot_state {
                match self
                    .split_state_store
                    .try_recover_from_state_store(ele, epoch)
                    .await
                {
                    //TODO(liqiu),从持久化里恢复，同barrier传有啥不一样？？？
                    Ok(recover_state) if recover_state.is_some() => {
                        *ele = recover_state.unwrap();
                    }
                    Err(e) => {
                        return Err(StreamExecutorError::source_error(e));
                    }
                    _ => {}
                }
            }
        }
        let recover_state: ConnectorState = if boot_state.is_empty() {
            None
        } else {
            Some(boot_state)
        };

        // todo: use epoch from msg to restore state from state store
        let stream_reader = Arc::new(Mutex::new(Some(
            self.build_stream_source_reader(recover_state)
                .await
                .map_err(StreamExecutorError::source_error)?,
        )));

        let reader = SourceReader {
            stream_reader: stream_reader.clone(),
            barrier_receiver,
            expected_barrier_latency_ms: self.expected_barrier_latency_ms,
        };
        yield Message::Barrier(barrier);

        let (abort_tx, abort_rx) = unbounded_channel::<()>();

        #[for_await]
        for msg in reader.into_stream(abort_rx) {
            match msg {
                // This branch will be preferred.
                Either::Left(barrier) => {
                    match barrier.map_err(StreamExecutorError::source_error)? {
                        Message::Barrier(barrier) => {
                            let epoch = barrier.epoch.prev;
                            self.take_snapshot(epoch)
                                .await
                                .map_err(StreamExecutorError::source_error)?;

                            if let Some(Mutation::SourceChangeSplit(mapping)) =
                                barrier.mutation.as_deref()
                            {
                                if let Some(target_splits) = mapping.get(&self.actor_id).cloned() {
                                    match self.get_diff(target_splits) {
                                        None => {}
                                        Some(target_state) => {
                                            log::info!(
                                                "actor {:?} apply source split change to {:?}",
                                                self.actor_id,
                                                target_state
                                            );
                                            let reader = self
                                                .build_stream_source_reader(Some(
                                                    target_state.clone(),
                                                ))
                                                .await
                                                .map_err(StreamExecutorError::source_error)?;
                                            abort_tx.send(()).unwrap();
                                            *stream_reader.lock().await = Some(reader);
                                            self.stream_source_splits = target_state;
                                        }
                                    }
                                }
                            }
                            self.state_cache.clear();
                            yield Message::Barrier(barrier)
                        }
                        _ => unreachable!(),
                    }
                }
                Either::Right(chunk_with_state) => {
                    let chunk_with_state =
                        chunk_with_state.map_err(StreamExecutorError::source_error)?;
                    if chunk_with_state.split_offset_mapping.is_some() {
                        let mapping: HashMap<String, String> =
                            chunk_with_state.split_offset_mapping.unwrap();
                        let state: HashMap<String, SplitImpl> = mapping
                            .iter()
                            .map(|(split, offset)| {
                                let origin_split_impl = self
                                    .stream_source_splits
                                    .iter()
                                    .filter(|origin_split| origin_split.id().as_str() == split)
                                    .collect::<Vec<&SplitImpl>>();
                                if origin_split_impl.is_empty() {
                                    Err(internal_error(format!(
                                        "cannot find split: {:?} in stream_source_splits: {:?}",
                                        split, self.stream_source_splits
                                    )))
                                } else {
                                    Ok((split.clone(), origin_split_impl[0].update(offset.clone())))
                                }
                            })
                            .collect::<Result<HashMap<String, SplitImpl>>>()
                            .map_err(StreamExecutorError::source_error)?;
                        self.state_cache.extend(state);
                    }
                    let mut chunk = chunk_with_state.chunk;

                    if !matches!(self.source_desc.source.as_ref(), SourceImpl::TableV2(_)) {
                        chunk = self.refill_row_id_column(chunk);
                    }

                    self.metrics
                        .source_output_row_count
                        .with_label_values(&[self.source_identify.as_str()])
                        .inc_by(chunk.cardinality() as u64);
                    yield Message::Chunk(chunk);
                }
            }
        }
        unreachable!();
    }
}

impl<S: StateStore> Executor for SourceExecutor<S> {
    fn execute(self: Box<Self>) -> BoxedMessageStream {
        self.into_stream().boxed()
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn pk_indices(&self) -> PkIndicesRef {
        &self.pk_indices
    }

    fn identity(&self) -> &str {
        self.identity.as_str()
    }
}

impl<S: StateStore> Debug for SourceExecutor<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceExecutor")
            .field("source_id", &self.source_id)
            .field("column_ids", &self.column_ids)
            .field("pk_indices", &self.pk_indices)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use futures::StreamExt;
    use maplit::hashmap;
    use piestream_common::array::stream_chunk::StreamChunkTestExt;
    use piestream_common::array::StreamChunk;
    use piestream_common::catalog::{ColumnDesc, Field, Schema};
    use piestream_common::types::DataType;
    use piestream_common::util::sort_util::{OrderPair, OrderType};
    use piestream_connector::datagen::DatagenSplit;
    use piestream_pb::catalog::StreamSourceInfo;
    use piestream_pb::data::data_type::TypeName;
    use piestream_pb::data::DataType as ProstDataType;
    use piestream_pb::plan_common::{
        ColumnCatalog as ProstColumnCatalog, ColumnDesc as ProstColumnDesc,
        RowFormatType as ProstRowFormatType,
    };
    use piestream_source::*;
    use piestream_storage::memory::MemoryStateStore;
    use tokio::sync::mpsc::unbounded_channel;

    use super::*;

    #[tokio::test]
    async fn test_table_source() -> Result<()> {
        let table_id = TableId::default();

        let rowid_type = DataType::Int64;
        let col1_type = DataType::Int32;
        let col2_type = DataType::Varchar;

        let table_columns = vec![
            ColumnDesc {
                column_id: ColumnId::from(0),
                data_type: rowid_type.clone(),
                name: String::new(),
                field_descs: vec![],
                type_name: "".to_string(),
            },
            ColumnDesc {
                column_id: ColumnId::from(1),
                data_type: col1_type.clone(),
                name: String::new(),
                field_descs: vec![],
                type_name: "".to_string(),
            },
            ColumnDesc {
                column_id: ColumnId::from(2),
                data_type: col2_type.clone(),
                name: String::new(),
                field_descs: vec![],
                type_name: "".to_string(),
            },
        ];
        let source_manager = MemSourceManager::default();
        source_manager.create_table_source(&table_id, table_columns)?;
        let source_desc = source_manager.get_source(&table_id)?;
        let source = source_desc.clone().source;

        let chunk1 = StreamChunk::from_pretty(
            " I i T
            + 0 1 foo
            + 0 2 bar
            + 0 3 baz",
        );
        let chunk2 = StreamChunk::from_pretty(
            " I i T
            + 0 4 hello
            + 0 5 .
            + 0 6 world",
        );

        let schema = Schema {
            fields: vec![
                Field::unnamed(rowid_type),
                Field::unnamed(col1_type),
                Field::unnamed(col2_type),
            ],
        };

        let column_ids = vec![0, 1, 2].into_iter().map(ColumnId::from).collect();
        let pk_indices = vec![0];

        let (barrier_sender, barrier_receiver) = unbounded_channel();
        let keyspace = Keyspace::table_root(MemoryStateStore::new(), &TableId::from(0x2333));

        let executor = SourceExecutor::new(
            0x3f3f3f,
            table_id,
            source_desc,
            keyspace,
            column_ids,
            schema,
            pk_indices,
            barrier_receiver,
            1,
            1,
            "SourceExecutor".to_string(),
            Arc::new(StreamingMetrics::new(prometheus::Registry::new())),
            u64::MAX,
        )
        .unwrap();
        let mut executor = Box::new(executor).execute();

        let write_chunk = |chunk: StreamChunk| {
            let source = source.clone();
            tokio::spawn(async move {
                let table_source = source.as_table_v2().unwrap();
                table_source.blocking_write_chunk(chunk).await.unwrap();
            });
        };

        barrier_sender.send(Barrier::new_test_barrier(1)).unwrap();

        // Write 1st chunk
        write_chunk(chunk1);

        for _ in 0..2 {
            match executor.next().await.unwrap().unwrap() {
                Message::Chunk(chunk) => assert_eq!(
                    chunk,
                    StreamChunk::from_pretty(
                        " I i T
                        + 0 1 foo
                        + 0 2 bar
                        + 0 3 baz",
                    )
                ),
                Message::Barrier(barrier) => {
                    assert_eq!(barrier.epoch, Epoch::new_test_epoch(1))
                }
            }
        }

        // Write 2nd chunk
        write_chunk(chunk2);

        let msg = executor.next().await.unwrap().unwrap();
        assert_eq!(
            msg.into_chunk().unwrap(),
            StreamChunk::from_pretty(
                " I i T
                + 0 4 hello
                + 0 5 .
                + 0 6 world",
            )
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_table_dropped() -> Result<()> {
        let table_id = TableId::default();

        let rowid_type = DataType::Int64;
        let col1_type = DataType::Int32;
        let col2_type = DataType::Varchar;

        let table_columns = vec![
            ColumnDesc {
                column_id: ColumnId::from(0),
                data_type: rowid_type.clone(),
                name: String::new(),
                field_descs: vec![],
                type_name: "".to_string(),
            },
            ColumnDesc {
                column_id: ColumnId::from(1),
                data_type: col1_type.clone(),
                name: String::new(),
                field_descs: vec![],
                type_name: "".to_string(),
            },
            ColumnDesc {
                column_id: ColumnId::from(2),
                data_type: col2_type.clone(),
                name: String::new(),
                field_descs: vec![],
                type_name: "".to_string(),
            },
        ];
        let source_manager = MemSourceManager::default();
        source_manager.create_table_source(&table_id, table_columns)?;
        let source_desc = source_manager.get_source(&table_id)?;
        let source = source_desc.clone().source;

        // Prepare test data chunks
        let chunk = StreamChunk::from_pretty(
            " I i T
            + 0 1 foo
            + 0 2 bar
            + 0 3 baz",
        );

        let schema = Schema {
            fields: vec![
                Field::unnamed(rowid_type),
                Field::unnamed(col1_type),
                Field::unnamed(col2_type),
            ],
        };

        let column_ids = vec![0.into(), 1.into(), 2.into()];
        let pk_indices = vec![0];

        let (barrier_sender, barrier_receiver) = unbounded_channel();
        let keyspace = Keyspace::table_root(MemoryStateStore::new(), &TableId::from(0x2333));
        let executor = SourceExecutor::new(
            0x3f3f3f,
            table_id,
            source_desc,
            keyspace,
            column_ids,
            schema,
            pk_indices,
            barrier_receiver,
            1,
            1,
            "SourceExecutor".to_string(),
            Arc::new(StreamingMetrics::unused()),
            u64::MAX,
        )
        .unwrap();
        let mut executor = Box::new(executor).execute();

        let write_chunk = |chunk: StreamChunk| {
            let source = source.clone();
            tokio::spawn(async move {
                let table_source = source.as_table_v2().unwrap();
                table_source.blocking_write_chunk(chunk).await.unwrap();
            });
        };

        write_chunk(chunk.clone());

        barrier_sender
            .send(Barrier::new_test_barrier(1).with_stop())
            .unwrap();

        executor.next().await.unwrap().unwrap();
        executor.next().await.unwrap().unwrap();
        write_chunk(chunk);

        Ok(())
    }

    fn mock_stream_source_info() -> StreamSourceInfo {
        let properties: HashMap<String, String> = hashmap! {
            "connector".to_string() => "datagen".to_string(),
            "fields.v1.min".to_string() => "1".to_string(),
            "fields.v1.max".to_string() => "1000".to_string(),
            "fields.v1.seed".to_string() => "12345".to_string(),
        };

        let columns = vec![
            ProstColumnCatalog {
                column_desc: Some(ProstColumnDesc {
                    column_type: Some(ProstDataType {
                        type_name: TypeName::Int64 as i32,
                        ..Default::default()
                    }),
                    column_id: 0,
                    ..Default::default()
                }),
                is_hidden: false,
            },
            ProstColumnCatalog {
                column_desc: Some(ProstColumnDesc {
                    column_type: Some(ProstDataType {
                        type_name: TypeName::Int32 as i32,
                        ..Default::default()
                    }),
                    column_id: 1,
                    name: "v1".to_string(),
                    ..Default::default()
                }),
                is_hidden: false,
            },
        ];

        StreamSourceInfo {
            properties,
            row_format: ProstRowFormatType::Json as i32,
            row_schema_location: "".to_string(),
            row_id_index: 0,
            columns,
            pk_column_ids: vec![0],
        }
    }

    fn drop_row_id(chunk: StreamChunk) -> StreamChunk {
        let (ops, mut columns, bitmap) = chunk.into_inner();
        columns.remove(0);
        StreamChunk::new(ops, columns, bitmap)
    }

    #[tokio::test]
    async fn test_split_change_mutation() -> Result<()> {
        let stream_source_info = mock_stream_source_info();
        let source_table_id = TableId::default();
        let source_manager = Arc::new(MemSourceManager::default());

        source_manager
            .create_source(&source_table_id, stream_source_info)
            .await?;

        let get_schema = |column_ids: &[ColumnId], source_desc: &SourceDesc| {
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

        let actor_id = ActorId::default();
        let source_desc = source_manager.get_source(&source_table_id)?;
        let mem_state_store = MemoryStateStore::new();
        let keyspace = Keyspace::table_root(mem_state_store.clone(), &TableId::from(0x2333));
        let column_ids = vec![ColumnId::from(0), ColumnId::from(1)];
        let schema = get_schema(&column_ids, &source_desc);
        let pk_indices = vec![0_usize];
        let (barrier_tx, barrier_rx) = unbounded_channel::<Barrier>();

        let source_exec = SourceExecutor::new(
            actor_id,
            source_table_id,
            source_desc,
            keyspace.clone(),
            column_ids.clone(),
            schema,
            pk_indices,
            barrier_rx,
            1,
            1,
            "SourceExecutor".to_string(),
            Arc::new(StreamingMetrics::unused()),
            u64::MAX,
        )?;

        let mut materialize = MaterializeExecutor::new_for_test(
            Box::new(source_exec),
            mem_state_store.clone(),
            TableId::from(0x2333),
            vec![OrderPair::new(0, OrderType::Ascending)],
            column_ids.clone(),
            2,
        )
        .boxed()
        .execute();

        let curr_epoch = 1919;
        let init_barrier =
            Barrier::new_test_barrier(curr_epoch).with_mutation(Mutation::AddOutput(AddOutput {
                map: HashMap::new(),
                splits: hashmap! {
                    ActorId::default() => vec![
                        SplitImpl::Datagen(
                        DatagenSplit {
                            split_index: 0,
                            split_num: 3,
                            start_offset: None,
                        }),
                    ],
                },
            }));
        barrier_tx.send(init_barrier).unwrap();

        let _ = materialize.next().await.unwrap(); // barrier

        let chunk_1 = materialize.next().await.unwrap().unwrap().into_chunk();

        let chunk_1_truth = StreamChunk::from_pretty(
            " I i
            + 0 533
            + 0 833
            + 0 738
            + 0 344",
        );

        assert_eq!(drop_row_id(chunk_1.unwrap()), drop_row_id(chunk_1_truth));

        let change_split_mutation = Barrier::new_test_barrier(curr_epoch + 1).with_mutation(
            Mutation::SourceChangeSplit(hashmap! {
                ActorId::default() => Some(vec![
                    SplitImpl::Datagen(
                        DatagenSplit {
                            split_index: 0,
                            split_num: 3,
                            start_offset: None,
                        }
                    ), SplitImpl::Datagen(
                        DatagenSplit {
                            split_index: 1,
                            split_num: 3,
                            start_offset: None,
                        }
                    ),
                ])
            }),
        );
        barrier_tx.send(change_split_mutation).unwrap();

        let _ = materialize.next().await.unwrap(); // barrier

        let chunk_2 = materialize.next().await.unwrap().unwrap().into_chunk();

        let chunk_2_truth = StreamChunk::from_pretty(
            " I i
            + 0 525
            + 0 425
            + 0 29
            + 0 201",
        );
        assert_eq!(drop_row_id(chunk_2.unwrap()), drop_row_id(chunk_2_truth));

        let chunk_3 = materialize.next().await.unwrap().unwrap().into_chunk();

        let chunk_3_truth = StreamChunk::from_pretty(
            " I i
            + 0 833
            + 0 533
            + 0 344",
        );
        assert_eq!(drop_row_id(chunk_3.unwrap()), drop_row_id(chunk_3_truth));

        Ok(())
    }
}
