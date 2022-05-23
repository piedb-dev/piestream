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

//! For expression that only accept two arguments + 1 bytes writer as input.

use risingwave_common::array::{I32Array, Utf8Array};
use risingwave_common::types::DataType;

use crate::expr::template::BinaryBytesExpression;
use crate::expr::BoxedExpression;
use crate::vector_op::substr::*;

pub fn new_substr_start(
    expr_ia1: BoxedExpression,
    expr_ia2: BoxedExpression,
    return_type: DataType,
) -> BoxedExpression {
    Box::new(BinaryBytesExpression::<Utf8Array, I32Array, _>::new(
        expr_ia1,
        expr_ia2,
        return_type,
        substr_start,
    ))
}

#[allow(dead_code)]
pub fn new_substr_for(
    expr_ia1: BoxedExpression,
    expr_ia2: BoxedExpression,
    return_type: DataType,
) -> BoxedExpression {
    Box::new(BinaryBytesExpression::<Utf8Array, I32Array, _>::new(
        expr_ia1,
        expr_ia2,
        return_type,
        substr_for,
    ))
}

#[cfg(test)]
mod tests {
    use risingwave_common::array::DataChunk;
    use risingwave_common::types::{Datum, ScalarImpl};

    use super::*;
    use crate::expr::LiteralExpression;

    fn create_str_i32_binary_expr(
        f: fn(BoxedExpression, BoxedExpression, DataType) -> BoxedExpression,
        str_arg: Datum,
        i32_arg: Datum,
    ) -> BoxedExpression {
        f(
            Box::new(LiteralExpression::new(DataType::Varchar, str_arg)),
            Box::new(LiteralExpression::new(DataType::Int32, i32_arg)),
            DataType::Varchar,
        )
    }

    #[test]
    fn test_substr() {
        let text = "quick brown";
        let start_pos = 3;
        let for_pos = 4;

        let substr_start_normal = create_str_i32_binary_expr(
            new_substr_start,
            Some(ScalarImpl::from(String::from(text))),
            Some(ScalarImpl::Int32(start_pos)),
        );
        let res = substr_start_normal.eval(&DataChunk::new_dummy(1)).unwrap();
        assert_eq!(
            res.to_datum(),
            Some(ScalarImpl::from(String::from(
                &text[start_pos as usize - 1..]
            )))
        );

        let substr_start_i32_none = create_str_i32_binary_expr(
            new_substr_start,
            Some(ScalarImpl::from(String::from(text))),
            None,
        );
        let res = substr_start_i32_none
            .eval(&DataChunk::new_dummy(1))
            .unwrap();
        assert_eq!(res.to_datum(), None);

        let substr_for_normal = create_str_i32_binary_expr(
            new_substr_for,
            Some(ScalarImpl::from(String::from(text))),
            Some(ScalarImpl::Int32(for_pos)),
        );
        let res = substr_for_normal.eval(&DataChunk::new_dummy(1)).unwrap();
        assert_eq!(
            res.to_datum(),
            Some(ScalarImpl::from(String::from(&text[..for_pos as usize])))
        );

        let substr_for_str_none =
            create_str_i32_binary_expr(new_substr_for, None, Some(ScalarImpl::Int32(for_pos)));
        let res = substr_for_str_none.eval(&DataChunk::new_dummy(1)).unwrap();
        assert_eq!(res.to_datum(), None);
    }
}
