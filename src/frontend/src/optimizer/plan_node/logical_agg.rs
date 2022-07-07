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
use std::fmt;

use fixedbitset::FixedBitSet;
use itertools::Itertools;
use piestream_common::catalog::{Field, Schema};
use piestream_common::error::{ErrorCode, Result};
use piestream_common::types::DataType;
use piestream_common::util::sort_util::OrderType;
use piestream_expr::expr::AggKind;
use piestream_pb::expr::AggCall as ProstAggCall;

use super::{
    BatchHashAgg, BatchSimpleAgg, ColPrunable, LogicalProjectBuilder, PlanBase, PlanRef,
    PlanTreeNodeUnary, PredicatePushdown, StreamGlobalSimpleAgg, StreamHashAgg,
    StreamLocalSimpleAgg, ToBatch, ToStream,
};
use crate::catalog::table_catalog::TableCatalog;
use crate::expr::{AggCall, Expr, ExprImpl, ExprRewriter, ExprType, FunctionCall, InputRef};
use crate::optimizer::plan_node::utils::TableCatalogBuilder;
use crate::optimizer::plan_node::{gen_filter_and_pushdown, LogicalProject};
use crate::optimizer::property::{Order, RequiredDist};
use crate::utils::{ColIndexMapping, Condition, Substitute};

/// Aggregation Call
#[derive(Clone)]
pub struct PlanAggCall {
    /// Kind of aggregation function
    pub agg_kind: AggKind,

    /// Data type of the returned column
    pub return_type: DataType,

    /// Column indexes of input columns.
    /// It's vary-length by design:
    /// can be 0-len (`RowCount`), 1-len (`Max`, `Min`), 2-len (`StringAgg`).
    /// Usually, we mark the first column as the aggregated column.
    pub inputs: Vec<InputRef>,

    pub distinct: bool,
    /// Selective aggregation: only the input rows for which
    /// the filter_clause evaluates to true will be fed to aggregate function.
    /// Other rows are discarded.
    pub filter: Condition,
}

impl fmt::Debug for PlanAggCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.agg_kind)?;
        if !self.inputs.is_empty() {
            write!(f, "(")?;
            for (idx, input) in self.inputs.iter().enumerate() {
                if idx == 0 && self.distinct {
                    write!(f, "distinct ")?;
                }
                write!(f, "{:?}", input)?;
                if idx != (self.inputs.len() - 1) {
                    write!(f, ",")?;
                }
            }
            write!(f, ")")?;
        }
        if !self.filter.always_true() {
            write!(
                f,
                " filter({:?})",
                self.filter.as_expr_unless_true().unwrap()
            )?;
        }
        Ok(())
    }
}

impl PlanAggCall {
    pub fn to_protobuf(&self) -> ProstAggCall {
        ProstAggCall {
            r#type: self.agg_kind.to_prost().into(),
            return_type: Some(self.return_type.to_protobuf()),
            args: self.inputs.iter().map(InputRef::to_agg_arg_proto).collect(),
            distinct: self.distinct,
        }
    }

    pub fn partial_to_total_agg_call(&self, partial_output_idx: usize) -> PlanAggCall {
        let total_agg_kind = match &self.agg_kind {
            AggKind::Min
            | AggKind::Max
            | AggKind::Avg
            | AggKind::StringAgg
            | AggKind::SingleValue => self.agg_kind.clone(),

            AggKind::Count | AggKind::RowCount | AggKind::Sum | AggKind::ApproxCountDistinct => {
                AggKind::Sum
            }
        };
        PlanAggCall {
            agg_kind: total_agg_kind,
            inputs: vec![InputRef::new(partial_output_idx, self.return_type.clone())],
            filter: Condition::true_cond(),
            ..self.clone()
        }
    }

    pub fn count_star() -> Self {
        PlanAggCall {
            agg_kind: AggKind::Count,
            return_type: DataType::Int64,
            inputs: vec![],
            distinct: false,
            filter: Condition::true_cond(),
        }
    }
}

/// `LogicalAgg` groups input data by their group keys and computes aggregation functions.
///
/// It corresponds to the `GROUP BY` operator in a SQL query statement together with the aggregate
/// functions in the `SELECT` clause.
///
/// The output schema will first include the group keys and then the aggregation calls.
#[derive(Clone, Debug)]
pub struct LogicalAgg {
    pub base: PlanBase,
    agg_calls: Vec<PlanAggCall>,
    group_keys: Vec<usize>,
    input: PlanRef,
}

