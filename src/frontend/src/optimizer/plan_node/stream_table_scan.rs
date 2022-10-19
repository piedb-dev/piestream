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

use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use itertools::Itertools;
use piestream_common::catalog::{Field, TableDesc};
use piestream_pb::stream_plan::stream_node::NodeBody as ProstStreamNode;
use piestream_pb::stream_plan::StreamNode as ProstStreamPlan;

use super::{LogicalScan, PlanBase, PlanNodeId, StreamIndexScan, StreamNode};
use crate::catalog::ColumnId;
use crate::optimizer::plan_node::utils::IndicesDisplay;
use crate::optimizer::property::{Distribution, DistributionDisplay};
use crate::stream_fragmenter::BuildFragmentGraphState;

/// `StreamTableScan` is a virtual plan node to represent a stream table scan. It will be converted
/// to chain + merge node (for upstream materialize) + batch table scan when converting to `MView`
/// creation request.
#[derive(Debug, Clone)]
pub struct StreamTableScan {
    pub base: PlanBase,
    logical: LogicalScan,
    batch_plan_id: PlanNodeId,
}

impl StreamTableScan {
    pub fn new(logical: LogicalScan) -> Self {
        let ctx = logical.base.ctx.clone();

        let batch_plan_id = ctx.next_plan_node_id();

        let distribution = {
            let distribution_key = logical
                .distribution_key()
                .expect("distribution key of stream chain must exist in output columns");
            if distribution_key.is_empty() {
                Distribution::Single
            } else {
                Distribution::UpstreamHashShard(distribution_key)
            }
        };
        let base = PlanBase::new_stream(
            ctx,
            logical.schema().clone(),
            logical.base.logical_pk.clone(),
            logical.functional_dependency().clone(),
            distribution,
            logical.table_desc().appendonly,
        );
        Self {
            base,
            logical,
            batch_plan_id,
        }
    }

    pub fn table_name(&self) -> &str {
        self.logical.table_name()
    }

    pub fn logical(&self) -> &LogicalScan {
        &self.logical
    }

    pub fn to_index_scan(
        &self,
        index_name: &str,
        index_table_desc: Rc<TableDesc>,
        primary_to_secondary_mapping: &HashMap<usize, usize>,
    ) -> StreamIndexScan {
        StreamIndexScan::new(self.logical.to_index_scan(
            index_name,
            index_table_desc,
            primary_to_secondary_mapping,
        ))
    }
}

impl_plan_tree_node_for_leaf! { StreamTableScan }

impl fmt::Display for StreamTableScan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.base.ctx.is_explain_verbose();
        let mut builder = f.debug_struct("StreamTableScan");

        builder
            .field("table", &format_args!("{}", self.logical.table_name()))
            .field(
                "columns",
                &format_args!(
                    "[{}]",
                    match verbose {
                        false => self.logical.column_names(),
                        true => self.logical.column_names_with_table_prefix(),
                    }
                    .join(", ")
                ),
            );

        if verbose {
            builder.field(
                "pk",
                &IndicesDisplay {
                    indices: self.logical_pk(),
                    input_schema: &self.base.schema,
                },
            );
            builder.field(
                "dist",
                &DistributionDisplay {
                    distribution: self.distribution(),
                    input_schema: &self.base.schema,
                },
            );
        }

        builder.finish()
    }
}

impl StreamNode for StreamTableScan {
    fn to_stream_prost_body(&self, _state: &mut BuildFragmentGraphState) -> ProstStreamNode {
        unreachable!("stream scan cannot be converted into a prost body -- call `adhoc_to_stream_prost` instead.")
    }
}

impl StreamTableScan {
    pub fn adhoc_to_stream_prost(&self) -> ProstStreamPlan {
        use piestream_pb::plan_common::Field as ProstField;
        use piestream_pb::stream_plan::*;

        let batch_plan_node = BatchPlanNode {
            table_desc: Some(self.logical.table_desc().to_protobuf()),
            column_ids: self
                .logical
                .output_column_ids()
                .iter()
                .map(ColumnId::get_id)
                .collect(),
        };

        let stream_key = self.logical_pk().iter().map(|x| *x as u32).collect_vec();

        ProstStreamPlan {
            fields: self.schema().to_prost(),
            input: vec![
                // The merge node should be empty
                ProstStreamPlan {
                    node_body: Some(ProstStreamNode::Merge(Default::default())),
                    identity: "Upstream".into(),
                    fields: self
                        .logical
                        .table_desc()
                        .columns
                        .iter()
                        .map(|c| Field::from(c).to_prost())
                        .collect(),
                    stream_key: self
                        .logical
                        .table_desc()
                        .stream_key
                        .iter()
                        .map(|i| *i as _)
                        .collect(),
                    ..Default::default()
                },
                ProstStreamPlan {
                    node_body: Some(ProstStreamNode::BatchPlan(batch_plan_node)),
                    operator_id: self.batch_plan_id.0 as u64,
                    identity: "BatchPlanNode".into(),
                    stream_key: stream_key.clone(),
                    fields: self.schema().to_prost(),
                    input: vec![],
                    append_only: true,
                },
            ],
            node_body: Some(ProstStreamNode::Chain(ChainNode {
                table_id: self.logical.table_desc().table_id.table_id,
                same_worker_node: false,
                disable_rearrange: false,
                // The fields from upstream
                upstream_fields: self
                    .logical
                    .table_desc()
                    .columns
                    .iter()
                    .map(|x| ProstField {
                        data_type: Some(x.data_type.to_protobuf()),
                        name: x.name.clone(),
                    })
                    .collect(),
                // The column indices need to be forwarded to the downstream
                upstream_column_indices: self
                    .logical
                    .output_column_indices()
                    .iter()
                    .map(|&i| i as _)
                    .collect(),
                is_singleton: *self.distribution() == Distribution::Single,
            })),
            stream_key,
            operator_id: self.base.id.0 as u64,
            identity: {
                let s = format!("{}", self);
                s.replace("StreamTableScan", "Chain")
            },
            append_only: self.append_only(),
        }
    }
}
