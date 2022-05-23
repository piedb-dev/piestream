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

use std::fmt;

use risingwave_pb::expr::InputRefExpr;
use risingwave_pb::plan_common::ColumnOrder;
use risingwave_pb::stream_plan::stream_node::NodeBody as ProstStreamNode;

use super::{LogicalTopN, PlanBase, PlanRef, PlanTreeNodeUnary, ToStreamProst};
use crate::optimizer::property::Distribution;

/// `StreamTopN` implements [`super::LogicalTopN`] to find the top N elements with a heap
#[derive(Debug, Clone)]
pub struct StreamTopN {
    pub base: PlanBase,
    logical: LogicalTopN,
}

impl StreamTopN {
    pub fn new(logical: LogicalTopN) -> Self {
        let ctx = logical.base.ctx.clone();
        let dist = match logical.input().distribution() {
            Distribution::Any => Distribution::Any,
            Distribution::Single => Distribution::Single,
            _ => panic!(),
        };

        let base = PlanBase::new_stream(
            ctx,
            logical.schema().clone(),
            logical.input().pk_indices().to_vec(),
            dist,
            false,
        );
        StreamTopN { base, logical }
    }
}

impl fmt::Display for StreamTopN {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "StreamTopN {{ order: {}, limit: {}, offset: {} }}",
            self.logical.topn_order(),
            self.logical.limit(),
            self.logical.offset(),
        )
    }
}

impl PlanTreeNodeUnary for StreamTopN {
    fn input(&self) -> PlanRef {
        self.logical.input()
    }

    fn clone_with_input(&self, input: PlanRef) -> Self {
        Self::new(self.logical.clone_with_input(input))
    }
}

impl_plan_tree_node_for_unary! { StreamTopN }

impl ToStreamProst for StreamTopN {
    fn to_stream_prost_body(&self) -> ProstStreamNode {
        use risingwave_pb::stream_plan::*;
        let column_orders = self
            .logical
            .topn_order()
            .field_order
            .iter()
            .map(|f| ColumnOrder {
                order_type: f.direct.to_protobuf() as i32,
                input_ref: Some(InputRefExpr {
                    column_idx: f.index as i32,
                }),
                return_type: Some(self.input().schema()[f.index].data_type().to_protobuf()),
            })
            .collect();
        ProstStreamNode::TopN(TopNNode {
            column_orders,
            limit: self.logical.limit() as u64,
            offset: self.logical.offset() as u64,
            distribution_keys: vec![], // TODO: seems unnecessary
        })
    }
}
