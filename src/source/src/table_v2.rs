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

use std::sync::RwLock;

use async_trait::async_trait;
use rand::prelude::SliceRandom;
use risingwave_common::array::StreamChunk;
use risingwave_common::catalog::{ColumnDesc, ColumnId};
use risingwave_common::error::Result;
use tokio::sync::{mpsc, oneshot};

use crate::{StreamChunkWithState, StreamSourceReader};

#[derive(Debug)]
struct TableSourceV2Core {
    /// The senders of the changes channel.
    ///
    /// When a `StreamReader` is created, a channel will be created and the sender will be
    /// saved here. The insert statement will take one channel randomly.
    changes_txs: Vec<mpsc::UnboundedSender<(StreamChunk, oneshot::Sender<usize>)>>,
}

/// [`TableSourceV2`] is a special internal source to handle table updates from user,
/// including insert/delete/update statements via SQL interface.
///
/// Changed rows will be send to the associated "materialize" streaming task, then be written to the
/// state store. Therefore, [`TableSourceV2`] can be simply be treated as a channel without side
/// effects.
#[derive(Debug)]
pub struct TableSourceV2 {
    core: RwLock<TableSourceV2Core>,

    /// All columns in this table.
    column_descs: Vec<ColumnDesc>,
}

impl TableSourceV2 {
    pub fn new(column_descs: Vec<ColumnDesc>) -> Self {
        let core = TableSourceV2Core {
            changes_txs: vec![],
        };

        Self {
            core: RwLock::new(core),
            column_descs,
        }
    }

    /// Asynchronously write stream chunk into table. Changes written here will be simply passed to
    /// the associated streaming task via channel, and then be materialized to storage there.
    ///
    /// Returns an oneshot channel which will be notified when the chunk is taken by some reader,
    /// and the `usize` represents the cardinality of this chunk.
    pub fn write_chunk(&self, chunk: StreamChunk) -> Result<oneshot::Receiver<usize>> {
        let tx = {
            let core = self.core.read().unwrap();
            core.changes_txs
                .choose(&mut rand::thread_rng())
                .expect("no table reader exists")
                .clone()
        };

        let (notifier_tx, notifier_rx) = oneshot::channel();
        tx.send((chunk, notifier_tx))
            .expect("write chunk to table reader failed");

        Ok(notifier_rx)
    }

    /// Write stream chunk into table using `write_chunk`, and then block until a reader consumes
    /// the chunk.
    ///
    /// Returns the cardinality of this chunk.
    pub async fn blocking_write_chunk(&self, chunk: StreamChunk) -> Result<usize> {
        let rx = self.write_chunk(chunk)?;
        let written_cardinality = rx.await.unwrap();
        Ok(written_cardinality)
    }
}

// TODO: Currently batch read directly calls api from `ScannableTable` instead of using
// `BatchReader`.
#[derive(Debug)]
pub struct TableV2BatchReader;

/// [`TableV2StreamReader`] reads changes from a certain table continuously.
/// This struct should be only used for associated materialize task, thus the reader should be
/// created only once. Further streaming task relying on this table source should follow the
/// structure of "`MView` on `MView`".
#[derive(Debug)]
pub struct TableV2StreamReader {
    /// The receiver of the changes channel.
    rx: mpsc::UnboundedReceiver<(StreamChunk, oneshot::Sender<usize>)>,

    /// Mappings from the source column to the column to be read.
    column_indices: Vec<usize>,
}

#[async_trait]
impl StreamSourceReader for TableV2StreamReader {
    async fn next(&mut self) -> Result<StreamChunkWithState> {
        let (chunk, notifier) = self
            .rx
            .recv()
            .await
            .expect("TableSourceV2 dropped before associated streaming task terminated");

        // Caveats: this function is an arm of `tokio::select`. We should ensure there's no `await`
        // after here.

        let (ops, columns, bitmap) = chunk.into_inner();

        let selected_columns = self
            .column_indices
            .iter()
            .map(|i| columns[*i].clone())
            .collect();
        let chunk = StreamChunk::new(ops, selected_columns, bitmap);

        // Notify about that we've taken the chunk.
        notifier.send(chunk.cardinality()).ok();

        Ok(StreamChunkWithState {
            chunk,
            split_offset_mapping: None,
        })
    }
}

impl TableSourceV2 {
    /// Create a new stream reader.
    pub async fn stream_reader(&self, column_ids: Vec<ColumnId>) -> Result<TableV2StreamReader> {
        let column_indices = column_ids
            .into_iter()
            .map(|id| {
                self.column_descs
                    .iter()
                    .position(|c| c.column_id == id)
                    .expect("column id not exists")
            })
            .collect();

        let mut core = self.core.write().unwrap();
        let (tx, rx) = mpsc::unbounded_channel();
        core.changes_txs.push(tx);

        Ok(TableV2StreamReader { rx, column_indices })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use assert_matches::assert_matches;
    use itertools::Itertools;
    use risingwave_common::array::{Array, I64Array, Op};
    use risingwave_common::column_nonnull;
    use risingwave_common::types::DataType;
    use risingwave_storage::memory::MemoryStateStore;
    use risingwave_storage::Keyspace;

    use super::*;

    fn new_source() -> TableSourceV2 {
        let store = MemoryStateStore::new();
        let _keyspace = Keyspace::table_root(store, &Default::default());

        TableSourceV2::new(vec![ColumnDesc::unnamed(
            ColumnId::from(0),
            DataType::Int64,
        )])
    }

    #[tokio::test]
    async fn test_table_source_v2() -> Result<()> {
        let source = Arc::new(new_source());
        let mut reader = source.stream_reader(vec![ColumnId::from(0)]).await?;

        macro_rules! write_chunk {
            ($i:expr) => {{
                let source = source.clone();
                let chunk = StreamChunk::new(
                    vec![Op::Insert],
                    vec![column_nonnull!(I64Array, [$i])],
                    None,
                );
                tokio::spawn(async move {
                    source.blocking_write_chunk(chunk).await.unwrap();
                })
            }};
        }

        write_chunk!(0);

        macro_rules! check_next_chunk {
            ($i: expr) => {
                assert_matches!(reader.next().await?, chunk => {
                    assert_eq!(chunk.chunk.columns()[0].array_ref().as_int64().iter().collect_vec(), vec![Some($i)]);
                });
            }
        }

        check_next_chunk!(0);

        write_chunk!(1);
        check_next_chunk!(1);

        Ok(())
    }
}
