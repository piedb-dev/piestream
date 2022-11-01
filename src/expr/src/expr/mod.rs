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

mod agg;
pub mod build_expr_from_prost;
pub mod data_types;
mod expr_array_concat;
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
mod expr_nested_construct;
mod expr_quaternary_bytes;
mod expr_regexp;
mod expr_ternary_bytes;
mod expr_to_char_const_tmpl;
pub mod expr_unary;
mod expr_vnode;
mod template;

use std::convert::TryFrom;
use std::sync::Arc;

pub use agg::AggKind;
pub use expr_input_ref::InputRefExpression;
pub use expr_literal::*;
use piestream_common::array::{ArrayRef, DataChunk, Row};
use piestream_common::types::{DataType, Datum};
use piestream_pb::expr::ExprNode;

use super::Result;
use crate::expr::build_expr_from_prost::*;
use crate::expr::expr_array_concat::ArrayConcatExpression;
use crate::expr::expr_case::CaseExpression;
use crate::expr::expr_coalesce::CoalesceExpression;
use crate::expr::expr_concat_ws::ConcatWsExpression;
use crate::expr::expr_field::FieldExpression;
use crate::expr::expr_in::InExpression;
use crate::expr::expr_nested_construct::NestedConstructExpression;
use crate::expr::expr_regexp::RegexpMatchExpression;
use crate::expr::expr_vnode::VnodeExpression;
use crate::ExprError;

pub type ExpressionRef = Arc<dyn Expression>;

/// Instance of an expression
pub trait Expression: std::fmt::Debug + Sync + Send {
    fn return_type(&self) -> DataType;

    /// Eval the result with extra checks.
    fn eval_checked(&self, input: &DataChunk) -> Result<ArrayRef> {
        let res = self.eval(input)?;

        // TODO: Decide to use assert or debug_assert by benchmarks.
        assert_eq!(res.len(), input.capacity());

        Ok(res)
    }

    /// Evaluate the expression
    ///
    /// # Arguments
    ///
    /// * `input` - input data of the Project Executor
    fn eval(&self, input: &DataChunk) -> Result<ArrayRef>;

    /// Evaluate the expression in row-based execution.
    fn eval_row(&self, input: &Row) -> Result<Datum>;

    fn boxed(self) -> BoxedExpression
    where
        Self: Sized + Send + 'static,
    {
        Box::new(self)
    }
}

pub type BoxedExpression = Box<dyn Expression>;

pub fn build_from_prost(prost: &ExprNode) -> Result<BoxedExpression> {
    use piestream_pb::expr::expr_node::Type::*;

    match prost.get_expr_type().unwrap() {
        // Fixed number of arguments and based on `Unary/Binary/Ternary/...Expression`
        Cast | Upper | Lower | Md5 | Not | IsTrue | IsNotTrue | IsFalse | IsNotFalse | IsNull
        | IsNotNull | Neg | Ascii | Abs | Ceil | Floor | Round | BitwiseNot | CharLength
        | BoolOut | OctetLength | BitLength => build_unary_expr_prost(prost),
        Equal | NotEqual | LessThan | LessThanOrEqual | GreaterThan | GreaterThanOrEqual | Add
        | Subtract | Multiply | Divide | Modulus | Extract | RoundDigit | TumbleStart
        | Position | BitwiseShiftLeft | BitwiseShiftRight | BitwiseAnd | BitwiseOr | BitwiseXor
        | ConcatOp => build_binary_expr_prost(prost),
        And | Or | IsDistinctFrom | IsNotDistinctFrom | ArrayAccess => {
            build_nullable_binary_expr_prost(prost)
        }
        ToChar => build_to_char_expr(prost),
        Length => build_length_expr(prost),
        Replace => build_replace_expr(prost),
        Like => build_like_expr(prost),
        Repeat => build_repeat_expr(prost),
        SplitPart => build_split_part_expr(prost),
        Translate => build_translate_expr(prost),

        // Variable number of arguments and based on `Unary/Binary/Ternary/...Expression`
        Substr => build_substr_expr(prost),
        Overlay => build_overlay_expr(prost),
        Trim => build_trim_expr(prost),
        Ltrim => build_ltrim_expr(prost),
        Rtrim => build_rtrim_expr(prost),

        // Dedicated types
        In => InExpression::try_from(prost).map(Expression::boxed),
        Case => CaseExpression::try_from(prost).map(Expression::boxed),
        Coalesce => CoalesceExpression::try_from(prost).map(Expression::boxed),
        ConcatWs => ConcatWsExpression::try_from(prost).map(Expression::boxed),
        ConstantValue => LiteralExpression::try_from(prost).map(Expression::boxed),
        InputRef => InputRefExpression::try_from(prost).map(Expression::boxed),
        Field => FieldExpression::try_from(prost).map(Expression::boxed),
        Array => NestedConstructExpression::try_from(prost).map(Expression::boxed),
        Row => NestedConstructExpression::try_from(prost).map(Expression::boxed),
        RegexpMatch => RegexpMatchExpression::try_from(prost).map(Expression::boxed),
        ArrayCat | ArrayAppend | ArrayPrepend => {
            // Now we implement these three functions as a single expression for the
            // sake of simplicity. If performance matters at some time, we can split
            // the implementation to improve performance.
            ArrayConcatExpression::try_from(prost).map(Expression::boxed)
        }
        Vnode => VnodeExpression::try_from(prost).map(Expression::boxed),
        _ => Err(ExprError::UnsupportedFunction(format!(
            "{:?}",
            prost.get_expr_type()
        ))),
    }
}

mod test_utils;
pub use test_utils::*;
