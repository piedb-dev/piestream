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

use std::fmt;

use piestream_pb::stream_plan::stream_node::NodeBody as ProstStreamNode;

use super::{LogicalTopN, PlanBase, PlanTreeNodeUnary, StreamNode};
use crate::optimizer::property::{Distribution, Order, OrderDisplay};
use crate::stream_fragmenter::BuildFragmentGraphState;
use crate::PlanRef;

#[derive(Debug, Clone)]
pub struct StreamGroupTopN {
    pub base: PlanBase,
    logical: LogicalTopN,
    /// an optional column index which is the vnode of each row computed by the input's consistent
    /// hash distribution
    vnode_col_idx: Option<usize>,
}

impl StreamGroupTopN {
    pub fn new(logical: LogicalTopN, vnode_col_idx: Option<usize>) -> Self {
        assert!(!logical.group_key().is_empty());
        assert!(logical.limit() > 0);
        let input = logical.input();
        let dist = match input.distribution() {
            Distribution::HashShard(_) => Distribution::HashShard(logical.group_key().to_vec()),
            Distribution::UpstreamHashShard(_) => {
                Distribution::UpstreamHashShard(logical.group_key().to_vec())
            }
            _ => input.distribution().clone(),
        };
        let base = PlanBase::new_stream(
            input.ctx(),
            input.schema().clone(),
            input.logical_pk().to_vec(),
            input.functional_dependency().clone(),
            dist,
            false,
        );
        StreamGroupTopN {
            base,
            logical,
            vnode_col_idx,
        }
    }

    pub fn limit(&self) -> usize {
        self.logical.limit()
    }

    pub fn offset(&self) -> usize {
        self.logical.offset()
    }

    pub fn topn_order(&self) -> &Order {
        self.logical.topn_order()
    }

    pub fn group_key(&self) -> &[usize] {
        self.logical.group_key()
    }

    pub fn with_ties(&self) -> bool {
        self.logical.with_ties()
    }
}

impl StreamNode for StreamGroupTopN {
    fn to_stream_prost_body(&self, state: &mut BuildFragmentGraphState) -> ProstStreamNode {
        use piestream_pb::stream_plan::*;
        let table = self
            .logical
            .infer_internal_table_catalog(self.vnode_col_idx)
            .with_id(state.gen_table_id_wrapped());
        let group_topn_node = GroupTopNNode {
            limit: self.limit() as u64,
            offset: self.offset() as u64,
            with_ties: self.with_ties(),
            group_key: self.group_key().iter().map(|idx| *idx as u32).collect(),
            table: Some(table.to_internal_table_prost()),
            order_by_len: self.topn_order().len() as u32,
        };

        ProstStreamNode::GroupTopN(group_topn_node)
    }
}

impl fmt::Display for StreamGroupTopN {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("StreamGroupTopN");
        let input = self.input();
        let input_schema = input.schema();
        builder.field(
            "order",
            &format!(
                "{}",
                OrderDisplay {
                    order: self.topn_order(),
                    input_schema
                }
            ),
        );
        builder
            .field("limit", &format_args!("{}", self.limit()))
            .field("offset", &format_args!("{}", self.offset()))
            .field("group_key", &format_args!("{:?}", self.group_key()));
        if self.with_ties() {
            builder.field("with_ties", &format_args!("true"));
        }
        builder.finish()
    }
}

impl_plan_tree_node_for_unary! { StreamGroupTopN }

impl PlanTreeNodeUnary for StreamGroupTopN {
    fn input(&self) -> PlanRef {
        self.logical.input()
    }

    fn clone_with_input(&self, input: PlanRef) -> Self {
        Self::new(self.logical.clone_with_input(input), self.vnode_col_idx)
    }
}
