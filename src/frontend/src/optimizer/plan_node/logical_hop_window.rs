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

use fixedbitset::FixedBitSet;
use itertools::Itertools;
use piestream_common::catalog::{Field, Schema};
use piestream_common::error::Result;
use piestream_common::types::{DataType, IntervalUnit};

use super::{
    gen_filter_and_pushdown, generic, BatchHopWindow, ColPrunable, PlanBase, PlanRef,
    PlanTreeNodeUnary, PredicatePushdown, StreamHopWindow, ToBatch, ToStream,
};
use crate::expr::InputRef;
use crate::optimizer::property::Order;
use crate::utils::{ColIndexMapping, Condition};

/// `LogicalHopWindow` implements Hop Table Function.
#[derive(Debug, Clone)]
pub struct LogicalHopWindow {
    pub base: PlanBase,
    pub(super) core: generic::HopWindow<PlanRef>,
}

impl LogicalHopWindow {
    fn new(
        input: PlanRef,
        time_col: InputRef,
        window_slide: IntervalUnit,
        window_size: IntervalUnit,
        output_indices: Option<Vec<usize>>,
    ) -> Self {
        // if output_indices is not specified, use default output_indices
        let output_indices = output_indices
            .unwrap_or_else(|| (0..input.schema().len() + 2).into_iter().collect_vec());
        let ctx = input.ctx();
        let output_type = DataType::window_of(&time_col.data_type).unwrap();
        let original_schema: Schema = input
            .schema()
            .clone()
            .into_fields()
            .into_iter()
            .chain([
                Field::with_name(output_type.clone(), "window_start"),
                Field::with_name(output_type, "window_end"),
            ])
            .collect();
        let actual_schema: Schema = output_indices
            .iter()
            .map(|&idx| original_schema[idx].clone())
            .collect();
        let window_start_index = output_indices
            .iter()
            .position(|&idx| idx == input.schema().len());
        let window_end_index = output_indices
            .iter()
            .position(|&idx| idx == input.schema().len() + 1);
        let pk_indices = {
            if window_start_index.is_none() && window_end_index.is_none() {
                None
            } else {
                let mut pk = input
                    .logical_pk()
                    .iter()
                    .filter_map(|&pk_idx| output_indices.iter().position(|&idx| idx == pk_idx))
                    .collect_vec();
                if let Some(start_idx) = window_start_index {
                    pk.push(start_idx);
                };
                if let Some(end_idx) = window_end_index {
                    pk.push(end_idx);
                };
                Some(pk)
            }
        };
        let functional_dependency = {
            let mut fd_set =
                ColIndexMapping::identity_or_none(input.schema().len(), original_schema.len())
                    .composite(&ColIndexMapping::with_remaining_columns(
                        &output_indices,
                        original_schema.len(),
                    ))
                    .rewrite_functional_dependency_set(input.functional_dependency().clone());
            if let Some(start_idx) = window_start_index && let Some(end_idx) = window_end_index {
                fd_set.add_functional_dependency_by_column_indices(&[start_idx], &[end_idx]);
                fd_set.add_functional_dependency_by_column_indices(&[end_idx], &[start_idx]);
            }
            fd_set
        };
        // NOTE(st1page): add join keys in the pk_indices a work around before we really have stream
        // key.
        // let pk_indices = match pk_indices {
        //     Some(pk_indices) if functional_dependency.is_key(&pk_indices) => {
        //         functional_dependency.minimize_key(&pk_indices)
        //     }
        //     _ => pk_indices.unwrap_or_default(),
        // };
        let base = PlanBase::new_logical(
            ctx,
            actual_schema,
            pk_indices.unwrap_or_default(),
            functional_dependency,
        );
        let core = generic::HopWindow {
            input,
            time_col,
            window_slide,
            window_size,
            output_indices,
        };
        LogicalHopWindow { base, core }
    }

    pub fn into_parts(self) -> (PlanRef, InputRef, IntervalUnit, IntervalUnit, Vec<usize>) {
        self.core.into_parts()
    }