impl LogicalAgg {
    pub fn infer_internal_table_catalog(&self) -> (Vec<TableCatalog>, HashMap<usize, i32>) {
        let mut table_catalogs = vec![];
        let base = self.input.plan_base();
        let schema = &base.schema;
        let fields = schema.fields();
        let append_only = self.input.append_only();
        for agg_call in &self.agg_calls {
            let mut internal_table_catalog_builder = TableCatalogBuilder::new();
            for &idx in &self.group_keys {
                internal_table_catalog_builder
                    .add_column_desc_from_field(Some(OrderType::Ascending), &fields[idx]);
            }
            match agg_call.agg_kind {
                AggKind::Min | AggKind::Max | AggKind::StringAgg => {
                    if !append_only {
                        // Add sort key as part of pk.
                        let order_type = if agg_call.agg_kind == AggKind::Min {
                            OrderType::Ascending
                        } else {
                            OrderType::Descending
                        };
                        // Assume the first input must be the aggregated column at least for these 3
                        // kind of AggCall.
                        internal_table_catalog_builder.add_column_desc_from_field(
                            Some(order_type),
                            &fields[agg_call.inputs[0].index],
                        );

                        // Add upstream pk.
                        for pk_index in &base.pk_indices {
                            internal_table_catalog_builder.add_column_desc_from_field(
                                Some(OrderType::Ascending),
                                &fields[*pk_index],
                            );
                        }
                    }

                    // TODO: Remove this (3474)
                    internal_table_catalog_builder.add_column_desc_from_field(
                        Some(OrderType::Ascending),
                        &fields[agg_call.inputs[0].index],
                    );
                }
                AggKind::Sum
                | AggKind::Count
                | AggKind::RowCount
                | AggKind::Avg
                | AggKind::SingleValue
                | AggKind::ApproxCountDistinct => {
                    // Here we do not use add column from field cuz for `RowCount`, it's input is
                    // empty vector, means do not derives from any data in upstream fields.
                    internal_table_catalog_builder
                        .add_unnamed_column(None, agg_call.return_type.clone());
                }
            }
            // Always reserve 1 for agg call value. See related issue (#3474).
            table_catalogs.push(
                internal_table_catalog_builder
                    .build(base.dist.dist_column_indices().to_vec(), append_only),
            );
        }
        // TODO: fill column mapping later (#3485).
        (table_catalogs, HashMap::new())
    }

    /// Two phase streaming agg.
    /// Should only be used iff input is distributed.
    /// input must be converted to stream form.
    fn gen_two_phase_streaming_agg_plan(&self, input_stream: PlanRef) -> Result<PlanRef> {
        // partial agg
        let partial_agg_plan =
            StreamLocalSimpleAgg::new(self.clone_with_input(input_stream)).into();

        let input =
            RequiredDist::single().enforce_if_not_satisfies(partial_agg_plan, &Order::any())?;

        // insert total agg
        let total_agg_types = self
            .agg_calls()
            .iter()
            .enumerate()
            .map(|(partial_output_idx, agg_call)| {
                agg_call.partial_to_total_agg_call(partial_output_idx)
            })
            .collect();
        let total_agg_logical_plan =
            LogicalAgg::new(total_agg_types, self.group_keys().to_vec(), input);
        Ok(StreamGlobalSimpleAgg::new(total_agg_logical_plan).into())
    }
}

/// `LogicalAggBuilder` extracts agg calls and references to group columns from select list and
/// build the plan like `LogicalAgg - LogicalProject`.
/// it is constructed by `group_exprs` and collect and rewrite the expression in selection and
/// having clause.
struct LogicalAggBuilder {
    /// the builder of the input Project
    input_proj_builder: LogicalProjectBuilder,
    /// the group keys column indices in the project's output
    group_keys: Vec<usize>,
    /// the agg calls
    agg_calls: Vec<PlanAggCall>,
    /// the error during the expression rewriting
    error: Option<ErrorCode>,
    /// If `is_in_filter_clause` is true, it means that
    /// we are processing a filter clause.
    /// This field is needed because input refs in filter clause
    /// are allowed to refer to any columns, while those not in filter
    /// clause are only allowed to refer to group keys.
    is_in_filter_clause: bool,
}

impl LogicalAggBuilder {
    fn new(group_exprs: Vec<ExprImpl>) -> Result<Self> {
        let mut input_proj_builder = LogicalProjectBuilder::default();

        for expr in &group_exprs {
            if expr.has_subquery() || expr.has_agg_call() {
                return Err(ErrorCode::InvalidInputSyntax(
                    "GROUP BY expr should not contain subquery or aggregation function".into(),
                )
                .into());
            }
        }

        let group_keys = group_exprs
            .into_iter()
            .map(|expr| input_proj_builder.add_expr(&expr))
            .collect_vec();

        Ok(LogicalAggBuilder {
            group_keys,
            agg_calls: vec![],
            error: None,
            input_proj_builder,
            is_in_filter_clause: false,
        })
    }

    pub fn build(self, input: PlanRef) -> LogicalAgg {
        // This LogicalProject focuses on the exprs in aggregates and GROUP BY clause.
        let logical_project = self.input_proj_builder.build(input);

        // This LogicalAgg focuses on calculating the aggregates and grouping.
        LogicalAgg::new(self.agg_calls, self.group_keys, logical_project.into())
    }

    fn rewrite_with_error(&mut self, expr: ExprImpl) -> Result<ExprImpl> {
        let rewritten_expr = self.rewrite_expr(expr);
        if let Some(error) = self.error.take() {
            return Err(error.into());
        }
        Ok(rewritten_expr)
    }

    /// check if the expression is a group by key, and try to return the group key
    pub fn try_as_group_expr(&self, expr: &ExprImpl) -> Option<usize> {
        if let Some(input_index) = self.input_proj_builder.expr_index(expr) {
            if let Some(index) = self
                .group_keys
                .iter()
                .position(|group_key| *group_key == input_index)
            {
                return Some(index);
            }
        }
        None
    }
}

