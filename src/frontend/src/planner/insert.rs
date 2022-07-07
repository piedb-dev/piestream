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

use fixedbitset::FixedBitSet;
use piestream_common::error::Result;

use crate::binder::BoundInsert;
use crate::optimizer::plan_node::{LogicalInsert, LogicalProject, PlanRef};
use crate::optimizer::property::{Order, RequiredDist};
use crate::optimizer::PlanRoot;
use crate::planner::Planner;

impl Planner {
    pub(super) fn plan_insert(&mut self, insert: BoundInsert) -> Result<PlanRoot> {
        let mut input = self.plan_query(insert.source)?.as_subplan();
        if !insert.cast_exprs.is_empty() {
            input = LogicalProject::create(input, insert.cast_exprs);
        }
        // `columns` not used by backend yet.
        let plan: PlanRef = LogicalInsert::create(
            input,
            insert.table_source.name,
            insert.table_source.source_id,
        )?
        .into();
        // For insert, frontend will only schedule one task so do not need this to be single.
        let dist = RequiredDist::Any;
        let mut out_fields = FixedBitSet::with_capacity(plan.schema().len());
        out_fields.insert_range(..);
        let out_names = plan.schema().names();
        let root = PlanRoot::new(plan, dist, Order::any(), out_fields, out_names);
        Ok(root)
    }
}
