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

use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use futures::future::{try_join_all, Either};
use itertools::Itertools;
use risingwave_common::array::StreamChunk;
use risingwave_common::catalog::ColumnId;
use risingwave_common::error::{internal_error, Result, RwError, ToRwResult};
use risingwave_connector::{
    Column, ConnectorProperties, ConnectorStateV2, SourceMessage, SplitImpl, SplitReaderImpl,
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use crate::common::SourceChunkBuilder;
use crate::{SourceColumnDesc, SourceParserImpl, StreamChunkWithState, StreamSourceReader};

struct InnerConnectorSourceReader {
    reader: SplitReaderImpl,
    split: SplitImpl,
}

struct InnerConnectorSourceReaderHandle {
    stop_tx: oneshot::Sender<()>,
    join_handle: JoinHandle<()>,
}

/// [`ConnectorSource`] serves as a bridge between external components and streaming or
/// batch processing. [`ConnectorSource`] introduces schema at this level while
/// [`SplitReaderImpl`] simply loads raw content from message queue or file system.
/// Parallel means that multiple [`InnerConnectorSourceReader`] will run in parallel during the
/// `next`, so that 0 or more Splits reads can be handled at the Source level.
pub struct ConnectorSourceReader {
    pub config: ConnectorProperties,
    pub parser: Arc<SourceParserImpl>,
    pub columns: Vec<SourceColumnDesc>,

    handles: Option<HashMap<String, InnerConnectorSourceReaderHandle>>,
    message_rx: UnboundedReceiver<Either<Vec<SourceMessage>, RwError>>,
    // We need to keep this tx, otherwise the channel will return none with 0 inner readers, and we
    // need to clone this tx when adding new inner readers in the future.
    #[allow(dead_code)]
    message_tx: UnboundedSender<Either<Vec<SourceMessage>, RwError>>,
}

impl InnerConnectorSourceReader {
    async fn new(
        prop: ConnectorProperties,
        split: SplitImpl,
        columns: Vec<SourceColumnDesc>,
    ) -> Result<Self> {
        log::debug!(
            "Spawning new connector source inner reader with config {:?}, split {:?}",
            prop,
            split
        );

        // Here is a workaround, we now provide the vec with only one element
        let reader = SplitReaderImpl::create(
            prop,
            ConnectorStateV2::Splits(vec![split.clone()]),
            Some(
                columns
                    .iter()
                    .cloned()
                    .map(|col| Column {
                        name: col.name,
                        data_type: col.data_type,
                    })
                    .collect_vec(),
            ),
        )
        .await
        .to_rw_result()?;

        Ok(InnerConnectorSourceReader { reader, split })
    }

    async fn run(
        &mut self,
        mut stop: oneshot::Receiver<()>,
        output: mpsc::UnboundedSender<Either<Vec<SourceMessage>, RwError>>,
    ) {
        loop {
            tokio::select! {
                biased;
                // stop chan has high priority
                _ = stop.borrow_mut() => {
                    log::debug!("connector reader {} stop signal received", self.split.id());
                    break;
                }

                chunk = self.reader.next() => {
                    match chunk.map_err(|e| internal_error(e.to_string())) {
                        Err(e) => {
                            log::error!("connector reader {} error happened {}", self.split.id(), e.to_string());
                            // just ignore
                            let _ = output.send(Either::Right(e));
                            break;
                        },
                        Ok(None) => {
                            log::warn!("connector reader {} stream stopped", self.split.id());
                            break;
                        },
                        Ok(Some(msg)) => {
                            // just ignore
                            let _ = output.send(Either::Left(msg));
                        },
                    }
                }
            }
        }
    }
}

impl SourceChunkBuilder for ConnectorSourceReader {}

#[async_trait]
impl StreamSourceReader for ConnectorSourceReader {
    async fn next(&mut self) -> Result<StreamChunkWithState> {
        let batch = self.message_rx.recv().await.unwrap();

        let batch = match batch {
            Either::Left(batch) => batch,
            Either::Right(e) => return Err(e),
        };

        let mut events = Vec::with_capacity(batch.len());
        let mut split_offset_mapping: HashMap<String, String> = HashMap::new();

        for msg in batch {
            if let Some(content) = msg.payload {
                *split_offset_mapping
                    .entry(msg.split_id.clone())
                    .or_insert_with(|| "".to_string()) = msg.offset.to_string();
                events.push(self.parser.parse(content.as_ref(), &self.columns)?);
            }
        }
        let mut ops = Vec::with_capacity(events.iter().map(|e| e.ops.len()).sum());
        let mut rows = Vec::with_capacity(events.iter().map(|e| e.rows.len()).sum());

        for event in events {
            rows.extend(event.rows);
            ops.extend(event.ops);
        }
        Ok(StreamChunkWithState {
            chunk: StreamChunk::new(
                ops,
                Self::build_columns(&self.columns, rows.as_ref())?,
                None,
            ),
            split_offset_mapping: Some(split_offset_mapping),
        })
    }
}

impl Drop for ConnectorSourceReader {
    fn drop(&mut self) {
        let handles = self.handles.take().unwrap();

        for (_, handle) in handles {
            handle.join_handle.abort();
        }
    }
}

impl ConnectorSourceReader {
    pub async fn add_split(&mut self, split: SplitImpl) -> Result<()> {
        let split_id = split.id();

        let mut reader =
            InnerConnectorSourceReader::new(self.config.clone(), split, self.columns.clone())
                .await?;
        let (stop_tx, stop_rx) = oneshot::channel();
        let sender = self.message_tx.clone();
        let join_handle = tokio::spawn(async move { reader.run(stop_rx, sender).await });

        if let Some(handles) = self.handles.as_mut() {
            handles.insert(
                split_id,
                InnerConnectorSourceReaderHandle {
                    stop_tx,
                    join_handle,
                },
            );
        }

        Ok(())
    }

    pub async fn drop_split(&mut self, split_id: String) -> Result<()> {
        let handle = self
            .handles
            .as_mut()
            .and_then(|handles| handles.remove(&split_id))
            .ok_or_else(|| internal_error(format!("could not find split {}", split_id)))
            .unwrap();
        handle.stop_tx.send(()).unwrap();
        handle
            .join_handle
            .await
            .map_err(|e| internal_error(e.to_string()))
    }
}

#[derive(Clone, Debug)]
pub struct ConnectorSource {
    pub config: ConnectorProperties,
    pub columns: Vec<SourceColumnDesc>,
    pub parser: Arc<SourceParserImpl>,
}

impl ConnectorSource {
    fn get_target_columns(&self, column_ids: Vec<ColumnId>) -> Result<Vec<SourceColumnDesc>> {
        column_ids
            .iter()
            .map(|id| {
                self.columns
                    .iter()
                    .find(|c| c.column_id == *id)
                    .ok_or_else(|| {
                        internal_error(format!(
                            "Failed to find column id: {} in source: {:?}",
                            id, self
                        ))
                    })
                    .map(|col| col.clone())
            })
            .collect::<Result<Vec<SourceColumnDesc>>>()
    }

    pub async fn stream_reader(
        &self,
        splits: Vec<SplitImpl>,
        column_ids: Vec<ColumnId>,
    ) -> Result<ConnectorSourceReader> {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut handles = HashMap::with_capacity(splits.len());
        let config = self.config.clone();
        let columns = self.get_target_columns(column_ids)?;

        let readers = try_join_all(splits.into_iter().map(|split| {
            log::debug!("spawning connector split reader for split {:?}", split);
            let props = config.clone();
            let columns = columns.clone();
            async move { InnerConnectorSourceReader::new(props, split, columns).await }
        }))
        .await?;

        for mut reader in readers {
            let split_id = reader.split.id();
            let (stop_tx, stop_rx) = oneshot::channel();
            let sender = tx.clone();
            let join_handle = tokio::spawn(async move { reader.run(stop_rx, sender).await });

            handles.insert(
                split_id,
                InnerConnectorSourceReaderHandle {
                    stop_tx,
                    join_handle,
                },
            );
        }

        Ok(ConnectorSourceReader {
            config: self.config.clone(),
            handles: Some(handles),
            message_rx: rx,
            parser: self.parser.clone(),
            columns,
            message_tx: tx,
        })
    }
}
