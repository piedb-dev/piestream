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

use std::convert::TryFrom;

use risingwave_common::error::{ErrorCode, Result, RwError};
use risingwave_pb::expr::agg_call::Type;

/// Kind of aggregation function
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AggKind {
    Min,
    Max,
    Sum,
    Count,
    RowCount,
    Avg,
    StringAgg,
    SingleValue,
}

impl std::fmt::Display for AggKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AggKind::Min => write!(f, "min"),
            AggKind::Max => write!(f, "max"),
            AggKind::Sum => write!(f, "sum"),
            AggKind::Count => write!(f, "count"),
            AggKind::RowCount => write!(f, "row_count"),
            AggKind::Avg => write!(f, "avg"),
            AggKind::StringAgg => write!(f, "string_agg"),
            AggKind::SingleValue => write!(f, "single_value"),
        }
    }
}

impl TryFrom<Type> for AggKind {
    type Error = RwError;

    fn try_from(prost: Type) -> Result<Self> {
        match prost {
            Type::Min => Ok(AggKind::Min),
            Type::Max => Ok(AggKind::Max),
            Type::Sum => Ok(AggKind::Sum),
            Type::Avg => Ok(AggKind::Avg),
            Type::Count => Ok(AggKind::Count),
            Type::StringAgg => Ok(AggKind::StringAgg),
            Type::SingleValue => Ok(AggKind::SingleValue),
            _ => Err(ErrorCode::InternalError("Unrecognized agg.".into()).into()),
        }
    }
}

impl AggKind {
    pub fn to_prost(&self) -> Type {
        match self {
            Self::Min => Type::Min,
            Self::Max => Type::Max,
            Self::Sum => Type::Sum,
            Self::Avg => Type::Avg,
            Self::Count => Type::Count,
            Self::StringAgg => Type::StringAgg,
            Self::SingleValue => Type::SingleValue,
            Self::RowCount => {
                panic!("cannot convert RowCount to prost, TODO: remove RowCount from AggKind")
            }
        }
    }
}
