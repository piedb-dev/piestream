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

use std::any::type_name;
use std::convert::TryInto;
use std::fmt::Debug;

use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedRem, CheckedSub, Signed};
use risingwave_common::error::ErrorCode::{InternalError, NumericValueOutOfRange};
use risingwave_common::error::{Result, RwError};
use risingwave_common::types::{
    CheckedAdd as NaiveDateTimeCheckedAdd, Decimal, IntervalUnit, NaiveDateTimeWrapper,
    NaiveDateWrapper,
};

use super::cast::date_to_timestamp;

#[inline(always)]
pub fn general_add<T1, T2, T3>(l: T1, r: T2) -> Result<T3>
where
    T1: TryInto<T3> + Debug,
    T2: TryInto<T3> + Debug,
    T3: CheckedAdd,
{
    general_atm(l, r, |a, b| match a.checked_add(&b) {
        Some(c) => Ok(c),
        None => Err(RwError::from(NumericValueOutOfRange)),
    })
}

#[inline(always)]
pub fn general_sub<T1, T2, T3>(l: T1, r: T2) -> Result<T3>
where
    T1: TryInto<T3> + Debug,
    T2: TryInto<T3> + Debug,
    T3: CheckedSub,
{
    general_atm(l, r, |a, b| match a.checked_sub(&b) {
        Some(c) => Ok(c),
        None => Err(RwError::from(NumericValueOutOfRange)),
    })
}

#[inline(always)]
pub fn general_mul<T1, T2, T3>(l: T1, r: T2) -> Result<T3>
where
    T1: TryInto<T3> + Debug,
    T2: TryInto<T3> + Debug,
    T3: CheckedMul,
{
    general_atm(l, r, |a, b| match a.checked_mul(&b) {
        Some(c) => Ok(c),
        None => Err(RwError::from(NumericValueOutOfRange)),
    })
}

#[inline(always)]
pub fn general_div<T1, T2, T3>(l: T1, r: T2) -> Result<T3>
where
    T1: TryInto<T3> + Debug,
    T2: TryInto<T3> + Debug,
    T3: CheckedDiv,
{
    general_atm(l, r, |a, b| match a.checked_div(&b) {
        Some(c) => Ok(c),
        None => Err(RwError::from(NumericValueOutOfRange)),
    })
}

#[inline(always)]
pub fn general_mod<T1, T2, T3>(l: T1, r: T2) -> Result<T3>
where
    T1: TryInto<T3> + Debug,
    T2: TryInto<T3> + Debug,
    T3: CheckedRem,
{
    general_atm(l, r, |a, b| match a.checked_rem(&b) {
        Some(c) => Ok(c),
        None => Err(RwError::from(NumericValueOutOfRange)),
    })
}

#[inline(always)]
pub fn general_neg<T1: CheckedNeg>(expr: T1) -> Result<T1> {
    match expr.checked_neg() {
        Some(expr) => Ok(expr),
        None => Err(RwError::from(NumericValueOutOfRange)),
    }
}

#[inline(always)]
pub fn general_abs<T1: Signed + CheckedNeg>(expr: T1) -> Result<T1> {
    if expr.is_negative() {
        general_neg(expr)
    } else {
        Ok(expr)
    }
}

pub fn decimal_abs(decimal: Decimal) -> Result<Decimal> {
    Ok(Decimal::abs(&decimal).unwrap())
}

#[inline(always)]
pub fn general_atm<T1, T2, T3, F>(l: T1, r: T2, atm: F) -> Result<T3>
where
    T1: TryInto<T3> + Debug,
    T2: TryInto<T3> + Debug,
    F: FnOnce(T3, T3) -> Result<T3>,
{
    // TODO: We need to improve the error message
    let l: T3 = l.try_into().map_err(|_| {
        RwError::from(InternalError(format!(
            "Can't convert {} to {}",
            type_name::<T1>(),
            type_name::<T3>()
        )))
    })?;
    let r: T3 = r.try_into().map_err(|_| {
        RwError::from(InternalError(format!(
            "Can't convert {} to {}",
            type_name::<T2>(),
            type_name::<T3>()
        )))
    })?;
    atm(l, r)
}

#[inline(always)]
pub fn timestamp_timestamp_sub<T1, T2, T3>(
    l: NaiveDateTimeWrapper,
    r: NaiveDateTimeWrapper,
) -> Result<IntervalUnit> {
    let tmp = l.0 - r.0;
    Ok(IntervalUnit::new(0, tmp.num_days() as i32, 0))
}

#[inline(always)]
pub fn date_date_sub<T1, T2, T3>(l: NaiveDateWrapper, r: NaiveDateWrapper) -> Result<i32> {
    Ok((l.0 - r.0).num_days() as i32)
}

#[inline(always)]
pub fn interval_timestamp_add<T1, T2, T3>(
    l: IntervalUnit,
    r: NaiveDateTimeWrapper,
) -> Result<NaiveDateTimeWrapper> {
    r.checked_add(l)
}

#[inline(always)]
pub fn interval_date_add<T1, T2, T3>(
    l: IntervalUnit,
    r: NaiveDateWrapper,
) -> Result<NaiveDateTimeWrapper> {
    interval_timestamp_add::<T1, T2, T3>(l, date_to_timestamp(r)?)
}

#[inline(always)]
pub fn date_interval_add<T2, T1, T3>(
    l: NaiveDateWrapper,
    r: IntervalUnit,
) -> Result<NaiveDateTimeWrapper> {
    interval_date_add::<T1, T2, T3>(r, l)
}

#[inline(always)]
pub fn date_interval_sub<T2, T1, T3>(
    l: NaiveDateWrapper,
    r: IntervalUnit,
) -> Result<NaiveDateTimeWrapper> {
    interval_date_add::<T1, T2, T3>(r.negative(), l)
}

#[inline(always)]
pub fn timestamp_interval_add<T1, T2, T3>(
    l: NaiveDateTimeWrapper,
    r: IntervalUnit,
) -> Result<NaiveDateTimeWrapper> {
    interval_timestamp_add::<T1, T2, T3>(r, l)
}

#[inline(always)]
pub fn timestamp_interval_sub<T1, T2, T3>(
    l: NaiveDateTimeWrapper,
    r: IntervalUnit,
) -> Result<NaiveDateTimeWrapper> {
    interval_timestamp_add::<T1, T2, T3>(r.negative(), l)
}

#[inline(always)]
pub fn interval_int_mul<T1, T2, T3>(l: IntervalUnit, r: T2) -> Result<IntervalUnit>
where
    T2: TryInto<i32> + Debug,
{
    l.checked_mul_int(r)
        .ok_or_else(|| NumericValueOutOfRange.into())
}

#[inline(always)]
pub fn int_interval_mul<T1, T2, T3>(l: T1, r: IntervalUnit) -> Result<IntervalUnit>
where
    T1: TryInto<i32> + Debug,
{
    interval_int_mul::<T2, T1, T3>(r, l)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use risingwave_common::types::Decimal;

    use crate::vector_op::arithmetic_op::general_add;

    #[test]
    fn test() {
        assert_eq!(
            general_add::<_, _, Decimal>(Decimal::from_str("1").unwrap(), 1i32).unwrap(),
            Decimal::from_str("2").unwrap()
        );
    }
}
