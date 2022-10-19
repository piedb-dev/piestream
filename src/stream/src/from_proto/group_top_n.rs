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

use piestream_common::util::sort_util::OrderPair;
use piestream_storage::table::streaming_table::state_table::StateTable;

use super::*;
use crate::executor::GroupTopNExecutor;

pub struct GroupTopNExecutorBuilder;

impl ExecutorBuilder for GroupTopNExecutorBuilder {
    fn new_boxed_executor(
        mut params: ExecutorParams,
        node: &StreamNode,
        store: impl StateStore,
        _stream: &mut LocalStreamManagerCore,
    ) -> StreamResult<BoxedExecutor> {
        let node = try_match_expand!(node.get_node_body().unwrap(), NodeBody::GroupTopN)?;
        let group_by = node
            .get_group_key()
            .iter()
            .map(|idx| *idx as usize)
            .collect();
        let table = node.get_table()?;
        let vnodes = params.vnode_bitmap.map(Arc::new);
        let state_table = StateTable::from_table_catalog(table, store, vnodes);
        let order_pairs = table.get_pk().iter().map(OrderPair::from_prost).collect();

        if node.with_ties {
            Ok(GroupTopNExecutor::new_with_ties(
                params.input.remove(0),
                params.actor_context,
                order_pairs,
                (node.offset as usize, node.limit as usize),
                node.order_by_len as usize,
                params.pk_indices,
                params.executor_id,
                group_by,
                state_table,
            )?
            .boxed())
        } else {
            Ok(GroupTopNExecutor::new_without_ties(
                params.input.remove(0),
                params.actor_context,
                order_pairs,
                (node.offset as usize, node.limit as usize),
                node.order_by_len as usize,
                params.pk_indices,
                params.executor_id,
                group_by,
                state_table,
            )?
            .boxed())
        }
    }
}