impl ExprRewriter for LogicalAggBuilder {
    /// When there is an agg call, there are 3 things to do:
    /// 1. eval its inputs via project;
    /// 2. add a `PlanAggCall` to agg;
    /// 3. rewrite it as an `InputRef` to the agg result in select list.
    ///
    /// Note that the rewriter does not traverse into inputs of agg calls.
    fn rewrite_agg_call(&mut self, agg_call: AggCall) -> ExprImpl {
        let return_type = agg_call.return_type();
        let (agg_kind, inputs, distinct, filter) = agg_call.decompose();
        self.is_in_filter_clause = true;
        let filter = filter.rewrite_expr(self);
        self.is_in_filter_clause = false;
        for i in &inputs {
            if i.has_agg_call() {
                self.error = Some(ErrorCode::InvalidInputSyntax(
                    "Aggregation calls should not be nested".into(),
                ));
                return AggCall::new(agg_kind, inputs, distinct, filter)
                    .unwrap()
                    .into();
            }
        }

        let inputs = inputs
            .iter()
            .map(|expr| {
                let index = self.input_proj_builder.add_expr(expr);
                InputRef::new(index, expr.return_type())
            })
            .collect_vec();

        if agg_kind == AggKind::Avg {
            assert_eq!(inputs.len(), 1);

            let left_return_type =
                AggCall::infer_return_type(&AggKind::Sum, &[inputs[0].return_type()]).unwrap();

            // Rewrite avg to cast(sum as avg_return_type) / count.
            self.agg_calls.push(PlanAggCall {
                agg_kind: AggKind::Sum,
                return_type: left_return_type.clone(),
                inputs: inputs.clone(),
                distinct,
                filter: filter.clone(),
            });
            let left = ExprImpl::from(InputRef::new(
                self.group_keys.len() + self.agg_calls.len() - 1,
                left_return_type,
            ))
            .cast_implicit(return_type)
            .unwrap();

            let right_return_type =
                AggCall::infer_return_type(&AggKind::Count, &[inputs[0].return_type()]).unwrap();

            self.agg_calls.push(PlanAggCall {
                agg_kind: AggKind::Count,
                return_type: right_return_type.clone(),
                inputs,
                distinct,
                filter,
            });

            let right = InputRef::new(
                self.group_keys.len() + self.agg_calls.len() - 1,
                right_return_type,
            );

            ExprImpl::from(FunctionCall::new(ExprType::Divide, vec![left, right.into()]).unwrap())
        } else {
            self.agg_calls.push(PlanAggCall {
                agg_kind,
                return_type: return_type.clone(),
                inputs,
                distinct,
                filter,
            });
            ExprImpl::from(InputRef::new(
                self.group_keys.len() + self.agg_calls.len() - 1,
                return_type,
            ))
        }
    }

    /// When there is an `FunctionCall` (outside of agg call), it must refers to a group column.
    /// Or all `InputRef`s appears in it must refer to a group column.
    fn rewrite_function_call(&mut self, func_call: FunctionCall) -> ExprImpl {
        let expr = func_call.into();
        if let Some(group_key) = self.try_as_group_expr(&expr) {
            InputRef::new(group_key, expr.return_type()).into()
        } else {
            let (func_type, inputs, ret) = expr.into_function_call().unwrap().decompose();
            let inputs = inputs
                .into_iter()
                .map(|expr| self.rewrite_expr(expr))
                .collect();
            FunctionCall::new_unchecked(func_type, inputs, ret).into()
        }
    }

    /// When there is an `InputRef` (outside of agg call), it must refers to a group column.
    fn rewrite_input_ref(&mut self, input_ref: InputRef) -> ExprImpl {
        let expr = input_ref.into();
        if let Some(group_key) = self.try_as_group_expr(&expr) {
            InputRef::new(group_key, expr.return_type()).into()
        } else if self.is_in_filter_clause {
            InputRef::new(self.input_proj_builder.add_expr(&expr), expr.return_type()).into()
        } else {
            self.error = Some(ErrorCode::InvalidInputSyntax(
                "column must appear in the GROUP BY clause or be used in an aggregate function"
                    .into(),
            ));
            expr
        }
    }

    fn rewrite_subquery(&mut self, subquery: crate::expr::Subquery) -> ExprImpl {
        if subquery.is_correlated() {
            self.error = Some(ErrorCode::NotImplemented(
                "correlated subquery in HAVING or SELECT with agg".into(),
                2275.into(),
            ));
        }
        subquery.into()
    }
}

impl LogicalAgg {
    pub fn new(agg_calls: Vec<PlanAggCall>, group_keys: Vec<usize>, input: PlanRef) -> Self {
        let ctx = input.ctx();
        let schema = Self::derive_schema(
            input.schema(),
            &group_keys,
            agg_calls
                .iter()
                .map(|agg_call| agg_call.return_type.clone())
                .collect(),
        );
        let pk_indices = match group_keys.is_empty() {
            // simple agg
            true => vec![],
            // group agg
            false => group_keys.clone(),
        };
        let base = PlanBase::new_logical(ctx, schema, pk_indices);
        Self {
            base,
            agg_calls,
            group_keys,
            input,
        }
    }

    /// get the Mapping of columnIndex from input column index to output column index,if a input
    /// column corresponds more than one out columns, mapping to any one
    pub fn o2i_col_mapping(&self) -> ColIndexMapping {
        let input_len = self.input.schema().len();
        let agg_cal_num = self.agg_calls().len();
        let group_keys = self.group_keys();
        let mut map = vec![None; agg_cal_num + group_keys.len()];
        for (i, key) in group_keys.iter().enumerate() {
            map[i] = Some(*key);
        }
        ColIndexMapping::with_target_size(map, input_len)
    }

