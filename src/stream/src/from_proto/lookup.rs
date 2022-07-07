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

use piestream_common::catalog::{ColumnDesc, Field, Schema, TableId};
use piestream_common::util::sort_util::{OrderPair, OrderType};
use piestream_pb::stream_plan::lookup_node::ArrangementTableId;

use super::*;
use crate::executor::{LookupExecutor, LookupExecutorParams};

pub struct LookupExecutorBuilder;

impl ExecutorBuilder for LookupExecutorBuilder {
    fn new_boxed_executor(
        mut params: ExecutorParams,
        node: &StreamNode,
        store: impl StateStore,
        _stream: &mut LocalStreamManagerCore,
    ) -> Result<BoxedExecutor> {
        let lookup = try_match_expand!(node.get_node_body().unwrap(), NodeBody::Lookup)?;

        let arrangement = params.input.remove(1);
        let stream = params.input.remove(0);

        let arrangement_order_rules = lookup
            .arrange_key
            .iter()
            // TODO: allow descending order
            .map(|x| OrderPair::new(*x as usize, OrderType::Ascending))
            .collect();

        let arrangement_table_id = match lookup.arrangement_table_id.as_ref().unwrap() {
            ArrangementTableId::IndexId(x) => *x,
            ArrangementTableId::TableId(x) => *x,
        };

        let arrangement_col_descs = lookup
            .get_arrangement_table_info()?
            .column_descs
            .iter()
            .map(ColumnDesc::from)
            .collect();

        Ok(Box::new(LookupExecutor::new(LookupExecutorParams {
            schema: Schema::new(node.fields.iter().map(Field::from).collect()),
            arrangement,
            stream,
            arrangement_store: store,
            arrangement_table_id: TableId::from(arrangement_table_id),
            arrangement_col_descs,
            arrangement_order_rules,
            pk_indices: params.pk_indices,
            use_current_epoch: lookup.use_current_epoch,
            stream_join_key_indices: lookup.stream_key.iter().map(|x| *x as usize).collect(),
            arrange_join_key_indices: lookup.arrange_key.iter().map(|x| *x as usize).collect(),
            column_mapping: lookup.column_mapping.iter().map(|x| *x as usize).collect(),
        })))
    }
}
