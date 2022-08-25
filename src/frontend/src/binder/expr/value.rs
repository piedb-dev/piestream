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
use piestream_common::types::{DataType, Decimal, IntervalUnit, ScalarImpl};
use piestream_expr::vector_op::cast::str_parse;
use piestream_sqlparser::ast::{DateTimeField, Expr, Value};

use crate::binder::Binder;
use crate::expr::{align_types, Expr as _, ExprImpl, ExprType, FunctionCall, Literal};

impl Binder {
    //获取sql输入值具体类型
    pub fn bind_value(&mut self, value: Value) -> Result<Literal> {
        match value {
            Value::Number(s, b) => self.bind_number(s, b),
            Value::SingleQuotedString(s) => self.bind_string(s),
            Value::Boolean(b) => self.bind_bool(b),
            // Both null and string literal will be treated as `unknown` during type inference.
            // See [`ExprImpl::is_unknown`].
            Value::Null => Ok(Literal::new(None, DataType::Varchar)),
            Value::Interval {
                value,
                leading_field,
                // TODO: support more interval types.
                leading_precision: None,
                last_field: None,
                fractional_seconds_precision: None,
            } => self.bind_interval(value, leading_field),
            _ => Err(ErrorCode::NotImplemented(format!("value: {:?}", value), None.into()).into()),
        }
    }

    pub(super) fn bind_string(&mut self, s: String) -> Result<Literal> {
        Ok(Literal::new(Some(ScalarImpl::Utf8(s)), DataType::Varchar))
    }

    fn bind_bool(&mut self, b: bool) -> Result<Literal> {
        Ok(Literal::new(Some(ScalarImpl::Bool(b)), DataType::Boolean))
    }

    fn bind_number(&mut self, s: String, _b: bool) -> Result<Literal> {
        let (data, data_type) = if let Ok(int_32) = s.parse::<i32>() {
            (Some(ScalarImpl::Int32(int_32)), DataType::Int32)
        } else if let Ok(int_64) = s.parse::<i64>() {
            (Some(ScalarImpl::Int64(int_64)), DataType::Int64)
        } else {
            // Notice: when the length of decimal exceeds 29(>= 30), it will be rounded up.
            let decimal = str_parse::<Decimal>(&s)?;
            (Some(ScalarImpl::Decimal(decimal)), DataType::Decimal)
        };
        Ok(Literal::new(data, data_type))
    }

    fn bind_interval(
        &mut self,
        s: String,
        leading_field: Option<DateTimeField>,
    ) -> Result<Literal> {
        // > INTERVAL '1' means 1 second.
        // https://www.postgresql.org/docs/current/datatype-datetime.html#DATATYPE-INTERVAL-INPUT
        let unit = leading_field.unwrap_or(DateTimeField::Second);
        use DateTimeField::*;
        let tokens = parse_interval(&s)?;
        // Todo: support more syntax
        if tokens.len() > 2 {
            return Err(ErrorCode::InvalidInputSyntax(format!("Invalid interval {}.", &s)).into());
        }
        let num = match tokens.get(0) {
            Some(TimeStrToken::Num(num)) => *num,
            _ => {
                return Err(
                    ErrorCode::InvalidInputSyntax(format!("Invalid interval {}.", &s)).into(),
                );
            }
        };
        let interval_unit = match tokens.get(1) {
            Some(TimeStrToken::TimeUnit(unit)) => unit,
            _ => &unit,
        };

        let interval = (|| match interval_unit {
            Year => {
                let months = num.checked_mul(12)?;
                Some(IntervalUnit::from_month(months as i32))
            }
            Month => Some(IntervalUnit::from_month(num as i32)),
            Day => Some(IntervalUnit::from_days(num as i32)),
            Hour => {
                let ms = num.checked_mul(3600 * 1000)?;
                Some(IntervalUnit::from_millis(ms))
            }
            Minute => {
                let ms = num.checked_mul(60 * 1000)?;
                Some(IntervalUnit::from_millis(ms))
            }
            Second => {
                let ms = num.checked_mul(1000)?;
                Some(IntervalUnit::from_millis(ms))
            }
        })()
        .ok_or_else(|| {
            RwError::from(ErrorCode::InvalidInputSyntax(format!(
                "Invalid interval {}.",
                s
            )))
        })?;

        let datum = Some(ScalarImpl::Interval(interval));
        let literal = Literal::new(datum, DataType::Interval);

        Ok(literal)
    }