    /// get the Mapping of columnIndex from input column index to out column index
    pub fn i2o_col_mapping(&self) -> ColIndexMapping {
        self.o2i_col_mapping().inverse()
    }

    /// get the Mapping of columnIndex from input column index to out column index
    pub fn i2o_col_mapping_with_required_out(&self, required: &FixedBitSet) -> ColIndexMapping {
        self.o2i_col_mapping().inverse_with_required(required)
    }

    fn derive_schema(
        input: &Schema,
        group_keys: &[usize],
        agg_call_data_types: Vec<DataType>,
    ) -> Schema {
        let fields = group_keys
            .iter()
            .cloned()
            .map(|i| input.fields()[i].clone())
            .chain(
                agg_call_data_types
                    .into_iter()
                    .enumerate()
                    .map(|(id, data_type)| {
                        let name = format!("agg#{}", id);
                        Field::with_name(data_type, name)
                    }),
            )
            .collect();
        Schema { fields }
    }

    /// `create` will analyze select exprs, group exprs and having, and construct a plan like
    ///
    /// ```text
    /// LogicalAgg -> LogicalProject -> input
    /// ```
    ///
    /// It also returns the rewritten select exprs and having that reference into the aggregated
    /// results.
    pub fn create(
        select_exprs: Vec<ExprImpl>,
        group_exprs: Vec<ExprImpl>,
        having: Option<ExprImpl>,
        input: PlanRef,
    ) -> Result<(PlanRef, Vec<ExprImpl>, Option<ExprImpl>)> {
        let mut agg_builder = LogicalAggBuilder::new(group_exprs)?;

        let rewritten_select_exprs = select_exprs
            .into_iter()
            .map(|expr| agg_builder.rewrite_with_error(expr))
            .collect::<Result<_>>()?;
        let rewritten_having = having
            .map(|expr| agg_builder.rewrite_with_error(expr))
            .transpose()?;

        Ok((
            agg_builder.build(input).into(),
            rewritten_select_exprs,
            rewritten_having,
        ))
    }

    /// Get a reference to the logical agg's agg calls.
    pub fn agg_calls(&self) -> &[PlanAggCall] {
        self.agg_calls.as_ref()
    }

    /// Get a reference to the logical agg's group keys.
    pub fn group_keys(&self) -> &[usize] {
        self.group_keys.as_ref()
    }

    pub fn decompose(self) -> (Vec<PlanAggCall>, Vec<usize>, PlanRef) {
        (self.agg_calls, self.group_keys, self.input)
    }
}

impl PlanTreeNodeUnary for LogicalAgg {
    fn input(&self) -> PlanRef {
        self.input.clone()
    }

    fn clone_with_input(&self, input: PlanRef) -> Self {
        Self::new(self.agg_calls().to_vec(), self.group_keys().to_vec(), input)
    }

    #[must_use]
    fn rewrite_with_input(
        &self,
        input: PlanRef,
        mut input_col_change: ColIndexMapping,
    ) -> (Self, ColIndexMapping) {
        let agg_calls = self
            .agg_calls
            .iter()
            .cloned()
            .map(|mut agg_call| {
                agg_call.inputs.iter_mut().for_each(|i| {
                    *i = InputRef::new(input_col_change.map(i.index()), i.return_type())
                });
                agg_call.filter = agg_call.filter.rewrite_expr(&mut input_col_change);
                agg_call
            })
            .collect();
        let group_keys = self
            .group_keys
            .iter()
            .cloned()
            .map(|key| input_col_change.map(key))
            .collect();
        let agg = Self::new(agg_calls, group_keys, input);
        // change the input columns index will not change the output column index
        let out_col_change = ColIndexMapping::identity(agg.schema().len());
        (agg, out_col_change)
    }
}

impl_plan_tree_node_for_unary! {LogicalAgg}

impl fmt::Display for LogicalAgg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LogicalAgg")
            .field("group_keys", &self.group_keys)
            .field("agg_calls", &self.agg_calls)
            .finish()
    }
}

