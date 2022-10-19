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

use anyhow::anyhow;
use piestream_common::catalog::{ColumnId, Field, Schema, TableId};
use piestream_common::types::DataType;
use piestream_pb::stream_plan::source_node::Info as SourceNodeInfo;
use piestream_source::SourceDescBuilder;
use tokio::sync::mpsc::unbounded_channel;

use super::*;
use crate::executor::state_table_handler::SourceStateTableHandler;
use crate::executor::SourceExecutor;

pub struct SourceExecutorBuilder;

impl ExecutorBuilder for SourceExecutorBuilder {
    fn new_boxed_executor(
        params: ExecutorParams,
        node: &StreamNode,
        store: impl StateStore,
        stream: &mut LocalStreamManagerCore,
    ) -> StreamResult<BoxedExecutor> {
        let node = try_match_expand!(node.get_node_body().unwrap(), NodeBody::Source)?;
        let (sender, barrier_receiver) = unbounded_channel();
        stream
            .context
            .lock_barrier_manager()
            .register_sender(params.actor_context.id, sender);

        let source_id = TableId::new(node.source_id);
        let source_builder = SourceDescBuilder::new(
            source_id,
            node.get_info()?,
            &params.env.source_manager_ref(),
        );

        let column_ids: Vec<_> = node
            .get_column_ids()
            .iter()
            .map(|i| ColumnId::from(*i))
            .collect();
        let columns = node
            .get_info()
            .map(|info| match info {
                SourceNodeInfo::StreamSource(stream) => &stream.columns,
                SourceNodeInfo::TableSource(table) => &table.columns,
            })
            .map_err(|_| anyhow!("source_info not found"))?;
        let fields = columns
            .iter()
            .map(|prost| {
                let column_desc = prost.column_desc.as_ref().unwrap();
                let data_type = DataType::from(column_desc.column_type.as_ref().unwrap());
                let name = column_desc.name.clone();
                Field::with_name(data_type, name)
            })
            .collect();
        let schema = Schema::new(fields);

        let vnodes = params
            .vnode_bitmap
            .expect("vnodes not set for source executor");

        let state_table_handler =
            SourceStateTableHandler::from_table_catalog(node.state_table.as_ref().unwrap(), store);

        Ok(Box::new(SourceExecutor::new(
            params.actor_context,
            source_builder,
            source_id,
            vnodes,
            state_table_handler,
            column_ids,
            schema,
            params.pk_indices,
            barrier_receiver,
            params.executor_id,
            params.operator_id,
            params.op_info,
            params.executor_stats,
            stream.config.barrier_interval_ms as u64,
        )?))
    }
}
