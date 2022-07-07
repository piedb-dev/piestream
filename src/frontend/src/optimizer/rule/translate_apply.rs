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

use std::collections::HashMap;

use itertools::Itertools;
use risingwave_common::types::DataType;
use risingwave_pb::plan_common::JoinType;

use super::{BoxedRule, Rule};
use crate::expr::{Expr, ExprImpl, ExprRewriter, ExprType, FunctionCall, InputRef};
use crate::optimizer::plan_node::{
    LogicalAgg, LogicalApply, LogicalJoin, LogicalProject, LogicalScan, PlanTreeNodeBinary,
    PlanTreeNodeUnary,
};
use crate::optimizer::PlanRef;
use crate::utils::{ColIndexMapping, Condition};

/// Translate `LogicalApply` into `LogicalJoin` and `LogicalApply`, and rewrite
/// `LogicalApply`'s left.
pub struct TranslateApply {}
impl Rule for TranslateApply {
    fn apply(&self, plan: PlanRef) -> Option<PlanRef> {
        let apply = plan.as_logical_apply()?;
        let (left, right, on, join_type, correlated_indices) = apply.clone().decompose();
        let apply_left_len = left.schema().len();
        let correlated_indices_len = correlated_indices.len();

        let mut index_mapping =
            ColIndexMapping::with_target_size(vec![None; apply_left_len], apply_left_len);
        let mut data_types = HashMap::new();
        let mut index = 0;

        let rewritten_left = Self::rewrite(
            &left,
            correlated_indices.clone(),
            0,
            &mut index_mapping,
            &mut data_types,
            &mut index,
        )?;

        // This `LogicalProject` is used to make sure that after `LogicalApply`'s left was
        // rewritten, the new index of `correlated_index` is always at its position in
        // `correlated_indices`.
        let exprs = correlated_indices
            .clone()
            .into_iter()
            .enumerate()
            .map(|(i, correlated_index)| {
                let index = index_mapping.map(correlated_index);
                let data_type = rewritten_left.schema().fields()[index].data_type.clone();
                index_mapping.put(correlated_index, Some(i));
                InputRef::new(index, data_type).into()
            })
            .collect();
        let project = LogicalProject::create(rewritten_left, exprs);

        let distinct = LogicalAgg::new(vec![], (0..project.schema().len()).collect_vec(), project);

        let eq_predicates = correlated_indices
            .clone()
            .into_iter()
            .enumerate()
            .map(|(i, correlated_index)| {
                assert_eq!(i, index_mapping.map(correlated_index));
                let shifted_index = i + apply_left_len;
                let data_type = data_types.get(&correlated_index).unwrap().clone();
                let left = InputRef::new(correlated_index, data_type.clone());
                let right = InputRef::new(shifted_index, data_type);
                FunctionCall::new_unchecked(
                    ExprType::Equal,
                    vec![left.into(), right.into()],
                    DataType::Boolean,
                )
                .into()
            })
            .collect::<Vec<ExprImpl>>();
        let on = Self::rewrite_on(on, correlated_indices_len, apply_left_len).and(Condition {
            conjunctions: eq_predicates,
        });

        let new_apply = LogicalApply::create(
            distinct.into(),
            right,
            JoinType::Inner,
            Condition::true_cond(),
            correlated_indices,
        );
        let new_join = LogicalJoin::new(left, new_apply, join_type, on);

        let new_node = if new_join.join_type() != JoinType::LeftSemi {
            // `new_join`'s shcema is different from original apply's schema, so `LogicalProject` is
            // used to ensure they are the same.
            let mut exprs: Vec<ExprImpl> = vec![];
            new_join
                .schema()
                .data_types()
                .into_iter()
                .enumerate()
                .for_each(|(index, data_type)| {
                    if index < apply_left_len || index >= apply_left_len + correlated_indices_len {
                        exprs.push(InputRef::new(index, data_type).into());
                    }
                });
            LogicalProject::create(new_join.into(), exprs)
        } else {
            new_join.into()
        };
        Some(new_node)
    }
}

impl TranslateApply {
    pub fn create() -> BoxedRule {
        Box::new(TranslateApply {})
    }

