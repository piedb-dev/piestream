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

use itertools::Itertools;
use risingwave_common::error::Result;
use risingwave_pb::batch_plan::plan_node::NodeBody;
use risingwave_pb::batch_plan::HashAggNode;

use super::logical_agg::PlanAggCall;
use super::{LogicalAgg, PlanBase, PlanRef, PlanTreeNodeUnary, ToBatchProst, ToDistributedBatch};
use crate::expr::InputRefDisplay;
use crate::optimizer::plan_node::ToLocalBatch;
use crate::optimizer::property::{Distribution, Order};

#[derive(Debug, Clone)]
pub struct BatchHashAgg {
    pub base: PlanBase,
    logical: LogicalAgg,
}

impl BatchHashAgg {
    pub fn new(logical: LogicalAgg) -> Self {
        let ctx = logical.base.ctx.clone();
        let input = logical.input();
        let input_dist = input.distribution();
        let dist = match input_dist {
            Distribution::Any => Distribution::Any,
            Distribution::Single => Distribution::Single,
            Distribution::Broadcast => panic!(),
            Distribution::AnyShard => Distribution::AnyShard,
            Distribution::HashShard(_) => logical
                .i2o_col_mapping()
                .rewrite_provided_distribution(input_dist),
        };
        let base = PlanBase::new_batch(ctx, logical.schema().clone(), dist, Order::any().clone());
        BatchHashAgg { base, logical }
    }

    pub fn agg_calls(&self) -> &[PlanAggCall] {
        self.logical.agg_calls()
    }

    pub fn group_keys(&self) -> &[usize] {
        self.logical.group_keys()
    }
}

impl fmt::Display for BatchHashAgg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BatchHashAgg")
            .field(
                "group_keys",
                &self
                    .group_keys()
                    .iter()
                    .copied()
                    .map(InputRefDisplay)
                    .collect_vec(),
            )
            .field("aggs", &self.agg_calls())
            .finish()
    }
}

impl PlanTreeNodeUnary for BatchHashAgg {
    fn input(&self) -> PlanRef {
        self.logical.input()
    }

    fn clone_with_input(&self, input: PlanRef) -> Self {
        Self::new(self.logical.clone_with_input(input))
    }
}
impl_plan_tree_node_for_unary! { BatchHashAgg }
impl ToDistributedBatch for BatchHashAgg {
    fn to_distributed(&self) -> Result<PlanRef> {
        let new_input = self.input().to_distributed_with_required(
            Order::any(),
            &Distribution::HashShard(self.group_keys().to_vec()),
        )?;
        Ok(self.clone_with_input(new_input).into())
    }
}

impl ToBatchProst for BatchHashAgg {
    fn to_batch_prost_body(&self) -> NodeBody {
        NodeBody::HashAgg(HashAggNode {
            agg_calls: self
                .agg_calls()
                .iter()
                .map(PlanAggCall::to_protobuf)
                .collect(),
            group_keys: self
                .group_keys()
                .iter()
                .clone()
                .map(|index| *index as u32)
                .collect(),
        })
    }
}

impl ToLocalBatch for BatchHashAgg {
    fn to_local(&self) -> Result<PlanRef> {
        let new_input = self.input().to_local_with_order_required(Order::any())?;
        Ok(self.clone_with_input(new_input).into())
    }
}