    /// the function will check if the cond is bool expression
    pub fn create(
        input: PlanRef,
        time_col: InputRef,
        window_slide: IntervalUnit,
        window_size: IntervalUnit,
    ) -> PlanRef {
        Self::new(input, time_col, window_slide, window_size, None).into()
    }

    fn window_start_col_idx(&self) -> usize {
        self.input().schema().len()
    }

    fn window_end_col_idx(&self) -> usize {
        self.window_start_col_idx() + 1
    }

    pub fn o2i_col_mapping(&self) -> ColIndexMapping {
        self.output2internal_col_mapping()
            .composite(&self.internal2input_col_mapping())
    }

    pub fn i2o_col_mapping(&self) -> ColIndexMapping {
        self.input2internal_col_mapping()
            .composite(&self.internal2output_col_mapping())
    }

    fn internal_column_num(&self) -> usize {
        self.window_start_col_idx() + 2
    }

    fn output2internal_col_mapping(&self) -> ColIndexMapping {
        self.internal2output_col_mapping().inverse()
    }

    fn internal2output_col_mapping(&self) -> ColIndexMapping {
        ColIndexMapping::with_remaining_columns(
            &self.core.output_indices,
            self.internal_column_num(),
        )
    }

    fn input2internal_col_mapping(&self) -> ColIndexMapping {
        ColIndexMapping::identity_or_none(self.window_start_col_idx(), self.internal_column_num())
    }

    fn internal2input_col_mapping(&self) -> ColIndexMapping {
        ColIndexMapping::identity_or_none(self.internal_column_num(), self.window_start_col_idx())
    }

    fn clone_with_output_indices(&self, output_indices: Vec<usize>) -> Self {
        Self::new(
            self.input().clone(),
            self.core.time_col.clone(),
            self.core.window_slide,
            self.core.window_size,
            Some(output_indices),
        )
    }

    pub fn fmt_with_name(&self, f: &mut fmt::Formatter<'_>, name: &str) -> fmt::Result {
        self.core.fmt_with_name(f, name)
    }

    /// Map the order of the input to use the updated indices
    pub fn get_out_column_index_order(&self) -> Order {
        self.i2o_col_mapping()
            .rewrite_provided_order(self.input().order())
    }
}

impl PlanTreeNodeUnary for LogicalHopWindow {
    fn input(&self) -> PlanRef {
        self.core.input.clone()
    }

    fn clone_with_input(&self, input: PlanRef) -> Self {
        Self::new(
            input,
            self.core.time_col.clone(),
            self.core.window_slide,
            self.core.window_size,
            Some(self.core.output_indices.clone()),
        )
    }

    #[must_use]
    fn rewrite_with_input(
        &self,
        input: PlanRef,
        input_col_change: ColIndexMapping,
    ) -> (Self, ColIndexMapping) {
        let mut time_col = self.core.time_col.clone();
        time_col.index = input_col_change.map(time_col.index);
        let mut columns_to_be_kept = Vec::new();
        let new_output_indices = self
            .core
            .output_indices
            .iter()
            .enumerate()
            .filter_map(|(i, &idx)| match input_col_change.try_map(idx) {
                Some(new_idx) => {
                    columns_to_be_kept.push(i);
                    Some(new_idx)
                }
                None => {
                    if idx == self.window_start_col_idx() {
                        columns_to_be_kept.push(i);
                        Some(input.schema().len())
                    } else if idx == self.window_end_col_idx() {
                        columns_to_be_kept.push(i);
                        Some(input.schema().len() + 1)
                    } else {
                        None
                    }
                }
            })
            .collect_vec();
        let new_hop = Self::new(
            input.clone(),
            time_col,
            self.core.window_slide,
            self.core.window_size,
            Some(new_output_indices),
        );
        (
            new_hop,
            ColIndexMapping::with_remaining_columns(
                &columns_to_be_kept,
                self.core.output_indices.len(),
            ),
        )
    }
}