    /// Rewrite `LogicalApply`'s left according to `correlated_indices`.
    ///
    /// Assumption: only `LogicalJoin`, `LogicalScan`, `LogicalProject` and `LogicalFilter` are in
    /// the left.
    fn rewrite(
        plan: &PlanRef,
        correlated_indices: Vec<usize>,
        offset: usize,
        index_mapping: &mut ColIndexMapping,
        data_types: &mut HashMap<usize, DataType>,
        index: &mut usize,
    ) -> Option<PlanRef> {
        if let Some(join) = plan.as_logical_join() {
            Self::rewrite_join(
                join,
                correlated_indices,
                offset,
                index_mapping,
                data_types,
                index,
            )
        } else if let Some(scan) = plan.as_logical_scan() {
            Self::rewrite_scan(
                scan,
                correlated_indices,
                offset,
                index_mapping,
                data_types,
                index,
            )
        } else if let Some(filter) = plan.as_logical_filter() {
            Self::rewrite(
                &filter.input(),
                correlated_indices,
                offset,
                index_mapping,
                data_types,
                index,
            )
        } else if let Some(project) = plan.as_logical_project() {
            Self::rewrite(
                &project.input(),
                correlated_indices,
                offset,
                index_mapping,
                data_types,
                index,
            )
        } else {
            // TODO: better to return an error.
            None
        }
    }

    fn rewrite_join(
        join: &LogicalJoin,
        required_col_idx: Vec<usize>,
        mut offset: usize,
        index_mapping: &mut ColIndexMapping,
        data_types: &mut HashMap<usize, DataType>,
        index: &mut usize,
    ) -> Option<PlanRef> {
        // TODO: Do we need to take the `on` into account?
        let left_len = join.left().schema().len();
        let (left_idxs, right_idxs): (Vec<_>, Vec<_>) = required_col_idx
            .into_iter()
            .partition(|idx| *idx < left_len);
        let mut rewrite =
            |plan: PlanRef, mut indices: Vec<usize>, is_right: bool| -> Option<PlanRef> {
                if is_right {
                    indices.iter_mut().for_each(|index| *index -= left_len);
                    offset += left_len;
                }
                if let Some(join) = plan.as_logical_join() {
                    Self::rewrite_join(join, indices, offset, index_mapping, data_types, index)
                } else if let Some(scan) = plan.as_logical_scan() {
                    Self::rewrite_scan(scan, indices, offset, index_mapping, data_types, index)
                } else {
                    None
                }
            };
        match (left_idxs.is_empty(), right_idxs.is_empty()) {
            (true, false) => rewrite(join.right(), right_idxs, true),
            (false, true) => rewrite(join.left(), left_idxs, false),
            (false, false) => {
                let left = rewrite(join.left(), left_idxs, false)?;
                let right = rewrite(join.right(), right_idxs, true)?;
                let new_join =
                    LogicalJoin::new(left, right, join.join_type(), Condition::true_cond());
                Some(new_join.into())
            }
            _ => None,
        }
    }

    fn rewrite_scan(
        scan: &LogicalScan,
        required_col_idx: Vec<usize>,
        offset: usize,
        index_mapping: &mut ColIndexMapping,
        data_types: &mut HashMap<usize, DataType>,
        index: &mut usize,
    ) -> Option<PlanRef> {
        for i in &required_col_idx {
            let correlated_index = *i + offset;
            index_mapping.put(correlated_index, Some(*index));
            data_types.insert(
                correlated_index,
                scan.schema().fields()[*i].data_type.clone(),
            );
            *index += 1;
        }

        Some(scan.clone_with_output_indices(required_col_idx).into())
    }

    fn rewrite_on(on: Condition, offset: usize, apply_left_len: usize) -> Condition {
        struct Rewriter {
            offset: usize,
            apply_left_len: usize,
        }
        impl ExprRewriter for Rewriter {
            fn rewrite_input_ref(&mut self, input_ref: InputRef) -> ExprImpl {
                let index = input_ref.index();
                if index >= self.apply_left_len {
                    InputRef::new(index + self.offset, input_ref.return_type()).into()
                } else {
                    input_ref.into()
                }
            }
        }
        let mut rewriter = Rewriter {
            offset,
            apply_left_len,
        };
        Condition {
            conjunctions: on
                .into_iter()
                .map(|expr| rewriter.rewrite_expr(expr))
                .collect_vec(),
        }
    }
}