impl ColPrunable for LogicalAgg {
    fn prune_col(&self, required_cols: &[usize]) -> PlanRef {
        let upstream_required_cols = {
            let mapping = self.o2i_col_mapping();
            FixedBitSet::from_iter(
                required_cols
                    .iter()
                    .filter_map(|&output_idx| mapping.try_map(output_idx)),
            )
        };
        let group_key_required_cols = FixedBitSet::from_iter(self.group_keys.iter().copied());

        let (agg_call_required_cols, agg_calls) = {
            let input_cnt = self.input().schema().fields().len();
            let mut tmp = FixedBitSet::with_capacity(input_cnt);
            let new_agg_calls = required_cols
                .iter()
                .filter(|&&index| index >= self.group_keys.len())
                .map(|&index| {
                    let index = index - self.group_keys.len();
                    let agg_call = self.agg_calls[index].clone();
                    tmp.extend(agg_call.inputs.iter().map(|x| x.index()));
                    // collect columns used in aggregate filter expressions
                    for i in &agg_call.filter.conjunctions {
                        tmp.union_with(&i.collect_input_refs(input_cnt));
                    }
                    agg_call
                })
                .collect_vec();
            (tmp, new_agg_calls)
        };

        let input_required_cols = {
            let mut tmp: FixedBitSet = upstream_required_cols;
            tmp.union_with(&group_key_required_cols);
            tmp.union_with(&agg_call_required_cols);
            tmp.ones().collect_vec()
        };
        let mapping = ColIndexMapping::with_remaining_columns(
            &input_required_cols,
            self.input().schema().len(),
        );
        let agg = {
            let agg_calls = agg_calls
                .iter()
                .cloned()
                .map(|mut agg_call| {
                    agg_call
                        .inputs
                        .iter_mut()
                        .for_each(|i| *i = InputRef::new(mapping.map(i.index()), i.return_type()));
                    agg_call
                })
                .collect();
            let group_keys = self
                .group_keys
                .iter()
                .cloned()
                .map(|key| mapping.map(key))
                .collect();
            LogicalAgg::new(
                agg_calls,
                group_keys,
                self.input.prune_col(&input_required_cols),
            )
        };
        let new_output_cols = {
            // group keys were never pruned or even re-ordered in current impl
            let mut tmp = (0..agg.group_keys().len()).collect_vec();
            tmp.extend(
                required_cols
                    .iter()
                    .filter(|&&index| index >= self.group_keys.len()),
            );
            tmp
        };
        if new_output_cols == required_cols {
            // current schema perfectly fit the required columns
            agg.into()
        } else {
            // some columns are not needed, or the order need to be adjusted.
            // so we did a projection to remove/reorder the columns.
            let mapping =
                &ColIndexMapping::with_remaining_columns(&new_output_cols, self.schema().len());
            let output_required_cols = required_cols
                .iter()
                .map(|&idx| mapping.map(idx))
                .collect_vec();
            let src_size = agg.schema().len();
            LogicalProject::with_mapping(
                agg.into(),
                ColIndexMapping::with_remaining_columns(&output_required_cols, src_size),
            )
            .into()
        }
    }
}

impl PredicatePushdown for LogicalAgg {
    fn predicate_pushdown(&self, predicate: Condition) -> PlanRef {
        let num_group_keys = self.group_keys.len();
        let num_agg_calls = self.agg_calls.len();
        assert!(num_group_keys + num_agg_calls == self.schema().len());

        // SimpleAgg should be skipped because the predicate either references agg_calls
        // or is const.
        // If the filter references agg_calls, we can not push it.
        // When it is constantly true, pushing is useless and may actually cause more evaulation
        // cost of the predicate.
        // When it is constantly false, pushing is wrong - the old plan returns 0 rows but new one
        // returns 1 row.
        if num_group_keys == 0 {
            return gen_filter_and_pushdown(self, predicate, Condition::true_cond());
        }

        // If the filter references agg_calls, we can not push it.
        let mut agg_call_columns = FixedBitSet::with_capacity(num_group_keys + num_agg_calls);
        agg_call_columns.insert_range(num_group_keys..num_group_keys + num_agg_calls);
        let (agg_call_pred, pushed_predicate) = predicate.split_disjoint(&agg_call_columns);

        // convert the predicate to one that references the child of the agg
        let mut subst = Substitute {
            mapping: self
                .group_keys()
                .iter()
                .enumerate()
                .map(|(i, group_key)| {
                    InputRef::new(*group_key, self.schema().fields()[i].data_type()).into()
                })
                .collect(),
        };
        let pushed_predicate = pushed_predicate.rewrite_expr(&mut subst);

        gen_filter_and_pushdown(self, agg_call_pred, pushed_predicate)
    }
}

impl ToBatch for LogicalAgg {
    fn to_batch(&self) -> Result<PlanRef> {
        let new_input = self.input().to_batch()?;
        let new_logical = self.clone_with_input(new_input);
        if self.group_keys().is_empty() {
            Ok(BatchSimpleAgg::new(new_logical).into())
        } else {
            Ok(BatchHashAgg::new(new_logical).into())
        }
    }
}

impl ToStream for LogicalAgg {
    fn to_stream(&self) -> Result<PlanRef> {
        let input = self.input();
        // simple-agg
        if self.group_keys().is_empty() {
            // TODO: Other agg calls will be supported by stateful local agg eventually.
            let agg_calls_can_use_two_phase = self
                .agg_calls
                .iter()
                .all(|c| matches!(c.agg_kind, AggKind::Count | AggKind::Sum));

            let input_stream = input.to_stream()?;
            let input_distribution = input_stream.distribution();

            // simple 2-phase-agg
            if input_distribution.satisfies(&RequiredDist::AnyShard) && agg_calls_can_use_two_phase
            {
                self.gen_two_phase_streaming_agg_plan(input_stream)
            // simple 1-phase-agg
            } else {
                Ok(StreamGlobalSimpleAgg::new(self.clone_with_input(
                    input.to_stream_with_dist_required(&RequiredDist::single())?,
                ))
                .into())
            }

        // hash-agg
        } else {
            Ok(
                StreamHashAgg::new(self.clone_with_input(input.to_stream_with_dist_required(
                    &RequiredDist::shard_by_key(input.schema().len(), self.group_keys()),
                )?))
                .into(),
            )
        }
    }

