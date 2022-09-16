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

use std::convert::TryFrom;
use std::ops::Index;

use piestream_common::array::{ArrayRef, DataChunk, Row};
use piestream_common::types::{DataType, Datum};
use piestream_pb::expr::expr_node::{RexNode, Type};
use piestream_pb::expr::ExprNode;

use crate::expr::Expression;
use crate::{bail, ensure, ExprError, Result};

/// `InputRefExpression` references to a column in input relation
#[derive(Debug)]
pub struct InputRefExpression {
    return_type: DataType,
    idx: usize,
}

impl Expression for InputRefExpression {
    fn return_type(&self) -> DataType {
        self.return_type.clone()
    }

    fn eval(&self, input: &DataChunk) -> Result<ArrayRef> {
        Ok(input.column_at(self.idx).array())
    }

    fn eval_row(&self, input: &Row) -> Result<Datum> {
        let cell = input.index(self.idx).as_ref().cloned();
        Ok(cell)
    }
}

impl InputRefExpression {
    pub fn new(return_type: DataType, idx: usize) -> Self {
        InputRefExpression { return_type, idx }
    }

    pub fn eval_immut(&self, input: &DataChunk) -> Result<ArrayRef> {
        Ok(input.column_at(self.idx).array())
    }
}

impl<'a> TryFrom<&'a ExprNode> for InputRefExpression {
    type Error = ExprError;

    fn try_from(prost: &'a ExprNode) -> Result<Self> {
        ensure!(prost.get_expr_type().unwrap() == Type::InputRef);

        let ret_type = DataType::from(prost.get_return_type().unwrap());
        if let RexNode::InputRef(input_ref_node) = prost.get_rex_node().unwrap() {
            Ok(Self {
                return_type: ret_type,
                idx: input_ref_node.column_idx as usize,
            })
        } else {
            bail!("Expect an input ref node")
        }
    }
}

#[cfg(test)]
mod tests {
    use piestream_common::array::Row;
    use piestream_common::types::{DataType, Datum};

    use crate::expr::{Expression, InputRefExpression};

    #[test]
    fn test_eval_row_input_ref() {
        let datums: Vec<Datum> = vec![Some(1.into()), Some(2.into()), None];
        let input_row = Row::new(datums.clone());

        for (i, expected) in datums.iter().enumerate() {
            let expr = InputRefExpression::new(DataType::Int32, i);
            let result = expr.eval_row(&input_row).unwrap();
            assert_eq!(*expected, result);
        }
    }
}
