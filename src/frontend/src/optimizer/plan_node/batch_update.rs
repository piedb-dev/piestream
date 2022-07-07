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

use piestream_common::error::Result;
use piestream_pb::batch_plan::plan_node::NodeBody;
use piestream_pb::batch_plan::UpdateNode;
use piestream_pb::plan_common::TableRefId;

use super::{
    LogicalUpdate, PlanBase, PlanRef, PlanTreeNodeUnary, ToBatchProst, ToDistributedBatch,
};
use crate::expr::Expr;
use crate::optimizer::plan_node::ToLocalBatch;
use crate::optimizer::property::{Distribution, Order};

/// `BatchUpdate` implements [`LogicalUpdate`]
#[derive(Debug, Clone)]
pub struct BatchUpdate {
    pub base: PlanBase,
    logical: LogicalUpdate,
}

impl BatchUpdate {
    pub fn new(logical: LogicalUpdate) -> Self {
        let ctx = logical.base.ctx.clone();
        let base = PlanBase::new_batch(
            ctx,
            logical.schema().clone(),
            Distribution::Single,
            Order::any(),
        );
        Self { base, logical }
    }
}

impl fmt::Display for BatchUpdate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.logical.fmt_with_name(f, "BatchUpdate")
    }
}

impl PlanTreeNodeUnary for BatchUpdate {
    fn input(&self) -> PlanRef {
        self.logical.input()
    }

    fn clone_with_input(&self, input: PlanRef) -> Self {
        Self::new(self.logical.clone_with_input(input))
    }
}

impl_plan_tree_node_for_unary! { BatchUpdate }

impl ToDistributedBatch for BatchUpdate {
    fn to_distributed(&self) -> Result<PlanRef> {
        let new_input = self.input().to_distributed()?;
        Ok(self.clone_with_input(new_input).into())
    }
}

impl ToBatchProst for BatchUpdate {
    fn to_batch_prost_body(&self) -> NodeBody {
        let table_id = TableRefId {
            table_id: self.logical.source_id().table_id() as i32,
            ..Default::default()
        };
        let exprs = self
            .logical
            .exprs()
            .iter()
            .map(Expr::to_expr_proto)
            .collect();

        NodeBody::Update(UpdateNode {
            table_source_ref_id: Some(table_id),
            exprs,
        })
    }
}

impl ToLocalBatch for BatchUpdate {
    fn to_local(&self) -> Result<PlanRef> {
        unreachable!()
    }
}
