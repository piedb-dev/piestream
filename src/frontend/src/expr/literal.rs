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

use piestream_common::array::Row;
use piestream_common::types::{DataType, Datum, ScalarImpl};
use piestream_expr::expr::expr_unary::new_unary_expr;
use piestream_expr::expr::{Expression, LiteralExpression};
use piestream_pb::expr::expr_node::RexNode;

use super::Expr;
use crate::expr::ExprType;
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Literal {
    data: Datum,
    data_type: DataType,
}

impl std::fmt::Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            f.debug_struct("Literal")
                .field("data", &self.data)
                .field("data_type", &self.data_type)
                .finish()
        } else {
            match &self.data {
                None => write!(f, "null"),
                // Add single quotation marks for string and interval literals
                Some(ScalarImpl::Utf8(v)) => write!(f, "'{}'", v),
                Some(ScalarImpl::Interval(v)) => write!(f, "'{}'", v),
                Some(v) => write!(f, "{}", v),
            }?;
            write!(f, ":{:?}", self.data_type)
        }
    }
}

impl Literal {
    pub fn new(data: Datum, data_type: DataType) -> Self {
        Literal { data, data_type }
    }

    pub fn get_expr_type(&self) -> ExprType {
        ExprType::ConstantValue
    }

    pub fn get_data(&self) -> &Datum {
        &self.data
    }

    pub fn is_null(&self) -> bool {
        self.data.is_none()
    }

    /// This is a temporary workaround.
    ///
    /// TODO: evaluate expression in frontend.
    /// Tracking issue <https://github.com/singularity-data/piestream/issues/3479>
    pub fn eval_as(&self, return_type: DataType) -> ScalarImpl {
        assert!(self.data.is_some());
        let lit = LiteralExpression::try_from(&self.to_expr_proto()).unwrap();
        let mut const_expr = lit.boxed();
        if const_expr.return_type() != return_type {
            const_expr = new_unary_expr(ExprType::Cast, return_type, const_expr).unwrap();
        }
        const_expr.eval_row(Row::empty()).unwrap().unwrap()
    }
}

impl Expr for Literal {
    fn return_type(&self) -> DataType {
        self.data_type.clone()
    }

    fn to_expr_proto(&self) -> piestream_pb::expr::ExprNode {
        use piestream_pb::expr::*;
        ExprNode {
            expr_type: self.get_expr_type() as i32,
            return_type: Some(self.return_type().to_protobuf()),
            rex_node: literal_to_protobuf(self.get_data()),
        }
    }
}

/// Convert a literal value (datum) into protobuf.
fn literal_to_protobuf(d: &Datum) -> Option<RexNode> {
    let Some(d) = d.as_ref() else {
        return None;
    };
    use piestream_pb::expr::*;
    let body = d.to_protobuf();
    Some(RexNode::Constant(ConstantValue { body }))
}

#[cfg(test)]
mod tests {
    use piestream_common::array::{ListValue, StructValue};
    use piestream_common::types::{DataType, ScalarImpl};
    use piestream_pb::expr::expr_node::RexNode;

    use crate::expr::literal::literal_to_protobuf;

    #[test]
    fn test_struct_to_protobuf() {
        let value = StructValue::new(vec![
            Some(ScalarImpl::Utf8("12222".to_string())),
            Some(2.into()),
            Some(3.into()),
        ]);
        let data = Some(ScalarImpl::Struct(value.clone()));
        let node = literal_to_protobuf(&data);
        if let RexNode::Constant(prost) = node.as_ref().unwrap() {
            let data2 = ScalarImpl::bytes_to_scalar(
                prost.get_body(),
                &DataType::Struct {
                    fields: vec![DataType::Varchar, DataType::Int32, DataType::Int32].into(),
                }
                .to_protobuf(),
            )
            .unwrap();
            assert_eq!(ScalarImpl::Struct(value), data2);
        }
    }

    #[test]
    fn test_list_to_protobuf() {
        let value = ListValue::new(vec![Some(1.into()), Some(2.into()), Some(3.into())]);
        let data = Some(ScalarImpl::List(value.clone()));
        let node = literal_to_protobuf(&data);
        if let RexNode::Constant(prost) = node.as_ref().unwrap() {
            let data2 = ScalarImpl::bytes_to_scalar(
                prost.get_body(),
                &DataType::List {
                    datatype: Box::new(DataType::Int32),
                }
                .to_protobuf(),
            )
            .unwrap();
            assert_eq!(ScalarImpl::List(value), data2);
        }
    }
}
