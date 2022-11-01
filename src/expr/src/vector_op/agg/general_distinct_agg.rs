// Copyright 2022 Piedb Data
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

use std::collections::HashSet;
use std::marker::PhantomData;

use piestream_common::array::*;
use piestream_common::bail;
use piestream_common::types::*;

use crate::vector_op::agg::aggregator::Aggregator;
use crate::vector_op::agg::functions::RTFn;
use crate::Result;

/// Where the actual aggregation happens.
///
/// This is for aggregation function with distinct keyword.
/// For example, select count(distinct c1) from t;
///
/// For aggregation without distinct keyword, please refer to `GeneralAgg`
#[derive(Clone)]
pub struct GeneralDistinctAgg<T, F, R>
where
    T: Array,
    F: for<'a> RTFn<'a, T, R>,
    R: Array,
{
    return_type: DataType,
    input_col_idx: usize,
    result: Option<R::OwnedItem>,
    f: F,
    exists: HashSet<Datum>,
    _phantom: PhantomData<T>,
}
impl<T, F, R> GeneralDistinctAgg<T, F, R>
where
    T: Array,
    F: for<'a> RTFn<'a, T, R>,
    R: Array,
{
    pub fn new(return_type: DataType, input_col_idx: usize, f: F) -> Self {
        Self {
            return_type,
            input_col_idx,
            result: None,
            f,
            exists: HashSet::new(),
            _phantom: PhantomData,
        }
    }

    fn update_single_concrete(&mut self, input: &T, row_id: usize) -> Result<()> {
        let value = input
            .value_at(row_id)
            .map(|scalar_ref| scalar_ref.to_owned_scalar().to_scalar_value());
        if self.exists.insert(value) {
            let datum = self
                .f
                .eval(
                    self.result.as_ref().map(|x| x.as_scalar_ref()),
                    input.value_at(row_id),
                )?
                .map(|x| x.to_owned_scalar());
            self.result = datum;
        }
        Ok(())
    }

    fn update_multi_concrete(
        &mut self,
        input: &T,
        start_row_id: usize,
        end_row_id: usize,
    ) -> Result<()> {
        let input = input
            .iter()
            .skip(start_row_id)
            .take(end_row_id - start_row_id)
            .filter(|scalar_ref| {
                self.exists.insert(
                    scalar_ref.map(|scalar_ref| scalar_ref.to_owned_scalar().to_scalar_value()),
                )
            });
        let mut cur = self.result.as_ref().map(|x| x.as_scalar_ref());
        for datum in input {
            cur = self.f.eval(cur, datum)?;
        }
        self.result = cur.map(|x| x.to_owned_scalar());
        Ok(())
    }

    fn output_concrete(&mut self, builder: &mut R::Builder) -> Result<()> {
        let res = std::mem::replace(&mut self.result, None);
        builder.append(res.as_ref().map(|x| x.as_scalar_ref()));
        Ok(())
    }
}

macro_rules! impl_aggregator {
    ($input:ty, $input_variant:ident, $result:ty, $result_variant:ident) => {
        impl<F> Aggregator for GeneralDistinctAgg<$input, F, $result>
        where
            F: for<'a> RTFn<'a, $input, $result>,
        {
            fn return_type(&self) -> DataType {
                self.return_type.clone()
            }

            fn update_single(&mut self, input: &DataChunk, row_id: usize) -> Result<()> {
                if let ArrayImpl::$input_variant(i) =
                    input.column_at(self.input_col_idx).array_ref()
                {
                    self.update_single_concrete(i, row_id)
                } else {
                    bail!("Input fail to match {}.", stringify!($input_variant))
                }
            }

            fn update_multi(
                &mut self,
                input: &DataChunk,
                start_row_id: usize,
                end_row_id: usize,
            ) -> Result<()> {
                if let ArrayImpl::$input_variant(i) =
                    input.column_at(self.input_col_idx).array_ref()
                {
                    self.update_multi_concrete(i, start_row_id, end_row_id)
                } else {
                    bail!("Input fail to match {}.", stringify!($input_variant))
                }
            }

            fn output(&mut self, builder: &mut ArrayBuilderImpl) -> Result<()> {
                if let ArrayBuilderImpl::$result_variant(b) = builder {
                    self.output_concrete(b)
                } else {
                    bail!("Builder fail to match {}.", stringify!($result_variant))
                }
            }
        }
    };
}

