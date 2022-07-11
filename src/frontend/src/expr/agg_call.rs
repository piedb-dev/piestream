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

use itertools::Itertools;
use piestream_common::error::{ErrorCode, Result, RwError};
use piestream_common::types::DataType;
use piestream_expr::expr::AggKind;

use super::{Expr, ExprImpl};
use crate::utils::Condition;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct AggCall {
    agg_kind: AggKind,
    return_type: DataType,
    inputs: Vec<ExprImpl>,
    distinct: bool,
    filter: Condition,
}

impl std::fmt::Debug for AggCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            f.debug_struct("AggCall")
                .field("agg_kind", &self.agg_kind)
                .field("return_type", &self.return_type)
                .field("inputs", &self.inputs)
                .field("filter", &self.filter)
                .finish()
        } else {
            let mut builder = f.debug_tuple(&format!("{}", self.agg_kind));
            self.inputs.iter().for_each(|child| {
                builder.field(child);
            });
            builder.finish()
        }
    }
}

impl AggCall {
    /// Infer the return type for the given agg call.
    /// Returns error if not supported or the arguments are invalid.
    pub fn infer_return_type(agg_kind: &AggKind, inputs: &[DataType]) -> Result<DataType> {
        let unsupported = || {
            let args = inputs.iter().map(|t| format!("{:?}", t)).join(", ");
            Err(RwError::from(ErrorCode::NotImplemented(
                format!("Unsupported aggregation: {}({})", agg_kind, args),
                112.into(),
            )))
        };
        let invalid = || {
            let args = inputs.iter().map(|t| format!("{:?}", t)).join(", ");
            Err(RwError::from(ErrorCode::InvalidInputSyntax(format!(
                "Invalid aggregation: {}({})",
                agg_kind, args
            ))))
        };

        // The function signatures are aligned with postgres, see
        // https://www.postgresql.org/docs/current/functions-aggregate.html.
        let return_type = match (&agg_kind, inputs) {
            // Min, Max
            (AggKind::Min | AggKind::Max, [input]) => input.clone(),
            (AggKind::Min | AggKind::Max, _) => return invalid(),

            // Avg
            (AggKind::Avg, [input]) => match input {
                DataType::Int16 | DataType::Int32 | DataType::Int64 | DataType::Decimal => {
                    DataType::Decimal
                }
                DataType::Float32 | DataType::Float64 => DataType::Float64,
                DataType::Interval => DataType::Interval,
                _ => return invalid(),
            },
            (AggKind::Avg, _) => return invalid(),

            // Sum
            (AggKind::Sum, [input]) => match input {
                DataType::Int16 => DataType::Int64,
                DataType::Int32 => DataType::Int64,
                DataType::Int64 => DataType::Decimal,
                DataType::Decimal => DataType::Decimal,
                DataType::Float32 => DataType::Float32,
                DataType::Float64 => DataType::Float64,
                DataType::Interval => DataType::Interval,
                _ => return invalid(),
            },
            (AggKind::Sum, _) => return invalid(),

            // Count
            (AggKind::Count | AggKind::ApproxCountDistinct, _) => DataType::Int64,

            // StringAgg
            (AggKind::StringAgg, _) => DataType::Varchar,

            // SingleValue
            (AggKind::SingleValue, [input]) => input.clone(),
            (AggKind::SingleValue, _) => return invalid(),

            // Others
            _ => return unsupported(),
        };

        Ok(return_type)
    }

    /// Returns error if the function name matches with an existing function
    /// but with illegal arguments.
    pub fn new(
        agg_kind: AggKind,
        inputs: Vec<ExprImpl>,
        distinct: bool,
        filter: Condition,
    ) -> Result<Self> {
        let data_types = inputs.iter().map(ExprImpl::return_type).collect_vec();
        let return_type = Self::infer_return_type(&agg_kind, &data_types)?;
        Ok(AggCall {
            agg_kind,
            return_type,
            inputs,
            distinct,
            filter,
        })
    }

    pub fn decompose(self) -> (AggKind, Vec<ExprImpl>, bool, Condition) {
        (self.agg_kind, self.inputs, self.distinct, self.filter)
    }

    pub fn agg_kind(&self) -> AggKind {
        self.agg_kind.clone()
    }

    /// Get a reference to the agg call's inputs.
    pub fn inputs(&self) -> &[ExprImpl] {
        self.inputs.as_ref()
    }
}

impl Expr for AggCall {
    fn return_type(&self) -> DataType {
        self.return_type.clone()
    }

    fn to_expr_proto(&self) -> piestream_pb::expr::ExprNode {
        // This function is always called on the physical planning step, where
        // `ExprImpl::AggCall` must have been rewritten to aggregate operators.

        unreachable!(
            "AggCall {:?} has not been rewritten to physical aggregate operators",
            self
        )
    }
}