    /// `ARRAY[...]` is represented as an function call at the binder stage.
    pub(super) fn bind_array(&mut self, exprs: Vec<Expr>) -> Result<ExprImpl> {
        let mut exprs = exprs
            .into_iter()
            .map(|e| self.bind_expr(e))
            .collect::<Result<Vec<ExprImpl>>>()?;
        let element_type = align_types(exprs.iter_mut())?;
        let expr: ExprImpl = FunctionCall::new_unchecked(
            ExprType::Array,
            exprs,
            DataType::List {
                datatype: Box::new(element_type),
            },
        )
        .into();
        Ok(expr)
    }

    pub(super) fn bind_array_index(&mut self, obj: Expr, indexs: Vec<Expr>) -> Result<ExprImpl> {
        let obj = self.bind_expr(obj)?;
        match obj.return_type() {
            DataType::List {
                datatype: return_type,
            } => {
                let mut indexs = indexs
                    .into_iter()
                    .map(|e| self.bind_expr(e))
                    .collect::<Result<Vec<ExprImpl>>>()?;
                indexs.insert(0, obj);

                let expr: ExprImpl =
                    FunctionCall::new_unchecked(ExprType::ArrayAccess, indexs, *return_type).into();
                Ok(expr)
            }
            _ => panic!("Should be a List"),
        }
    }

