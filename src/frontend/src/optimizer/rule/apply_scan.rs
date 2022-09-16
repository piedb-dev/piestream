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

use std::collections::HashMap;

use piestream_common::types::DataType;
use piestream_pb::plan_common::JoinType;

use super::{BoxedRule, Rule};
use crate::expr::{Expr, ExprImpl, ExprType, FunctionCall, InputRef};
use crate::optimizer::plan_node::{LogicalJoin, LogicalProject};
use crate::optimizer::PlanRef;

pub struct ApplyScan {}
impl Rule for ApplyScan {
    fn apply(&self, plan: PlanRef) -> Option<PlanRef> {
        let apply = plan.as_logical_apply()?;
        let (left, right, on, join_type, correlated_indices) = apply.clone().decompose();
        let apply_left_len = left.schema().len();
        assert_eq!(join_type, JoinType::Inner);
        // TODO: Push `LogicalApply` down `LogicalJoin`.
        if let (None, None) = (right.as_logical_scan(), right.as_logical_join()) {
            return None;
        }

        // Record the mapping from `CorrelatedInputRef`'s index to `InputRef`'s index.
        // We currently can remove DAG only if ALL the `CorrelatedInputRef` are equal joined to
        // `InputRef`.
        // TODO: Do some transformation for IN, and try to remove DAG for it.
        let mut column_mapping = HashMap::new();
        on.conjunctions.iter().for_each(|expr| {
            if let ExprImpl::FunctionCall(func_call) = expr {
                if let Some((left, right, data_type)) = Self::check(func_call, apply_left_len) {
                    column_mapping.insert(left, (right, data_type));
                }
            }
        });
        if column_mapping.len() == apply_left_len {
            // Remove DAG.

            // Replace `LogicalApply` with `LogicalProject` and insert the `InputRef`s which is
            // equal to `CorrelatedInputRef` at the beginning of `LogicalProject`.
            // See the fourth section of Unnesting Arbitrary Queries for how to do the optimization.
            let mut exprs: Vec<ExprImpl> = correlated_indices
                .into_iter()
                .map(|correlated_index| {
                    let (col_index, data_type) = column_mapping.get(&correlated_index).unwrap();
                    InputRef::new(*col_index - apply_left_len, data_type.clone()).into()
                })
                .collect();
            exprs.extend(
                right
                    .schema()
                    .data_types()
                    .into_iter()
                    .enumerate()
                    .map(|(index, data_type)| InputRef::new(index, data_type).into()),
            );
            let project = LogicalProject::create(right, exprs);
            // TODO: add LogicalFilter here.
            Some(project)
        } else {
            let join = LogicalJoin::new(left, right, join_type, on);
            Some(join.into())
        }
    }
}

impl ApplyScan {
    pub fn create() -> BoxedRule {
        Box::new(ApplyScan {})
    }

    /// Check whether the `func_call` is like v1 = v2, in which v1 and v2 belong respectively to
    /// `LogicalApply`'s left and right.
    fn check(func_call: &FunctionCall, apply_left_len: usize) -> Option<(usize, usize, DataType)> {
        let inputs = func_call.inputs();
        if func_call.get_expr_type() == ExprType::Equal && inputs.len() == 2 {
            let left = &inputs[0];
            let right = &inputs[1];
            match (left, right) {
                (ExprImpl::InputRef(left), ExprImpl::InputRef(right)) => {
                    let left_type = left.return_type();
                    let left = left.index();
                    let right_type = right.return_type();
                    let right = right.index();
                    if left < apply_left_len && right >= apply_left_len {
                        Some((left, right, right_type))
                    } else if left >= apply_left_len && right < apply_left_len {
                        Some((right, left, left_type))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
