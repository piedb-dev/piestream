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

use risingwave_common::error::{ErrorCode, Result};
use risingwave_sqlparser::ast::{BinaryOperator, Expr};

use crate::binder::Binder;
use crate::expr::{ExprImpl, ExprType, FunctionCall};

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
            _ => return Err(ErrorCode::NotImplemented(format!("{:?}", op), 112.into()).into()),
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
}