    fn logical_rewrite_for_stream(&self) -> Result<(PlanRef, ColIndexMapping)> {
        let (input, input_col_change) = self.input.logical_rewrite_for_stream()?;
        let (agg, out_col_change) = self.rewrite_with_input(input, input_col_change);

        // To rewrite StreamAgg, there are two things to do:
        // 1. insert a RowCount(Count with zero argument) at the beginning of agg_calls of
        // LogicalAgg.
        // 2. increment the index of agg_calls in `out_col_change` by 1 due to
        // the insertion of RowCount, and it will be used to rewrite LogicalProject above this
        // LogicalAgg.
        // Please note that the index of group keys need not be changed.
        let (mut agg_calls, group_keys, input) = agg.decompose();
        agg_calls.insert(0, PlanAggCall::count_star());

        let (mut map, _) = out_col_change.into_parts();
        map.iter_mut().skip(group_keys.len()).for_each(|index| {
            if let Some(i) = *index {
                *index = Some(i + 1);
            }
        });

        Ok((
            LogicalAgg::new(agg_calls, group_keys, input).into(),
            ColIndexMapping::new(map),
        ))
    }
}

#[cfg(test)]
mod tests {

    use std::rc::Rc;

    use piestream_common::catalog::Field;
    use piestream_common::types::DataType;

    use super::*;
    use crate::expr::{
        assert_eq_input_ref, input_ref_to_column_indices, AggCall, ExprType, FunctionCall,
    };
    use crate::optimizer::plan_node::LogicalValues;
    use crate::session::OptimizerContext;

    #[tokio::test]
    async fn test_create() {
        let ty = DataType::Int32;
        let ctx = OptimizerContext::mock().await;
        let fields: Vec<Field> = vec![
            Field::with_name(ty.clone(), "v1"),
            Field::with_name(ty.clone(), "v2"),
            Field::with_name(ty.clone(), "v3"),
        ];
        let values = LogicalValues::new(vec![], Schema { fields }, ctx);
        let input = Rc::new(values);
        let input_ref_1 = InputRef::new(0, ty.clone());
        let input_ref_2 = InputRef::new(1, ty.clone());
        let input_ref_3 = InputRef::new(2, ty.clone());

        let gen_internal_value = |select_exprs: Vec<ExprImpl>,
                                  group_exprs|
         -> (Vec<ExprImpl>, Vec<PlanAggCall>, Vec<usize>) {
            let (plan, exprs, _) =
                LogicalAgg::create(select_exprs, group_exprs, None, input.clone()).unwrap();

            let logical_agg = plan.as_logical_agg().unwrap();
            let agg_calls = logical_agg.agg_calls().to_vec();
            let group_keys = logical_agg.group_keys().to_vec();

            (exprs, agg_calls, group_keys)
        };

        // Test case: select v1 from test group by v1;
        {
            let select_exprs = vec![input_ref_1.clone().into()];
            let group_exprs = vec![input_ref_1.clone().into()];

            let (exprs, agg_calls, group_keys) = gen_internal_value(select_exprs, group_exprs);

            assert_eq!(exprs.len(), 1);
            assert_eq_input_ref!(&exprs[0], 0);

            assert_eq!(agg_calls.len(), 0);
            assert_eq!(group_keys, vec![0]);
        }

        // Test case: select v1, min(v2) from test group by v1;
        {
            let min_v2 = AggCall::new(
                AggKind::Min,
                vec![input_ref_2.clone().into()],
                false,
                Condition::true_cond(),
            )
            .unwrap();
            let select_exprs = vec![input_ref_1.clone().into(), min_v2.into()];
            let group_exprs = vec![input_ref_1.clone().into()];

            let (exprs, agg_calls, group_keys) = gen_internal_value(select_exprs, group_exprs);

            assert_eq!(exprs.len(), 2);
            assert_eq_input_ref!(&exprs[0], 0);
            assert_eq_input_ref!(&exprs[1], 1);

            assert_eq!(agg_calls.len(), 1);
            assert_eq!(agg_calls[0].agg_kind, AggKind::Min);
            assert_eq!(input_ref_to_column_indices(&agg_calls[0].inputs), vec![1]);
            assert_eq!(group_keys, vec![0]);
        }

        // Test case: select v1, min(v2) + max(v3) from t group by v1;
        {
            let min_v2 = AggCall::new(
                AggKind::Min,
                vec![input_ref_2.clone().into()],
                false,
                Condition::true_cond(),
            )
            .unwrap();
            let max_v3 = AggCall::new(
                AggKind::Max,
                vec![input_ref_3.clone().into()],
                false,
                Condition::true_cond(),
            )
            .unwrap();
            let func_call =
                FunctionCall::new(ExprType::Add, vec![min_v2.into(), max_v3.into()]).unwrap();
            let select_exprs = vec![input_ref_1.clone().into(), ExprImpl::from(func_call)];
            let group_exprs = vec![input_ref_1.clone().into()];

            let (exprs, agg_calls, group_keys) = gen_internal_value(select_exprs, group_exprs);

            assert_eq_input_ref!(&exprs[0], 0);
            if let ExprImpl::FunctionCall(func_call) = &exprs[1] {
                assert_eq!(func_call.get_expr_type(), ExprType::Add);
                let inputs = func_call.inputs();
                assert_eq_input_ref!(&inputs[0], 1);
                assert_eq_input_ref!(&inputs[1], 2);
            } else {
                panic!("Wrong expression type!");
            }

            assert_eq!(agg_calls.len(), 2);
            assert_eq!(agg_calls[0].agg_kind, AggKind::Min);
            assert_eq!(input_ref_to_column_indices(&agg_calls[0].inputs), vec![1]);
            assert_eq!(agg_calls[1].agg_kind, AggKind::Max);
            assert_eq!(input_ref_to_column_indices(&agg_calls[1].inputs), vec![2]);
            assert_eq!(group_keys, vec![0]);
        }

        // Test case: select v2, min(v1 * v3) from test group by v2;
        {
            let v1_mult_v3 = FunctionCall::new(
                ExprType::Multiply,
                vec![input_ref_1.into(), input_ref_3.into()],
            )
            .unwrap();
            let agg_call = AggCall::new(
                AggKind::Min,
                vec![v1_mult_v3.into()],
                false,
                Condition::true_cond(),
            )
            .unwrap();
            let select_exprs = vec![input_ref_2.clone().into(), agg_call.into()];
            let group_exprs = vec![input_ref_2.into()];

            let (exprs, agg_calls, group_keys) = gen_internal_value(select_exprs, group_exprs);

            assert_eq_input_ref!(&exprs[0], 0);
            assert_eq_input_ref!(&exprs[1], 1);

            assert_eq!(agg_calls.len(), 1);
            assert_eq!(agg_calls[0].agg_kind, AggKind::Min);
            assert_eq!(input_ref_to_column_indices(&agg_calls[0].inputs), vec![1]);
            assert_eq!(group_keys, vec![0]);
        }
    }
    /// Generate a agg call node with given [`DataType`] and fields.
    /// For example, `generate_agg_call(Int32, [v1, v2, v3])` will result in:
    /// ```text
    /// Agg(min(input_ref(2))) group by (input_ref(1))
    ///   TableScan(v1, v2, v3)
    /// ```
    async fn generate_agg_call(ty: DataType, fields: Vec<Field>) -> LogicalAgg {
        let ctx = OptimizerContext::mock().await;

        let values = LogicalValues::new(vec![], Schema { fields }, ctx);
        let agg_call = PlanAggCall {
            agg_kind: AggKind::Min,
            return_type: ty.clone(),
            inputs: vec![InputRef::new(2, ty.clone())],
            distinct: false,
            filter: Condition::true_cond(),
        };
        LogicalAgg::new(vec![agg_call], vec![1], values.into())
    }

