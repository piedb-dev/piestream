// Copyright 2022 Piedb Data
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

//! Define all [`Rule`]

use super::PlanRef;

/// A one-to-one transform for the [`PlanNode`](super::plan_node::PlanNode), every [`Rule`] should
/// downcast and check if the node matches the rule.
pub trait Rule: Send + Sync + Description {
    /// return err(()) if not match
    fn apply(&self, plan: PlanRef) -> Option<PlanRef>;
}

pub trait Description {
    fn description(&self) -> &str;
}

pub(super) type BoxedRule = Box<dyn Rule>;

mod project_join;
pub use project_join::*;
mod project_elim;
pub use project_elim::*;
mod project_merge;
pub use project_merge::*;
mod pull_up_correlated_predicate;
pub use pull_up_correlated_predicate::*;
mod index_delta_join;
pub use index_delta_join::*;
mod reorder_multijoin;
pub use reorder_multijoin::*;
mod apply_agg;
pub use apply_agg::*;
mod apply_filter;
pub use apply_filter::*;
mod apply_proj;
pub use apply_proj::*;
mod apply_scan;
pub use apply_scan::*;
mod translate_apply;
pub use translate_apply::*;
mod merge_multijoin;
pub use merge_multijoin::*;
mod max_one_row_elim;
pub use max_one_row_elim::*;
mod apply_join;
pub use apply_join::*;
mod apply_to_join;
pub use apply_to_join::*;
mod distinct_agg;
pub use distinct_agg::*;
mod index_selection;
pub use index_selection::*;
mod push_calculation_of_join;
pub use push_calculation_of_join::*;
mod join_commute;
mod over_agg_to_topn;
pub use join_commute::*;
pub use over_agg_to_topn::*;

#[macro_export]
macro_rules! for_all_rules {
    ($macro:ident) => {
        $macro! {
             {ApplyAggRule}
            ,{ApplyFilterRule}
            ,{ApplyProjRule}
            ,{ApplyScanRule}
            ,{ApplyJoinRule}
            ,{ApplyToJoinRule}
            ,{MaxOneRowEliminateRule}
            ,{DistinctAggRule}
            ,{IndexDeltaJoinRule}
            ,{MergeMultiJoinRule}
            ,{ProjectEliminateRule}
            ,{ProjectJoinRule}
            ,{ProjectMergeRule}
            ,{PullUpCorrelatedPredicateRule}
            ,{ReorderMultiJoinRule}
            ,{TranslateApplyRule}
            ,{PushCalculationOfJoinRule}
            ,{IndexSelectionRule}
            ,{OverAggToTopNRule}
            ,{JoinCommuteRule}
        }
    };
}

macro_rules! impl_description {
    ($( { $name:ident }),*) => {
        paste::paste!{
            $(impl Description for [<$name>] {
                fn description(&self) -> &str {
                    stringify!([<$name>])
                }
            })*
        }
    }
}

for_all_rules! {impl_description}