// max/min
impl_aggregator! { I16Array, Int16, I16Array, Int16 }
impl_aggregator! { I32Array, Int32, I32Array, Int32 }
impl_aggregator! { I64Array, Int64, I64Array, Int64 }
impl_aggregator! { F32Array, Float32, F32Array, Float32 } // sum
impl_aggregator! { F64Array, Float64, F64Array, Float64 } // sum
impl_aggregator! { DecimalArray, Decimal, DecimalArray, Decimal } // sum
impl_aggregator! { Utf8Array, Utf8, Utf8Array, Utf8 }
impl_aggregator! { BoolArray, Bool, BoolArray, Bool } // TODO(#359): remove once unnecessary
impl_aggregator! { StructArray, Struct, StructArray, Struct }
impl_aggregator! { ListArray, List, ListArray, List }
impl_aggregator! { IntervalArray, Interval, IntervalArray, Interval }
impl_aggregator! { NaiveTimeArray, NaiveTime, NaiveTimeArray, NaiveTime }
impl_aggregator! { NaiveDateArray, NaiveDate, NaiveDateArray, NaiveDate }
impl_aggregator! { NaiveDateTimeArray, NaiveDateTime, NaiveDateTimeArray, NaiveDateTime }

// count
impl_aggregator! { I16Array, Int16, I64Array, Int64 } // sum
impl_aggregator! { I32Array, Int32, I64Array, Int64 } // sum
impl_aggregator! { F32Array, Float32, I64Array, Int64 }
impl_aggregator! { F64Array, Float64, I64Array, Int64 }
impl_aggregator! { DecimalArray, Decimal, I64Array, Int64 }
impl_aggregator! { Utf8Array, Utf8, I64Array, Int64 }
impl_aggregator! { BoolArray, Bool, I64Array, Int64 }
impl_aggregator! { StructArray, Struct, I64Array, Int64 }
impl_aggregator! { ListArray, List, I64Array, Int64 }
impl_aggregator! { IntervalArray, Interval, I64Array, Int64 }
impl_aggregator! { NaiveTimeArray, NaiveTime, I64Array, Int64 }
impl_aggregator! { NaiveDateArray, NaiveDate, I64Array, Int64 }
impl_aggregator! { NaiveDateTimeArray, NaiveDateTime, I64Array, Int64 }