impl_plan_tree_node_for_unary! {LogicalHopWindow}

impl fmt::Display for LogicalHopWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_name(f, "LogicalHopWindow")
    }
}

impl ColPrunable for LogicalHopWindow {
    fn prune_col(&self, required_cols: &[usize]) -> PlanRef {
        let o2i = self.o2i_col_mapping();
        let input_required_cols = {
            let mut tmp = FixedBitSet::with_capacity(self.schema().len());
            tmp.extend(required_cols.iter().copied());
            tmp = o2i.rewrite_bitset(&tmp);
            // LogicalHopWindow should keep all required cols from upstream,
            // as well as its own time_col.
            tmp.put(self.core.time_col.index());
            tmp.ones().collect_vec()
        };
        let input = self.input().prune_col(&input_required_cols);
        let input_change = ColIndexMapping::with_remaining_columns(
            &input_required_cols,
            self.input().schema().len(),
        );
        let (new_hop, _) = self.rewrite_with_input(input, input_change.clone());
        let output_cols = {
            // output cols = { cols required by upstream from input node } ∪ { additional window
            // cols }
            #[derive(Copy, Clone, Debug)]
            enum IndexType {
                Input(usize),
                WindowStart,
                WindowEnd,
            }
            // map the indices from output to input
            let input_required_cols = required_cols
                .iter()
                .filter_map(|&idx| {
                    if let Some(idx) = o2i.try_map(idx) {
                        Some(IndexType::Input(idx))
                    } else if idx == self.window_start_col_idx() {
                        Some(IndexType::WindowStart)
                    } else if idx == self.window_end_col_idx() {
                        Some(IndexType::WindowEnd)
                    } else {
                        None
                    }
                })
                .collect_vec();
            // this mapping will only keeps required columns
            let mapping = input_change.composite(&new_hop.i2o_col_mapping());
            input_required_cols
                .iter()
                .filter_map(|&idx| match idx {
                    IndexType::Input(x) => mapping.try_map(x),
                    IndexType::WindowStart => Some(new_hop.window_start_col_idx()),
                    IndexType::WindowEnd => Some(new_hop.window_end_col_idx()),
                })
                .collect_vec()
        };
        new_hop.clone_with_output_indices(output_cols).into()
    }
}

impl PredicatePushdown for LogicalHopWindow {
    fn predicate_pushdown(&self, predicate: Condition) -> PlanRef {
        gen_filter_and_pushdown(self, predicate, Condition::true_cond())
    }
}

impl ToBatch for LogicalHopWindow {
    fn to_batch(&self) -> Result<PlanRef> {
        let new_input = self.input().to_batch()?;
        let new_logical = self.clone_with_input(new_input);
        Ok(BatchHopWindow::new(new_logical).into())
    }
}

impl ToStream for LogicalHopWindow {
    fn to_stream(&self) -> Result<PlanRef> {
        let new_input = self.input().to_stream()?;
        let new_logical = self.clone_with_input(new_input);
        Ok(StreamHopWindow::new(new_logical).into())
    }

