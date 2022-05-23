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
use std::sync::Arc;

use risingwave_common::array::{
    Array, ArrayBuilder, ArrayImpl, ArrayRef, DataChunk, Utf8ArrayBuilder,
};
use risingwave_common::error::{Result, RwError};
use risingwave_common::types::DataType;
use risingwave_common::{ensure, try_match_expand};
use risingwave_pb::expr::expr_node::{RexNode, Type};
use risingwave_pb::expr::ExprNode;

use crate::expr::{build_from_prost as expr_build_from_prost, BoxedExpression, Expression};

#[derive(Debug)]
pub struct ConcatWsExpression {
    return_type: DataType,
    sep_expr: BoxedExpression,
    string_exprs: Vec<BoxedExpression>,
}

impl Expression for ConcatWsExpression {
    fn return_type(&self) -> DataType {
        self.return_type.clone()
    }

    fn eval(&self, input: &DataChunk) -> Result<ArrayRef> {
        let sep_column = self.sep_expr.eval(input)?;
        let sep_column = sep_column.as_utf8();

        let string_columns = self
            .string_exprs
            .iter()
            .map(|c| c.eval(input))
            .collect::<Result<Vec<_>>>()?;
        let string_columns_ref = string_columns
            .iter()
            .map(|c| c.as_utf8())
            .collect::<Vec<_>>();

        let row_len = input.cardinality();
        let mut builder = Utf8ArrayBuilder::new(row_len)?;

        for row_idx in 0..row_len {
            let sep = match sep_column.value_at(row_idx) {
                Some(sep) => sep,
                None => {
                    builder.append(None)?;
                    continue;
                }
            };

            let mut writer = builder.writer().begin();

            let mut string_columns = string_columns_ref.iter();
            for string_column in string_columns.by_ref() {
                if let Some(string) = string_column.value_at(row_idx) {
                    writer.write_ref(string)?;
                    break;
                }
            }

            for string_column in string_columns {
                if let Some(string) = string_column.value_at(row_idx) {
                    writer.write_ref(sep)?;
                    writer.write_ref(string)?;
                }
            }

            builder = writer.finish()?.into_inner();
        }
        Ok(Arc::new(ArrayImpl::from(builder.finish()?)))
    }
}

impl ConcatWsExpression {
    pub fn new(
        return_type: DataType,
        sep_expr: BoxedExpression,
        string_exprs: Vec<BoxedExpression>,
    ) -> Self {
        ConcatWsExpression {
            return_type,
            sep_expr,
            string_exprs,
        }
    }
}

impl<'a> TryFrom<&'a ExprNode> for ConcatWsExpression {
    type Error = RwError;

    fn try_from(prost: &'a ExprNode) -> Result<Self> {
        ensure!(prost.get_expr_type()? == Type::ConcatWs);

        let ret_type = DataType::from(prost.get_return_type()?);
        let func_call_node = try_match_expand!(prost.get_rex_node().unwrap(), RexNode::FuncCall)?;

        let children = &func_call_node.children;
        let sep_expr = expr_build_from_prost(&children[0])?;

        let string_exprs = children[1..]
            .iter()
            .map(expr_build_from_prost)
            .collect::<Result<Vec<_>>>()?;
        Ok(ConcatWsExpression::new(ret_type, sep_expr, string_exprs))
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use risingwave_common::array::{DataChunk, DataChunkTestExt};
    use risingwave_pb::data::data_type::TypeName;
    use risingwave_pb::data::DataType as ProstDataType;
    use risingwave_pb::expr::expr_node::RexNode;
    use risingwave_pb::expr::expr_node::Type::ConcatWs;
    use risingwave_pb::expr::{ExprNode, FunctionCall};

    use crate::expr::expr_concat_ws::ConcatWsExpression;
    use crate::expr::test_utils::make_input_ref;
    use crate::expr::Expression;

    pub fn make_concat_ws_function(children: Vec<ExprNode>, ret: TypeName) -> ExprNode {
        ExprNode {
            expr_type: ConcatWs as i32,
            return_type: Some(ProstDataType {
                type_name: ret as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::FuncCall(FunctionCall { children })),
        }
    }

    #[test]
    fn test_eval_concat_ws_expr() {
        let input_node1 = make_input_ref(0, TypeName::Varchar);
        let input_node2 = make_input_ref(1, TypeName::Varchar);
        let input_node3 = make_input_ref(2, TypeName::Varchar);
        let input_node4 = make_input_ref(3, TypeName::Varchar);
        let concat_ws_expr = ConcatWsExpression::try_from(&make_concat_ws_function(
            vec![input_node1, input_node2, input_node3, input_node4],
            TypeName::Varchar,
        ))
        .unwrap();

        let chunk = DataChunk::from_pretty(
            "
            T T T T
            , a b c
            . a b c
            , . b c
            , . . .
            . . . .",
        );

        let actual = concat_ws_expr.eval(&chunk).unwrap();
        let actual = actual
            .iter()
            .map(|r| r.map(|s| s.into_utf8()))
            .collect_vec();

        let expected = vec![Some("a,b,c"), None, Some("b,c"), Some(""), None];

        assert_eq!(actual, expected);
    }
}