    /// `Row(...)` is represented as an function call at the binder stage.
    pub(super) fn bind_row(&mut self, exprs: Vec<Expr>) -> Result<ExprImpl> {
        let exprs = exprs
            .into_iter()
            .map(|e| self.bind_expr(e))
            .collect::<Result<Vec<ExprImpl>>>()?;
        let data_type = DataType::Struct {
            fields: exprs.iter().map(|e| e.return_type()).collect_vec().into(),
        };
        let expr: ExprImpl = FunctionCall::new_unchecked(ExprType::Row, exprs, data_type).into();
        Ok(expr)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TimeStrToken {
    Num(i64),
    TimeUnit(DateTimeField),
}

fn convert_digit(c: &mut String, t: &mut Vec<TimeStrToken>) -> Result<()> {
    if !c.is_empty() {
        match c.parse::<i64>() {
            Ok(num) => {
                t.push(TimeStrToken::Num(num));
            }
            Err(_) => {
                return Err(
                    ErrorCode::InvalidInputSyntax(format!("Invalid interval: {}", c)).into(),
                );
            }
        }
        c.clear();
    }
    Ok(())
}

fn convert_unit(c: &mut String, t: &mut Vec<TimeStrToken>) -> Result<()> {
    if !c.is_empty() {
        t.push(TimeStrToken::TimeUnit(c.parse()?));
        c.clear();
    }
    Ok(())
}

pub fn parse_interval(s: &str) -> Result<Vec<TimeStrToken>> {
    let s = s.trim();
    let mut tokens = Vec::new();
    let mut num_buf = "".to_string();
    let mut char_buf = "".to_string();
    for (i, c) in s.chars().enumerate() {
        match c {
            '-' => {
                num_buf.push(c);
            }
            c if c.is_ascii_digit() => {
                convert_unit(&mut char_buf, &mut tokens)?;
                num_buf.push(c);
            }
            c if c.is_ascii_alphabetic() => {
                convert_digit(&mut num_buf, &mut tokens)?;
                char_buf.push(c);
            }
            chr if chr.is_ascii_whitespace() => {
                convert_unit(&mut char_buf, &mut tokens)?;
                convert_digit(&mut num_buf, &mut tokens)?;
            }
            _ => {
                return Err(ErrorCode::InvalidInputSyntax(format!(
                    "Invalid character at offset {} in {}: {:?}. Only support digit or alphabetic now",
                    i, s, c
                )).into());
            }
        }
    }
    convert_digit(&mut num_buf, &mut tokens)?;
    convert_unit(&mut char_buf, &mut tokens)?;

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use piestream_common::types::DataType;
    use piestream_expr::expr::build_from_prost;

    use crate::binder::test_utils::mock_binder;
    use crate::expr::{Expr, ExprImpl, ExprType, FunctionCall};

    #[test]
    fn test_bind_value() {
        use std::str::FromStr;

        use super::*;

        let mut binder = mock_binder();
        let values = vec![
            "1",
            "111111111111111",
            "111111111.111111",
            "111111111111111111111111",
            "0.111111",
            "-0.01",
        ];
        let data = vec![
            Some(ScalarImpl::Int32(1)),
            Some(ScalarImpl::Int64(111111111111111)),
            Some(ScalarImpl::Decimal(
                Decimal::from_str("111111111.111111").unwrap(),
            )),
            Some(ScalarImpl::Decimal(
                Decimal::from_str("111111111111111111111111").unwrap(),
            )),
            Some(ScalarImpl::Decimal(Decimal::from_str("0.111111").unwrap())),
            Some(ScalarImpl::Decimal(Decimal::from_str("-0.01").unwrap())),
        ];
        let data_type = vec![
            DataType::Int32,
            DataType::Int64,
            DataType::Decimal,
            DataType::Decimal,
            DataType::Decimal,
            DataType::Decimal,
        ];

        //Value::Number是开源库sqlparse字段类型
        for i in 0..values.len() {
            let value = Value::Number(String::from(values[i]), false);
            let res = binder.bind_value(value).unwrap();
            println!("res={:?}", res);
            let ans = Literal::new(data[i].clone(), data_type[i].clone());
            assert_eq!(res, ans);
        }
    }

    #[test]
    fn test_array_expr() {
        /*
        select * from t1 where v1 < 3;
        where_clause: Some(
                    FunctionCall(
                        FunctionCall {
                            func_type: LessThan,
                            return_type: Boolean,
                            inputs: [
                                InputRef(
                                    InputRef {
                                        index: 1,
                                        data_type: Int32,
                                    },
                                ),
                                Literal(
                                    Literal {
                                        data: Some(
                                            Int32(
                                                3,
                                            ),
                                        ),
                                        data_type: Int32,
                                    },
                                ),
                            ],
                        },
                    ),
                ),
         */
        let expr: ExprImpl = FunctionCall::new_unchecked(
            ExprType::Array,
            vec![ExprImpl::literal_int(11)],
            DataType::List {
                datatype: Box::new(DataType::Int32),
            },
        )
        .into();
        //to_expr_proto函数src/frontend/src/expr/input_ref.rs
        let expr_pb = expr.to_expr_proto();
        let expr = build_from_prost(&expr_pb).unwrap();
        println!("expr:{:?}", expr);
        match expr.return_type() {
            DataType::List { datatype } => {
                assert_eq!(datatype, Box::new(DataType::Int32));
            }
            _ => panic!("unexpected type"),
        };
    }

    #[test]
    fn test_array_index_expr() {
        let array_expr = FunctionCall::new_unchecked(
            ExprType::Array,
            vec![ExprImpl::literal_int(11), ExprImpl::literal_int(22)],
            DataType::List {
                datatype: Box::new(DataType::Int32),
            },
        )
        .into();

        let expr: ExprImpl = FunctionCall::new_unchecked(
            ExprType::ArrayAccess,
            vec![array_expr, ExprImpl::literal_int(1)],
            DataType::Int32,
        )
        .into();

        let expr_pb = expr.to_expr_proto();
        let expr = build_from_prost(&expr_pb).unwrap();
        assert_eq!(expr.return_type(), DataType::Int32);
    }

    #[test]
    fn test_bind_interval() {
        use super::*;

        let mut binder = mock_binder();
        let values = vec![
            "1 hour",
            "1 h",
            "1 year",
            "6 second",
            "2 minutes",
            "1 month",
        ];
        let data = vec![
            Ok(Literal::new(
                Some(ScalarImpl::Interval(IntervalUnit::from_minutes(60))),
                DataType::Interval,
            )),
            Ok(Literal::new(
                Some(ScalarImpl::Interval(IntervalUnit::from_minutes(60))),
                DataType::Interval,
            )),
            Ok(Literal::new(
                Some(ScalarImpl::Interval(IntervalUnit::from_ymd(1, 0, 0))),
                DataType::Interval,
            )),
            Ok(Literal::new(
                Some(ScalarImpl::Interval(IntervalUnit::from_millis(6 * 1000))),
                DataType::Interval,
            )),
            Ok(Literal::new(
                Some(ScalarImpl::Interval(IntervalUnit::from_minutes(2))),
                DataType::Interval,
            )),
            Ok(Literal::new(
                Some(ScalarImpl::Interval(IntervalUnit::from_month(1))),
                DataType::Interval,
            )),
        ];

        for i in 0..values.len() {
            let value = Value::Interval {
                value: values[i].to_string(),
                leading_field: None,
                leading_precision: None,
                last_field: None,
                fractional_seconds_precision: None,
            };
            assert_eq!(binder.bind_value(value), data[i]);
        }
    }
}