    #[tokio::test]
    /// Pruning
    /// ```text
    /// Agg(min(input_ref(2))) group by (input_ref(1))
    ///   TableScan(v1, v2, v3)
    /// ```
    /// with required columns [0,1] (all columns) will result in
    /// ```text
    /// Agg(min(input_ref(1))) group by (input_ref(0))
    ///  TableScan(v2, v3)
    /// ```
    async fn test_prune_all() {
        let ty = DataType::Int32;
        let fields: Vec<Field> = vec![
            Field::with_name(ty.clone(), "v1"),
            Field::with_name(ty.clone(), "v2"),
            Field::with_name(ty.clone(), "v3"),
        ];
        let agg = generate_agg_call(ty.clone(), fields.clone()).await;
        // Perform the prune
        let required_cols = vec![0, 1];
        let plan = agg.prune_col(&required_cols);

        // Check the result
        let agg_new = plan.as_logical_agg().unwrap();
        assert_eq!(agg_new.group_keys(), vec![0]);

        assert_eq!(agg_new.agg_calls.len(), 1);
        let agg_call_new = agg_new.agg_calls[0].clone();
        assert_eq!(agg_call_new.agg_kind, AggKind::Min);
        assert_eq!(input_ref_to_column_indices(&agg_call_new.inputs), vec![1]);
        assert_eq!(agg_call_new.return_type, ty);

        let values = agg_new.input();
        let values = values.as_logical_values().unwrap();
        assert_eq!(values.schema().fields(), &fields[1..]);
    }

    #[tokio::test]
    /// Pruning
    /// ```text
    /// Agg(min(input_ref(2))) group by (input_ref(1))
    ///   TableScan(v1, v2, v3)
    /// ```
    /// with required columns [1,0] (all columns, with reversed order) will result in
    /// ```text
    /// Project [input_ref(1), input_ref(0)]
    ///   Agg(min(input_ref(1))) group by (input_ref(0))
    ///     TableScan(v2, v3)
    /// ```
    async fn test_prune_all_with_order_required() {
        let ty = DataType::Int32;
        let fields: Vec<Field> = vec![
            Field::with_name(ty.clone(), "v1"),
            Field::with_name(ty.clone(), "v2"),
            Field::with_name(ty.clone(), "v3"),
        ];
        let agg = generate_agg_call(ty.clone(), fields.clone()).await;
        // Perform the prune
        let required_cols = vec![1, 0];
        let plan = agg.prune_col(&required_cols);
        // Check the result
        let proj = plan.as_logical_project().unwrap();
        assert_eq!(proj.exprs().len(), 2);
        assert_eq!(proj.exprs()[0].as_input_ref().unwrap().index(), 1);
        assert_eq!(proj.exprs()[1].as_input_ref().unwrap().index(), 0);
        let proj_input = proj.input();
        let agg_new = proj_input.as_logical_agg().unwrap();
        assert_eq!(agg_new.group_keys(), vec![0]);

        assert_eq!(agg_new.agg_calls.len(), 1);
        let agg_call_new = agg_new.agg_calls[0].clone();
        assert_eq!(agg_call_new.agg_kind, AggKind::Min);
        assert_eq!(input_ref_to_column_indices(&agg_call_new.inputs), vec![1]);
        assert_eq!(agg_call_new.return_type, ty);

        let values = agg_new.input();
        let values = values.as_logical_values().unwrap();
        assert_eq!(values.schema().fields(), &fields[1..]);
    }

