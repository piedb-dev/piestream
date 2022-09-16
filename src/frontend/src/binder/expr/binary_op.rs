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

use piestream_common::error::{ErrorCode, Result};
use piestream_common::types::DataType;
use piestream_sqlparser::ast::{BinaryOperator, Expr};

use crate::binder::Binder;
use crate::expr::{Expr as _, ExprImpl, ExprType, FunctionCall};

impl Binder {
    pub(super) fn bind_binary_op(
        &mut self,
        left: Expr,
        op: BinaryOperator,
        right: Expr,
    ) -> Result<ExprImpl> {
        let bound_left = self.bind_expr(left)?;
        let bound_right = self.bind_expr(right)?;
        let func_type = match op {
            BinaryOperator::Plus => ExprType::Add,
            BinaryOperator::Minus => ExprType::Subtract,
            BinaryOperator::Multiply => ExprType::Multiply,
            BinaryOperator::Divide => ExprType::Divide,
            BinaryOperator::Modulo => ExprType::Modulus,
            BinaryOperator::NotEq => ExprType::NotEqual,
            BinaryOperator::Eq => ExprType::Equal,
            BinaryOperator::Lt => ExprType::LessThan,
            BinaryOperator::LtEq => ExprType::LessThanOrEqual,
            BinaryOperator::Gt => ExprType::GreaterThan,
            BinaryOperator::GtEq => ExprType::GreaterThanOrEqual,
            BinaryOperator::And => ExprType::And,
            BinaryOperator::Or => ExprType::Or,
            BinaryOperator::Like => ExprType::Like,
            BinaryOperator::NotLike => return self.bind_not_like(bound_left, bound_right),
            BinaryOperator::BitwiseOr => ExprType::BitwiseOr,
            BinaryOperator::BitwiseAnd => ExprType::BitwiseAnd,
            BinaryOperator::PGBitwiseXor => ExprType::BitwiseXor,
            BinaryOperator::PGBitwiseShiftLeft => ExprType::BitwiseShiftLeft,
            BinaryOperator::PGBitwiseShiftRight => ExprType::BitwiseShiftRight,
            BinaryOperator::Concat => return self.bind_concat_op(bound_left, bound_right),

            _ => {
                return Err(
                    ErrorCode::NotImplemented(format!("binary op: {:?}", op), 112.into()).into(),
                )
            }
        };
        Ok(FunctionCall::new(func_type, vec![bound_left, bound_right])?.into())
    }

    /// Apply a NOT on top of LIKE.
    fn bind_not_like(&mut self, left: ExprImpl, right: ExprImpl) -> Result<ExprImpl> {
        Ok(FunctionCall::new(
            ExprType::Not,
            vec![FunctionCall::new(ExprType::Like, vec![left, right])?.into()],
        )?
        .into())
    }

    /// Bind `||`. Based on the types of the inputs, this can be string concat or array concat.
    fn bind_concat_op(&mut self, left: ExprImpl, right: ExprImpl) -> Result<ExprImpl> {
        let types = [left.return_type(), right.return_type()];

        let has_string = types.iter().any(|t| matches!(t, DataType::Varchar));
        let has_array = types.iter().any(|t| matches!(t, DataType::List { .. }));

        // StringConcat
        if has_string && !has_array {
            Ok(FunctionCall::new(ExprType::ConcatOp, vec![left, right])?.into())
        }
        // ArrayConcat
        else if has_array {
            Err(ErrorCode::NotImplemented("array concat operator".into(), None.into()).into())
        }
        // Invalid types
        else {
            Err(ErrorCode::BindError(format!(
                "operator does not exist: {:?} || {:?}",
                &types[0], &types[1]
            ))
            .into())
        }
    }
}
