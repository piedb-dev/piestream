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

mod agg;
pub mod build_expr_from_prost;
pub mod data_types;
mod expr_array;
mod expr_binary_bytes;
pub mod expr_binary_nonnull;
pub mod expr_binary_nullable;
mod expr_case;
mod expr_coalesce;
mod expr_concat_ws;
mod expr_field;
mod expr_in;
mod expr_input_ref;
mod expr_is_null;
mod expr_literal;
mod expr_ternary_bytes;
pub mod expr_unary;
mod template;

use std::convert::TryFrom;
use std::slice;
use std::sync::Arc;

pub use agg::AggKind;
pub use expr_input_ref::InputRefExpression;
pub use expr_literal::*;
use risingwave_common::array::{ArrayRef, DataChunk, Row};
use risingwave_common::error::ErrorCode::InternalError;
use risingwave_common::error::Result;
use risingwave_common::types::DataType;
use risingwave_pb::expr::ExprNode;

use crate::expr::build_expr_from_prost::*;
use crate::expr::expr_array::ArrayExpression;
use crate::expr::expr_coalesce::CoalesceExpression;
use crate::expr::expr_concat_ws::ConcatWsExpression;
use crate::expr::expr_field::FieldExpression;

pub type ExpressionRef = Arc<dyn Expression>;

/// Instance of an expression
pub trait Expression: std::fmt::Debug + Sync + Send {
    fn return_type(&self) -> DataType;

    /// Evaluate the expression
    ///
    /// # Arguments
    ///
    /// * `input` - input data of the Project Executor
    fn eval(&self, input: &DataChunk) -> Result<ArrayRef>;

    fn boxed(self) -> BoxedExpression
    where
        Self: Sized + Send + 'static,
    {
        Box::new(self)
    }
}

pub type BoxedExpression = Box<dyn Expression>;

pub fn build_from_prost(prost: &ExprNode) -> Result<BoxedExpression> {
    use risingwave_pb::expr::expr_node::Type::*;

    match prost.get_expr_type()? {
        Cast | Upper | Lower | Not | IsTrue | IsNotTrue | IsFalse | IsNotFalse | IsNull
        | IsNotNull | Neg | Ascii | Abs => build_unary_expr_prost(prost),
        Equal | NotEqual | LessThan | LessThanOrEqual | GreaterThan | GreaterThanOrEqual | Add
        | Subtract | Multiply | Divide | Modulus | Extract | RoundDigit | TumbleStart
        | Position => build_binary_expr_prost(prost),
        And | Or | IsDistinctFrom => build_nullable_binary_expr_prost(prost),
        Coalesce => CoalesceExpression::try_from(prost).map(|d| Box::new(d) as BoxedExpression),
        Substr => build_substr_expr(prost),
        Length => build_length_expr(prost),
        Replace => build_replace_expr(prost),
        Like => build_like_expr(prost),
        Trim => build_trim_expr(prost),
        Ltrim => build_ltrim_expr(prost),
        Rtrim => build_rtrim_expr(prost),
        ConcatWs => ConcatWsExpression::try_from(prost).map(|d| Box::new(d) as BoxedExpression),
        ConstantValue => LiteralExpression::try_from(prost).map(|d| Box::new(d) as BoxedExpression),
        InputRef => InputRefExpression::try_from(prost).map(|d| Box::new(d) as BoxedExpression),
        Case => build_case_expr(prost),
        Translate => build_translate_expr(prost),
        In => build_in_expr(prost),
        Field => FieldExpression::try_from(prost).map(|d| Box::new(d) as BoxedExpression),
        Array => ArrayExpression::try_from(prost).map(|d| Box::new(d) as BoxedExpression),
        _ => Err(InternalError(format!(
            "Unsupported expression type: {:?}",
            prost.get_expr_type()
        ))
        .into()),
    }
}

#[derive(Debug)]
/// Simply wrap a row level expression as an array level expression
pub struct RowExpression {
    expr: BoxedExpression,
}

impl RowExpression {
    pub fn new(expr: BoxedExpression) -> Self {
        Self { expr }
    }

    pub fn eval(&mut self, row: &Row, data_types: &[DataType]) -> Result<ArrayRef> {
        let input = DataChunk::from_rows(slice::from_ref(row), data_types)?;
        self.expr.eval(&input)
    }

    pub fn return_type(&self) -> DataType {
        self.expr.return_type()
    }
}

#[cfg(test)]
mod test_utils;