    #[tokio::test]
    /// Pruning
    /// ```text
    /// Agg(min(input_ref(2))) group by (input_ref(1))
    ///   TableScan(v1, v2, v3)
    /// ```
    /// with required columns [1] (group key removed) will result in
    /// ```text
    /// Project(input_ref(1))
    ///   Agg(min(input_ref(1))) group by (input_ref(0))
    ///     TableScan(v2, v3)
    /// ```
    async fn test_prune_group_key() {
        let ctx = OptimizerContext::mock().await;
        let ty = DataType::Int32;
        let fields: Vec<Field> = vec![
            Field::with_name(ty.clone(), "v1"),
            Field::with_name(ty.clone(), "v2"),
            Field::with_name(ty.clone(), "v3"),
        ];
        let values = LogicalValues::new(
            vec![],
            Schema {
                fields: fields.clone(),
            },
            ctx,
        );
        let agg_call = PlanAggCall {
            agg_kind: AggKind::Min,
            return_type: ty.clone(),
            inputs: vec![InputRef::new(2, ty.clone())],
            distinct: false,
            filter: Condition::true_cond(),
        };
        let agg = LogicalAgg::new(vec![agg_call], vec![1], values.into());

        // Perform the prune
        let required_cols = vec![1];
        let plan = agg.prune_col(&required_cols);

        // Check the result
        let project = plan.as_logical_project().unwrap();
        assert_eq!(project.exprs().len(), 1);
        assert_eq_input_ref!(&project.exprs()[0], 1);
        assert_eq!(project.id().0, 4);

        let agg_new = project.input();
        let agg_new = agg_new.as_logical_agg().unwrap();
        assert_eq!(agg_new.group_keys(), vec![0]);
        assert_eq!(agg_new.id().0, 3);

        assert_eq!(agg_new.agg_calls.len(), 1);
        let agg_call_new = agg_new.agg_calls[0].clone();
        assert_eq!(agg_call_new.agg_kind, AggKind::Min);
        assert_eq!(input_ref_to_column_indices(&agg_call_new.inputs), vec![1]);
        assert_eq!(agg_call_new.return_type, ty);

        let values = agg_new.input();
        let values = values.as_logical_values().unwrap();
        assert_eq!(values.schema().fields(), &fields[1..]);
    }

    #[tokio::test]
    /// Pruning
    /// ```text
    /// Agg(min(input_ref(2)), max(input_ref(1))) group by (input_ref(1), input_ref(2))
    ///   TableScan(v1, v2, v3)
    /// ```
    /// with required columns [0,3] will result in
    /// ```text
    /// Project(input_ref(0), input_ref(2))
    ///   Agg(max(input_ref(0))) group by (input_ref(0), input_ref(1))
    ///     TableScan(v2, v3)
    /// ```
    async fn test_prune_agg() {
        let ty = DataType::Int32;
        let ctx = OptimizerContext::mock().await;
        let fields: Vec<Field> = vec![
            Field::with_name(ty.clone(), "v1"),
            Field::with_name(ty.clone(), "v2"),
            Field::with_name(ty.clone(), "v3"),
        ];
        let values = LogicalValues::new(
            vec![],
            Schema {
                fields: fields.clone(),
            },
            ctx,
        );

        let agg_calls = vec![
            PlanAggCall {
                agg_kind: AggKind::Min,
                return_type: ty.clone(),
                inputs: vec![InputRef::new(2, ty.clone())],
                distinct: false,
                filter: Condition::true_cond(),
            },
            PlanAggCall {
                agg_kind: AggKind::Max,
                return_type: ty.clone(),
                inputs: vec![InputRef::new(1, ty.clone())],
                distinct: false,
                filter: Condition::true_cond(),
            },
        ];
        let agg = LogicalAgg::new(agg_calls, vec![1, 2], values.into());

        // Perform the prune
        let required_cols = vec![0, 3];
        let plan = agg.prune_col(&required_cols);
        // Check the result
        let project = plan.as_logical_project().unwrap();
        assert_eq!(project.exprs().len(), 2);
        assert_eq_input_ref!(&project.exprs()[0], 0);
        assert_eq_input_ref!(&project.exprs()[1], 2);

        let agg_new = project.input();
        let agg_new = agg_new.as_logical_agg().unwrap();
        assert_eq!(agg_new.group_keys(), vec![0, 1]);

        assert_eq!(agg_new.agg_calls.len(), 1);
        let agg_call_new = agg_new.agg_calls[0].clone();
        assert_eq!(agg_call_new.agg_kind, AggKind::Max);
        assert_eq!(input_ref_to_column_indices(&agg_call_new.inputs), vec![0]);
        assert_eq!(agg_call_new.return_type, ty);

        let values = agg_new.input();
        let values = values.as_logical_values().unwrap();
        assert_eq!(values.schema().fields(), &fields[1..]);
    }
}
