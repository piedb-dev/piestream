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

use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use futures_async_stream::try_stream;
use itertools::Itertools;
use piestream_common::array::{Op, Row, StreamChunk};
use piestream_common::buffer::Bitmap;
use piestream_common::catalog::Schema;
use piestream_common::types::DataType;
use piestream_common::util::chunk_coalesce::DataChunkBuilder;
use piestream_common::util::epoch::EpochPair;
use piestream_common::util::sort_util::{OrderPair, OrderType};

use crate::executor::error::{StreamExecutorError, StreamExecutorResult};
use crate::executor::{
    expect_first_barrier, BoxedExecutor, BoxedMessageStream, Executor, Message, PkIndices,
    PkIndicesRef,
};

#[async_trait]
pub trait TopNExecutorBase: Send + 'static {
    /// Apply the chunk to the dirty state and get the diffs.
    async fn apply_chunk(&mut self, chunk: StreamChunk) -> StreamExecutorResult<StreamChunk>;

    /// Flush the buffered chunk to the storage backend.
    async fn flush_data(&mut self, epoch: EpochPair) -> StreamExecutorResult<()>;

    /// See [`Executor::schema`].
    fn schema(&self) -> &Schema;

    /// See [`Executor::pk_indices`].
    fn pk_indices(&self) -> PkIndicesRef<'_>;

    /// See [`Executor::identity`].
    fn identity(&self) -> &str;

    /// Update the vnode bitmap for the state tables, only used by Group Top-N since it's
    /// distributed.
    fn update_state_table_vnode_bitmap(&mut self, _vnode_bitmap: Arc<Bitmap>) {}

    async fn init(&mut self, epoch: EpochPair) -> StreamExecutorResult<()>;
}

/// The struct wraps a [`TopNExecutorBase`]
pub struct TopNExecutorWrapper<E> {
    pub(super) input: BoxedExecutor,
    pub(super) inner: E,
}

impl<E> Executor for TopNExecutorWrapper<E>
where
    E: TopNExecutorBase,
{
    fn execute(self: Box<Self>) -> BoxedMessageStream {
        self.top_n_executor_execute().boxed()
    }

    fn schema(&self) -> &Schema {
        self.inner.schema()
    }

    fn pk_indices(&self) -> PkIndicesRef<'_> {
        self.inner.pk_indices()
    }

    fn identity(&self) -> &str {
        self.inner.identity()
    }
}

impl<E> TopNExecutorWrapper<E>
where
    E: TopNExecutorBase,
{
    /// We remark that topN executor diffs from aggregate executor as it must output diffs
    /// whenever it applies a batch of input data. Therefore, topN executor flushes data only
    /// instead of computing diffs and flushing when receiving a barrier.
    #[try_stream(ok = Message, error = StreamExecutorError)]
    pub(crate) async fn top_n_executor_execute(mut self: Box<Self>) {
        let mut input = self.input.execute();

        let barrier = expect_first_barrier(&mut input).await?;
        self.inner.init(barrier.epoch).await?;

        yield Message::Barrier(barrier);

        #[for_await]
        for msg in input {
            let msg = msg?;
            match msg {
                Message::Chunk(chunk) => yield Message::Chunk(self.inner.apply_chunk(chunk).await?),
                Message::Barrier(barrier) => {
                    self.inner.flush_data(barrier.epoch).await?;
                    yield Message::Barrier(barrier)
                }
            };
        }
    }
}

pub fn generate_output(
    new_rows: Vec<Row>,
    new_ops: Vec<Op>,
    schema: &Schema,
) -> StreamExecutorResult<StreamChunk> {
    if !new_rows.is_empty() {
        let mut data_chunk_builder = DataChunkBuilder::new(schema.data_types(), new_rows.len() + 1);
        for row in &new_rows {
            let res = data_chunk_builder.append_one_row_from_datums(row.0.iter());
            debug_assert!(res.is_none());
        }
        // since `new_rows` is not empty, we unwrap directly
        let new_data_chunk = data_chunk_builder.consume_all().unwrap();
        let new_stream_chunk = StreamChunk::new(new_ops, new_data_chunk.columns().to_vec(), None);
        Ok(new_stream_chunk)
    } else {
        let columns = schema
            .create_array_builders(0)
            .into_iter()
            .map(|x| x.finish().into())
            .collect_vec();
        Ok(StreamChunk::new(vec![], columns, None))
    }
}

pub fn generate_executor_pk_indices_info(
    order_pairs: &[OrderPair],
    schema: &Schema,
) -> (PkIndices, Vec<DataType>, Vec<OrderType>) {
    let mut internal_key_indices = vec![];
    let mut internal_order_types = vec![];
    for order_pair in order_pairs {
        internal_key_indices.push(order_pair.column_idx);
        internal_order_types.push(order_pair.order_type);
    }
    let internal_data_types = internal_key_indices
        .iter()
        .map(|idx| schema.fields()[*idx].data_type())
        .collect();
    (
        internal_key_indices,
        internal_data_types,
        internal_order_types,
    )
}
