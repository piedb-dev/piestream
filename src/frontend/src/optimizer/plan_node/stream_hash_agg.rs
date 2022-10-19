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

use super::generic::PlanAggCall;
use super::{LogicalAgg, PlanBase, PlanRef, PlanTreeNodeUnary, StreamNode};
use crate::optimizer::property::Distribution;
use crate::stream_fragmenter::BuildFragmentGraphState;

#[derive(Debug, Clone)]
pub struct StreamHashAgg {
    pub base: PlanBase,
    /// an optional column index which is the vnode of each row computed by the input's consistent
    /// hash distribution
    vnode_col_idx: Option<usize>,
    logical: LogicalAgg,
}

impl StreamHashAgg {
    pub fn new(logical: LogicalAgg, vnode_col_idx: Option<usize>) -> Self {
        let ctx = logical.base.ctx.clone();
        let pk_indices = logical.base.logical_pk.to_vec();
        let input = logical.input();
        let input_dist = input.distribution();
        let dist = match input_dist {
            Distribution::HashShard(_) => logical
                .i2o_col_mapping()
                .rewrite_provided_distribution(input_dist),
            d => d.clone(),
        };
        // Hash agg executor might change the append-only behavior of the stream.
        let base = PlanBase::new_stream(
            ctx,
            logical.schema().clone(),
            pk_indices,
            logical.functional_dependency().clone(),
            dist,
            false,
        );
        StreamHashAgg {
            base,
            vnode_col_idx,
            logical,
        }
    }

    pub fn agg_calls(&self) -> &[PlanAggCall] {
        self.logical.agg_calls()
    }

    pub fn group_key(&self) -> &[usize] {
        self.logical.group_key()
    }
}

impl fmt::Display for StreamHashAgg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.input().append_only() {
            self.logical.fmt_with_name(f, "StreamAppendOnlyHashAgg")
        } else {
            self.logical.fmt_with_name(f, "StreamHashAgg")
        }
    }
}

impl PlanTreeNodeUnary for StreamHashAgg {
    fn input(&self) -> PlanRef {
        self.logical.input()
    }

    fn clone_with_input(&self, input: PlanRef) -> Self {
        Self::new(self.logical.clone_with_input(input), self.vnode_col_idx)
    }
}
impl_plan_tree_node_for_unary! { StreamHashAgg }

impl StreamNode for StreamHashAgg {
    fn to_stream_prost_body(&self, state: &mut BuildFragmentGraphState) -> ProstStreamNode {
        use piestream_pb::stream_plan::*;
        let result_table = self.logical.infer_result_table(self.vnode_col_idx);
        let agg_states = self.logical.infer_stream_agg_state(self.vnode_col_idx);

        ProstStreamNode::HashAgg(HashAggNode {
            group_key: self.group_key().iter().map(|idx| *idx as u32).collect(),
            agg_calls: self
                .agg_calls()
                .iter()
                .map(PlanAggCall::to_protobuf)
                .collect(),

            is_append_only: self.input().append_only(),
            agg_call_states: agg_states
                .into_iter()
                .map(|s| s.into_prost(state))
                .collect(),
            result_table: Some(
                result_table
                    .with_id(state.gen_table_id_wrapped())
                    .to_internal_table_prost(),
            ),
        })
    }
}