    fn logical_rewrite_for_stream(&self) -> Result<(PlanRef, ColIndexMapping)> {
        let (input, input_col_change) = self.input().logical_rewrite_for_stream()?;
        let (hop, out_col_change) = self.rewrite_with_input(input.clone(), input_col_change);
        let (input, time_col, window_slide, window_size, mut output_indices) = hop.into_parts();
        if !output_indices.contains(&input.schema().len())
            && !output_indices.contains(&(input.schema().len() + 1))
        // When both `window_start` and `window_end` are not in `output_indices`,
        // we add `window_start` to ensure we can derive pk.
        {
            output_indices.push(input.schema().len());
        }
        let i2o = self.i2o_col_mapping();
        output_indices.extend(
            input
                .logical_pk()
                .iter()
                .cloned()
                .filter(|i| i2o.try_map(*i).is_none()),
        );
        let new_hop = Self::new(
            input,
            time_col,
            window_slide,
            window_size,
            Some(output_indices),
        );
        Ok((new_hop.into(), out_col_change))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use piestream_common::catalog::{Field, Schema};
    use piestream_common::types::DataType;

    use super::*;
    use crate::expr::InputRef;
    use crate::optimizer::plan_node::LogicalValues;
    use crate::optimizer::property::FunctionalDependency;
    use crate::session::OptimizerContext;
    #[tokio::test]
    /// Pruning
    /// ```text
    /// HopWindow(time_col: $0 slide: 1 day 00:00:00 size: 3 days 00:00:00)
    ///   TableScan(date, v1, v2)
    /// ```
    /// with required columns [4, 2, 3] will result in
    /// ```text
    ///   HopWindow(time_col: $0 slide: 1 day 00:00:00 size: 3 days 00:00:00 output_indices: [3, 1, 2])
    ///     TableScan(date, v3)
    /// ```
    async fn test_prune_hop_window_with_order_required() {
        let ctx = OptimizerContext::mock().await;
        let fields: Vec<Field> = vec![
            Field::with_name(DataType::Date, "date"),
            Field::with_name(DataType::Int32, "v1"),
            Field::with_name(DataType::Int32, "v2"),
        ];
        let values = LogicalValues::new(
            vec![],
            Schema {
                fields: fields.clone(),
            },
            ctx,
        );
        let hop_window: PlanRef = LogicalHopWindow::new(
            values.into(),
            InputRef::new(0, DataType::Date),
            IntervalUnit::new(0, 1, 0),
            IntervalUnit::new(0, 3, 0),
            None,
        )
        .into();
        // Perform the prune
        let required_cols = vec![4, 2, 3];
        let plan = hop_window.prune_col(&required_cols);
        println!(
            "{}\n{}",
            hop_window.explain_to_string().unwrap(),
            plan.explain_to_string().unwrap()
        );
        // Check the result
        let hop_window = plan.as_logical_hop_window().unwrap();
        assert_eq!(hop_window.core.output_indices, vec![3, 1, 2]);
        assert_eq!(hop_window.schema().fields().len(), 3);

        let values = hop_window.input();
        let values = values.as_logical_values().unwrap();
        assert_eq!(values.schema().fields().len(), 2);
        assert_eq!(values.schema().fields()[0], fields[0]);
        assert_eq!(values.schema().fields()[1], fields[2]);
    }

    #[tokio::test]
    async fn fd_derivation_hop_window() {
        // input: [date, v1, v2]
        // FD: { date, v1 } --> { v2 }
        // output: [date, v1, v2, window_start, window_end],
        // FD: { date, v1 } --> { v2 }
        //     window_start --> window_end
        //     window_end --> window_start
        let ctx = OptimizerContext::mock().await;
        let fields: Vec<Field> = vec![
            Field::with_name(DataType::Date, "date"),
            Field::with_name(DataType::Int32, "v1"),
            Field::with_name(DataType::Int32, "v2"),
        ];
        let mut values = LogicalValues::new(vec![], Schema { fields }, ctx);
        // 0, 1 --> 2
        values
            .base
            .functional_dependency
            .add_functional_dependency_by_column_indices(&[0, 1], &[2]);
        let hop_window: PlanRef = LogicalHopWindow::new(
            values.into(),
            InputRef::new(0, DataType::Date),
            IntervalUnit::new(0, 1, 0),
            IntervalUnit::new(0, 3, 0),
            None,
        )
        .into();
        let fd_set: HashSet<_> = hop_window
            .functional_dependency()
            .as_dependencies()
            .iter()
            .cloned()
            .collect();
        let expected_fd_set: HashSet<_> = [
            FunctionalDependency::with_indices(5, &[0, 1], &[2]),
            FunctionalDependency::with_indices(5, &[3], &[4]),
            FunctionalDependency::with_indices(5, &[4], &[3]),
        ]
        .into_iter()
        .collect();
        assert_eq!(fd_set, expected_fd_set);
    }
}
