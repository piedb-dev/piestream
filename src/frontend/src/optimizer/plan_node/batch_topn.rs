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

use risingwave_common::error::Result;
use risingwave_pb::batch_plan::plan_node::NodeBody;
use risingwave_pb::batch_plan::TopNNode;

use super::{LogicalTopN, PlanBase, PlanRef, PlanTreeNodeUnary, ToBatchProst, ToDistributedBatch};
use crate::optimizer::plan_node::ToLocalBatch;
use crate::optimizer::property::{Distribution, Order};

/// `BatchTopN` implements [`super::LogicalTopN`] to find the top N elements with a heap
#[derive(Debug, Clone)]
pub struct BatchTopN {
    pub base: PlanBase,
    logical: LogicalTopN,
}

impl BatchTopN {
    pub fn new(logical: LogicalTopN) -> Self {
        let ctx = logical.base.ctx.clone();
        let base = PlanBase::new_batch(
            ctx,
            logical.schema().clone(),
            logical.input().distribution().clone(),
            // BatchTopN outputs data in the order of specified order
            logical.topn_order().clone(),
        );
        BatchTopN { base, logical }
    }
}

impl fmt::Display for BatchTopN {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BatchTopN {{ order: {}, limit: {}, offset: {} }}",
            self.logical.topn_order(),
            self.logical.limit(),
            self.logical.offset(),
        )
    }
}

impl PlanTreeNodeUnary for BatchTopN {
    fn input(&self) -> PlanRef {
        self.logical.input()
    }

    fn clone_with_input(&self, input: PlanRef) -> Self {
        Self::new(self.logical.clone_with_input(input))
    }
}

impl_plan_tree_node_for_unary! {BatchTopN}

impl ToDistributedBatch for BatchTopN {
    fn to_distributed(&self) -> Result<PlanRef> {
        let new_input = self
            .input()
            .to_distributed_with_required(Order::any(), &Distribution::Single)?;
        Ok(self.clone_with_input(new_input).into())
    }
}

impl ToBatchProst for BatchTopN {
    fn to_batch_prost_body(&self) -> NodeBody {
        let column_orders = self.logical.topn_order().to_protobuf(&self.base.schema);
        NodeBody::TopN(TopNNode {
            limit: self.logical.limit() as u32,
            offset: self.logical.offset() as u32,
            column_orders,
        })
    }
}

impl ToLocalBatch for BatchTopN {
    fn to_local(&self) -> Result<PlanRef> {
        let new_input = self.input().to_local()?;
        Ok(self.clone_with_input(new_input).into())
    }
}
