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

use itertools::{Either, Itertools};

use super::super::plan_node::*;
use super::{BoxedRule, Rule};
use crate::expr::{CorrelatedInputRef, Expr, ExprImpl, ExprRewriter, InputRef};
use crate::optimizer::PlanRef;
use crate::utils::Condition;

/// This rule is for pattern: Apply->Project->Filter.
///
/// To unnest, we just pull predicates contain correlated variables in Filter into Apply, and
/// convert it into corresponding type of Join.
pub struct PullUpCorrelatedPredicate {}
impl Rule for PullUpCorrelatedPredicate {
    fn apply(&self, plan: PlanRef) -> Option<PlanRef> {
        let apply = plan.as_logical_apply()?;
        let (apply_left, apply_right, apply_on, join_type, ..) = apply.clone().decompose();

        let project = apply_right.as_logical_project()?;
        let (mut proj_exprs, _) = project.clone().decompose();

        let input = project.input();
        let filter = input.as_logical_filter()?;

        let mut rewriter = Rewriter {
            input_refs: vec![],
            index: proj_exprs.len() + apply_left.schema().fields().len(),
        };
        // Split predicates in LogicalFilter into correlated expressions and uncorrelated
        // expressions.
        let (cor_exprs, uncor_exprs) =
            filter
                .predicate()
                .clone()
                .into_iter()
                .partition_map(|expr| {
                    if expr.has_correlated_input_ref() {
                        Either::Left(rewriter.rewrite_expr(expr))
                    } else {
                        Either::Right(expr)
                    }
                });
        // Append `InputRef`s in the predicate expression to be pulled to the project, so that they
        // are accessible by the expression after it is pulled.
        proj_exprs.extend(
            rewriter
                .input_refs
                .drain(..)
                .map(|input_ref| input_ref.into()),
        );

        let filter = LogicalFilter::create(
            filter.input(),
            Condition {
                conjunctions: uncor_exprs,
            },
        );

        let project = LogicalProject::new(filter, proj_exprs);

        // Merge these expressions with LogicalApply into LogicalJoin.
        let on = apply_on.and(Condition {
            conjunctions: cor_exprs,
        });
        Some(LogicalJoin::new(apply_left, project.into(), join_type, on).into())
    }
}

/// Rewrites a pulled predicate expression. It is pulled from the right of the apply to the `on`
/// clause.
///
/// Rewrites `correlated_input_ref` (referencing left side) to `input_ref` and shifting `input_ref`
/// (referencing right side).
///
/// Also collects all `InputRef`s, which will be added to the project, so that they are accessible
/// by the expression after it is pulled.
struct Rewriter {
    // All uncorrelated `InputRef`s in the expression.
    pub input_refs: Vec<InputRef>,

    pub index: usize,
}

impl ExprRewriter for Rewriter {
    fn rewrite_correlated_input_ref(
        &mut self,
        correlated_input_ref: CorrelatedInputRef,
    ) -> ExprImpl {
        // Convert correlated_input_ref to input_ref.
        // TODO: use LiftCorrelatedInputRef here.
        InputRef::new(
            correlated_input_ref.index(),
            correlated_input_ref.return_type(),
        )
        .into()
    }

    fn rewrite_input_ref(&mut self, input_ref: InputRef) -> ExprImpl {
        let data_type = input_ref.return_type();

        // It will be appended to exprs in LogicalProject, so its index remain the same.
        self.input_refs.push(input_ref);

        // Rewrite input_ref's index to its new location.
        let input_ref = InputRef::new(self.index, data_type);
        self.index += 1;
        input_ref.into()
    }
}

impl PullUpCorrelatedPredicate {
    pub fn create() -> BoxedRule {
        Box::new(PullUpCorrelatedPredicate {})
    }
}
