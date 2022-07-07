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

use piestream_common::error::Result;
use piestream_pb::batch_plan::plan_node::NodeBody;
use piestream_pb::batch_plan::LimitNode;

use super::{LogicalLimit, PlanBase, PlanRef, PlanTreeNodeUnary, ToBatchProst, ToDistributedBatch};
use crate::optimizer::plan_node::ToLocalBatch;

/// `BatchLimit` implements [`super::LogicalLimit`] to fetch specified rows from input
#[derive(Debug, Clone)]
pub struct BatchLimit {
    pub base: PlanBase,
    logical: LogicalLimit,
}

impl BatchLimit {
    pub fn new(logical: LogicalLimit) -> Self {
        let ctx = logical.base.ctx.clone();
        let base = PlanBase::new_batch(
            ctx,
            logical.schema().clone(),
            logical.input().distribution().clone(),
            logical.input().order().clone(),
        );
        BatchLimit { base, logical }
    }
}

impl fmt::Display for BatchLimit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BatchLimit {{ limit: {limit}, offset: {offset} }}",
            limit = self.logical.limit,
            offset = self.logical.offset
        )
    }
}

impl PlanTreeNodeUnary for BatchLimit {
    fn input(&self) -> PlanRef {
        self.logical.input()
    }

    fn clone_with_input(&self, input: PlanRef) -> Self {
        Self::new(self.logical.clone_with_input(input))
    }
}
impl_plan_tree_node_for_unary! {BatchLimit}
impl ToDistributedBatch for BatchLimit {
    fn to_distributed(&self) -> Result<PlanRef> {
        let new_input = self.input().to_distributed()?;
        Ok(self.clone_with_input(new_input).into())
    }
}

impl ToBatchProst for BatchLimit {
    fn to_batch_prost_body(&self) -> NodeBody {
        NodeBody::Limit(LimitNode {
            limit: self.logical.limit() as u32,
            offset: self.logical.offset() as u32,
        })
    }
}

impl ToLocalBatch for BatchLimit {
    fn to_local(&self) -> Result<PlanRef> {
        let new_input = self.input().to_local()?;
        Ok(self.clone_with_input(new_input).into())
    }
}