// sum
impl_aggregator! { I64Array, Int64, DecimalArray, Decimal }

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use piestream_common::array::column::Column;
    use piestream_common::types::Decimal;

    use super::*;
    use crate::expr::AggKind;
    use crate::vector_op::agg::aggregator::create_agg_state_unary;

    fn eval_agg(
        input_type: DataType,
        input: ArrayRef,
        agg_kind: AggKind,
        return_type: DataType,
        mut builder: ArrayBuilderImpl,
    ) -> Result<ArrayImpl> {
        let len = input.len();
        let input_chunk = DataChunk::new(vec![Column::new(input)], len);
        let mut agg_state = create_agg_state_unary(input_type, 0, agg_kind, return_type, true)?;
        agg_state.update_multi(&input_chunk, 0, input_chunk.cardinality())?;
        agg_state.output(&mut builder)?;
        Ok(builder.finish())
    }

    #[test]
    fn vec_distinct_sum_int32() -> Result<()> {
        let input = I32Array::from_slice(&[Some(1), Some(1), Some(3)]);
        let agg_kind = AggKind::Sum;
        let input_type = DataType::Int32;
        let return_type = DataType::Int64;
        let actual = eval_agg(
            input_type,
            Arc::new(input.into()),
            agg_kind,
            return_type,
            ArrayBuilderImpl::Int64(I64ArrayBuilder::new(0)),
        )?;
        let actual = actual.as_int64();
        let actual = actual.iter().collect::<Vec<_>>();
        assert_eq!(actual, &[Some(4)]);
        Ok(())
    }

    #[test]
    fn vec_distinct_sum_int64() -> Result<()> {
        let input = I64Array::from_slice(&[Some(1), Some(1), Some(3)]);
        let agg_kind = AggKind::Sum;
        let input_type = DataType::Int64;
        let return_type = DataType::Decimal;
        let actual = eval_agg(
            input_type,
            Arc::new(input.into()),
            agg_kind,
            return_type,
            DecimalArrayBuilder::new(0).into(),
        )?;
        let actual: &DecimalArray = (&actual).into();
        let actual = actual.iter().collect::<Vec<Option<Decimal>>>();
        assert_eq!(actual, vec![Some(Decimal::from(4))]);
        Ok(())
    }

    #[test]
    fn vec_distinct_min_float32() -> Result<()> {
        let input = F32Array::from_slice(&[Some(1.0.into()), Some(2.0.into()), Some(3.0.into())]);
        let agg_kind = AggKind::Min;
        let input_type = DataType::Float32;
        let return_type = DataType::Float32;
        let actual = eval_agg(
            input_type,
            Arc::new(input.into()),
            agg_kind,
            return_type,
            ArrayBuilderImpl::Float32(F32ArrayBuilder::new(0)),
        )?;
        let actual = actual.as_float32();
        let actual = actual.iter().collect::<Vec<_>>();
        assert_eq!(actual, &[Some(1.0.into())]);
        Ok(())
    }

    #[test]
    fn vec_distinct_min_char() -> Result<()> {
        let input = Utf8Array::from_slice(&[Some("b"), Some("aa")]);
        let agg_kind = AggKind::Min;
        let input_type = DataType::Varchar;
        let return_type = DataType::Varchar;
        let actual = eval_agg(
            input_type,
            Arc::new(input.into()),
            agg_kind,
            return_type,
            ArrayBuilderImpl::Utf8(Utf8ArrayBuilder::new(0)),
        )?;
        let actual = actual.as_utf8();
        let actual = actual.iter().collect::<Vec<_>>();
        assert_eq!(actual, vec![Some("aa")]);
        Ok(())
    }

    #[test]
    fn vec_distinct_max_char() -> Result<()> {
        let input = Utf8Array::from_slice(&[Some("b"), Some("aa")]);
        let agg_kind = AggKind::Max;
        let input_type = DataType::Varchar;
        let return_type = DataType::Varchar;
        let actual = eval_agg(
            input_type,
            Arc::new(input.into()),
            agg_kind,
            return_type,
            ArrayBuilderImpl::Utf8(Utf8ArrayBuilder::new(0)),
        )?;
        let actual = actual.as_utf8();
        let actual = actual.iter().collect::<Vec<_>>();
        assert_eq!(actual, vec![Some("b")]);
        Ok(())
    }

    #[test]
    fn vec_distinct_count_int32() -> Result<()> {
        let test_case = |input: ArrayImpl, expected: &[Option<i64>]| -> Result<()> {
            let agg_kind = AggKind::Count;
            let input_type = DataType::Int32;
            let return_type = DataType::Int64;
            let actual = eval_agg(
                input_type,
                Arc::new(input),
                agg_kind,
                return_type,
                ArrayBuilderImpl::Int64(I64ArrayBuilder::new(0)),
            )?;
            let actual = actual.as_int64();
            let actual = actual.iter().collect::<Vec<_>>();
            assert_eq!(actual, expected);
            Ok(())
        };
        let input = I32Array::from_slice(&[Some(1), Some(1), Some(3)]);
        let expected = &[Some(2)];
        test_case(input.into(), expected)?;
        let input = I32Array::from_slice(&[]);
        let expected = &[None];
        test_case(input.into(), expected)?;
        let input = I32Array::from_slice(&[None]);
        let expected = &[Some(0)];
        test_case(input.into(), expected)
    }
}
