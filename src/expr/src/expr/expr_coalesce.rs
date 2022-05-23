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

use risingwave_common::array::{ArrayRef, DataChunk};
use risingwave_common::error::{Result, RwError};
use risingwave_common::types::DataType;
use risingwave_common::{ensure, try_match_expand};
use risingwave_pb::expr::expr_node::{RexNode, Type};
use risingwave_pb::expr::ExprNode;

use crate::expr::{build_from_prost as expr_build_from_prost, BoxedExpression, Expression};

#[derive(Debug)]
pub struct CoalesceExpression {
    return_type: DataType,
    children: Vec<BoxedExpression>,
}

impl Expression for CoalesceExpression {
    fn return_type(&self) -> DataType {
        self.return_type.clone()
    }

    fn eval(&self, input: &DataChunk) -> Result<ArrayRef> {
        let children_array = self
            .children
            .iter()
            .map(|c| c.eval(input))
            .collect::<Result<Vec<_>>>()?;
        let mut builder = self.return_type.create_array_builder(input.cardinality())?;

        let len = children_array[0].len();
        for i in 0..len {
            let mut data = None;
            for array in &children_array {
                let datum = array.datum_at(i);
                if datum.is_some() {
                    data = datum;
                    break;
                }
            }
            builder.append_datum(&data)?;
        }
        Ok(Arc::new(builder.finish()?))
    }
}

impl CoalesceExpression {
    pub fn new(return_type: DataType, children: Vec<BoxedExpression>) -> Self {
        CoalesceExpression {
            return_type,
            children,
        }
    }
}

impl<'a> TryFrom<&'a ExprNode> for CoalesceExpression {
    type Error = RwError;

    fn try_from(prost: &'a ExprNode) -> Result<Self> {
        ensure!(prost.get_expr_type()? == Type::Coalesce);

        let ret_type = DataType::from(prost.get_return_type()?);
        let func_call_node = try_match_expand!(prost.get_rex_node().unwrap(), RexNode::FuncCall)?;

        let children = func_call_node
            .children
            .to_vec()
            .iter()
            .map(expr_build_from_prost)
            .collect::<Result<Vec<_>>>()?;
        Ok(CoalesceExpression::new(ret_type, children))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use risingwave_common::array::column::Column;
    use risingwave_common::array::{DataChunk, PrimitiveArray};
    use risingwave_common::types::ScalarImpl;
    use risingwave_pb::data::data_type::TypeName;
    use risingwave_pb::data::DataType as ProstDataType;
    use risingwave_pb::expr::expr_node::RexNode;
    use risingwave_pb::expr::expr_node::Type::Coalesce;
    use risingwave_pb::expr::{ExprNode, FunctionCall};

    use crate::expr::expr_coalesce::CoalesceExpression;
    use crate::expr::test_utils::make_input_ref;
    use crate::expr::Expression;

    pub fn make_coalesce_function(children: Vec<ExprNode>, ret: TypeName) -> ExprNode {
        ExprNode {
            expr_type: Coalesce as i32,
            return_type: Some(ProstDataType {
                type_name: ret as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::FuncCall(FunctionCall { children })),
        }
    }

    #[test]
    fn test_coalesce_expr() {
        let input_node1 = make_input_ref(0, TypeName::Int32);
        let input_node2 = make_input_ref(1, TypeName::Int32);
        let input_node3 = make_input_ref(2, TypeName::Int32);

        let array = PrimitiveArray::<i32>::from_slice(&[Some(1), None, None, None])
            .map(|x| Arc::new(x.into()))
            .unwrap();
        let col1 = Column::new(array);
        let array = PrimitiveArray::<i32>::from_slice(&[None, Some(2), None, None])
            .map(|x| Arc::new(x.into()))
            .unwrap();
        let col2 = Column::new(array);
        let array = PrimitiveArray::<i32>::from_slice(&[None, None, Some(3), None])
            .map(|x| Arc::new(x.into()))
            .unwrap();
        let col3 = Column::new(array);

        let data_chunk = DataChunk::builder().columns(vec![col1, col2, col3]).build();

        let nullif_expr = CoalesceExpression::try_from(&make_coalesce_function(
            vec![input_node1, input_node2, input_node3],
            TypeName::Int32,
        ))
        .unwrap();
        let res = nullif_expr.eval(&data_chunk).unwrap();
        assert_eq!(res.datum_at(0), Some(ScalarImpl::Int32(1)));
        assert_eq!(res.datum_at(1), Some(ScalarImpl::Int32(2)));
        assert_eq!(res.datum_at(2), Some(ScalarImpl::Int32(3)));
        assert_eq!(res.datum_at(3), None);
    }
}
