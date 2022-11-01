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

use std::collections::HashMap;
use std::sync::LazyLock;

use itertools::{iproduct, Itertools as _};
use num_integer::Integer as _;
use piestream_common::error::{ErrorCode, Result};
use piestream_common::types::{DataType, DataTypeName};

use super::{align_types, cast_ok_base, CastContext};
use crate::expr::type_inference::cast::align_array_and_element;
use crate::expr::{Expr as _, ExprImpl, ExprType};

/// Infers the return type of a function. Returns `Err` if the function with specified data types
/// is not supported on backend.
pub fn infer_type(func_type: ExprType, inputs: &mut Vec<ExprImpl>) -> Result<DataType> {
    if let Some(res) = infer_type_for_special(func_type, inputs).transpose() {
        return res;
    }

    let actuals = inputs
        .iter()
        .map(|e| match e.is_unknown() {
            true => None,
            false => Some(e.return_type().into()),
        })
        .collect_vec();
    let sig = infer_type_name(&FUNC_SIG_MAP, func_type, &actuals)?;
    let inputs_owned = std::mem::take(inputs);
    *inputs = inputs_owned
        .into_iter()
        .zip_eq(&sig.inputs_type)
        .map(|(expr, t)| {
            if DataTypeName::from(expr.return_type()) != *t {
                return expr.cast_implicit((*t).into());
            }
            Ok(expr)
        })
        .try_collect()?;
    Ok(sig.ret_type.into())
}

macro_rules! ensure_arity {
    ($func:literal, $lower:literal <= | $inputs:ident | <= $upper:literal) => {
        if !($lower <= $inputs.len() && $inputs.len() <= $upper) {
            return Err(ErrorCode::BindError(format!(
                "Function `{}` takes {} to {} arguments ({} given)",
                $func,
                $lower,
                $upper,
                $inputs.len(),
            ))
            .into());
        }
    };
    ($func:literal, $lower:literal <= | $inputs:ident |) => {
        if !($lower <= $inputs.len()) {
            return Err(ErrorCode::BindError(format!(
                "Function `{}` takes at least {} arguments ({} given)",
                $func,
                $lower,
                $inputs.len(),
            ))
            .into());
        }
    };
    ($func:literal, | $inputs:ident | == $num:literal) => {
        if !($inputs.len() == $num) {
            return Err(ErrorCode::BindError(format!(
                "Function `{}` takes {} arguments ({} given)",
                $func,
                $num,
                $inputs.len(),
            ))
            .into());
        }
    };
}

/// Special exprs that cannot be handled by [`infer_type_name`] and [`FuncSigMap`] are handled here.
/// These include variadic functions, list and struct type, as well as non-implicit cast.
///
/// We should aim for enhancing the general inferring framework and reduce the special cases here.
fn infer_type_for_special(
    func_type: ExprType,
    inputs: &mut Vec<ExprImpl>,
) -> Result<Option<DataType>> {
    match func_type {
        ExprType::Case => {
            let len = inputs.len();
            align_types(inputs.iter_mut().enumerate().filter_map(|(i, e)| {
                // `Case` organize `inputs` as (cond, res) pairs with a possible `else` res at
                // the end. So we align exprs at odd indices as well as the last one when length
                // is odd.
                match i.is_odd() || len.is_odd() && i == len - 1 {
                    true => Some(e),
                    false => None,
                }
            }))
            .map(Some)
        }
        ExprType::In => {
            align_types(inputs.iter_mut())?;
            Ok(Some(DataType::Boolean))
        }
        ExprType::Coalesce => {
            ensure_arity!("coalesce", 1 <= | inputs |);
            align_types(inputs.iter_mut()).map(Some)
        }
        ExprType::ConcatWs => {
            ensure_arity!("concat_ws", 2 <= | inputs |);
            let inputs_owned = std::mem::take(inputs);
            *inputs = inputs_owned
                .into_iter()
                .enumerate()
                .map(|(i, input)| match i {
                    // 0-th arg must be string
                    0 => input.cast_implicit(DataType::Varchar),
                    // subsequent can be any type, using the output format
                    _ => input.cast_output(),
                })
                .try_collect()?;
            Ok(Some(DataType::Varchar))
        }
        ExprType::ConcatOp => {
            let inputs_owned = std::mem::take(inputs);
            *inputs = inputs_owned
                .into_iter()
                .map(|input| input.cast_explicit(DataType::Varchar))
                .try_collect()?;
            Ok(Some(DataType::Varchar))
        }
        ExprType::IsNotNull => {
            ensure_arity!("is_not_null", | inputs | == 1);
            match inputs[0].return_type() {
                DataType::Struct(_) | DataType::List { .. } => Ok(Some(DataType::Boolean)),
                _ => Ok(None),
            }
        }
        ExprType::IsNull => {
            ensure_arity!("is_null", | inputs | == 1);
            match inputs[0].return_type() {
                DataType::Struct(_) | DataType::List { .. } => Ok(Some(DataType::Boolean)),
                _ => Ok(None),
            }
        }
        ExprType::RegexpMatch => {
            ensure_arity!("regexp_match", 2 <= | inputs | <= 3);
            if inputs.len() == 3 {
                return Err(ErrorCode::NotImplemented(
                    "flag in regexp_match".to_string(),
                    4545.into(),
                )
                .into());
            }
            Ok(Some(DataType::List {
                datatype: Box::new(DataType::Varchar),
            }))
        }
        ExprType::ArrayCat => {
            ensure_arity!("array_cat", | inputs | == 2);
            let left_type = inputs[0].return_type();
            let right_type = inputs[1].return_type();
            let return_type = match (&left_type, &right_type) {
                (
                    DataType::List {
                        datatype: left_elem_type,
                    },
                    DataType::List {
                        datatype: right_elem_type,
                    },
                ) => {
                    if let Ok(res) = align_types(inputs.iter_mut()) {
                        Some(res)
                    } else if **left_elem_type == right_type {
                        Some(left_type.clone())
                    } else if left_type == **right_elem_type {
                        Some(right_type.clone())
                    } else {
                        let common_type = align_array_and_element(0, 1, inputs)
                            .or_else(|_| align_array_and_element(1, 0, inputs));
                        match common_type {
                            Ok(casted) => Some(casted),
                            Err(err) => return Err(err.into()),
                        }
                    }
                }
                _ => None,
            };
            Ok(Some(return_type.ok_or_else(|| {
                ErrorCode::BindError(format!(
                    "Cannot concatenate {} and {}",
                    left_type, right_type
                ))
            })?))
        }
        ExprType::ArrayAppend => {
            ensure_arity!("array_append", | inputs | == 2);
            let common_type = align_array_and_element(0, 1, inputs);
            match common_type {
                Ok(casted) => Ok(Some(casted)),
                Err(_) => Err(ErrorCode::BindError(format!(
                    "Cannot append {} to {}",
                    inputs[0].return_type(),
                    inputs[1].return_type()
                ))
                .into()),
            }
        }
        ExprType::ArrayPrepend => {
            ensure_arity!("array_prepend", | inputs | == 2);
            let common_type = align_array_and_element(1, 0, inputs);
            match common_type {
                Ok(casted) => Ok(Some(casted)),
                Err(_) => Err(ErrorCode::BindError(format!(
                    "Cannot prepend {} to {}",
                    inputs[0].return_type(),
                    inputs[1].return_type()
                ))
                .into()),
            }
        }
        ExprType::Vnode => {
            ensure_arity!("vnode", 1 <= | inputs |);
            Ok(Some(DataType::Int16))
        }
        _ => Ok(None),
    }
}

/// From all available functions in `sig_map`, find and return the best matching `FuncSign` for the
/// provided `func_type` and `inputs`. This not only support exact function signature match, but can
/// also match `substr(varchar, smallint)` or even `substr(varchar, unknown)` to `substr(varchar,
/// int)`.
///
/// This corresponds to the `PostgreSQL` rules on operators and functions here:
/// * <https://www.postgresql.org/docs/current/typeconv-oper.html>
/// * <https://www.postgresql.org/docs/current/typeconv-func.html>
///
/// To summarize,
/// 1. Find all functions with matching `func_type` and argument count.
/// 2. For binary operator with unknown on exactly one side, try to find an exact match assuming
///    both sides are same type.
/// 3. Rank candidates based on most matching positions. This covers Rule 2, 4a, 4c and 4d in
///    `PostgreSQL`. See [`top_matches`] for details.
/// 4. Attempt to narrow down candidates by selecting type categories for unknowns. This covers Rule
///    4e in `PostgreSQL`. See [`narrow_category`] for details.
/// 5. Attempt to narrow down candidates by assuming all arguments are same type. This covers Rule
///    4f in `PostgreSQL`. See [`narrow_same_type`] for details.
fn infer_type_name<'a, 'b>(
    sig_map: &'a FuncSigMap,
    func_type: ExprType,
    inputs: &'b [Option<DataTypeName>],
) -> Result<&'a FuncSign> {
    let candidates = sig_map
        .0
        .get(&(func_type, inputs.len()))
        .map(std::ops::Deref::deref)
        .unwrap_or_default();

    // Binary operators have a special `unknown` handling rule for exact match. We do not
    // distinguish operators from functions as of now.
    if inputs.len() == 2 {
        let t = match (inputs[0], inputs[1]) {
            (None, t) => Ok(t),
            (t, None) => Ok(t),
            (Some(_), Some(_)) => Err(()),
        };
        if let Ok(Some(t)) = t {
            let exact = candidates.iter().find(|sig| sig.inputs_type == [t, t]);
            if let Some(sig) = exact {
                return Ok(sig);
            }
        }
    }

    let mut candidates = top_matches(candidates, inputs);

    if candidates.is_empty() {
        return Err(ErrorCode::NotImplemented(
            format!(
                "{:?}{:?}",
                func_type,
                inputs.iter().map(TypeDebug).collect_vec()
            ),
            112.into(),
        )
        .into());
    }

    // After this line `candidates` will never be empty, as the narrow rules will retain original
    // list when no desirable candidates can be selected.

    candidates = narrow_category(candidates, inputs);

    candidates = narrow_same_type(candidates, inputs);

    match &candidates[..] {
        [] => unreachable!(),
        [sig] => Ok(sig),
        _ => Err(ErrorCode::BindError(format!(
            "function {:?}{:?} is not unique\nHINT:  Could not choose a best candidate function. You might need to add explicit type casts.",
            func_type,
            inputs.iter().map(TypeDebug).collect_vec(),
        ))
        .into()),
    }
}

/// Checks if `t` is a preferred type in any type category, as defined by `PostgreSQL`:
/// <https://www.postgresql.org/docs/current/catalog-pg-type.html>.
fn is_preferred(t: DataTypeName) -> bool {
    use DataTypeName as T;
    matches!(
        t,
        T::Float64 | T::Boolean | T::Varchar | T::Timestampz | T::Interval
    )
}

/// Checks if `source` can be implicitly casted to `target`. Generally we do not consider it being a
/// valid cast when they are of the same type, because it may deserve special treatment. This is
/// also the behavior of underlying [`cast_ok_base`].
///
/// Sometimes it is more convenient to include equality when checking whether a formal parameter can
/// accept an actual argument. So we introduced `eq_ok` to control this behavior.
fn implicit_ok(source: DataTypeName, target: DataTypeName, eq_ok: bool) -> bool {
    eq_ok && source == target || cast_ok_base(source, target, CastContext::Implicit)
}

/// Find the top `candidates` that match `inputs` on most non-null positions. This covers Rule 2,
/// 4a, 4c and 4d in [`PostgreSQL`](https://www.postgresql.org/docs/current/typeconv-func.html).
///
/// * Rule 2 & 4c: Keep candidates that have most exact type matches. Exact match on all posistions
///   is just a special case.
/// * Rule 4d: Break ties by selecting those that accept preferred types at most positions.
/// * Rule 4a: If the input cannot implicit cast to expected type at any position, this candidate is
///   discarded.
///
/// Corresponding implementation in `PostgreSQL`:
/// * Rule 2 on operators: `OpernameGetOprid()` in `namespace.c` [14.0/86a4dc1][rule 2 oper src]
///   * Note that unknown-handling logic of `binary_oper_exact()` in `parse_oper.c` is in
///     [`infer_type_name`].
/// * Rule 2 on functions: Line 1427 - Line 1439 of `func_get_detail()` in `parse_func.c`
///   [14.0/86a4dc1][rule 2 func src]
/// * Rule 4a: `func_match_argtypes()` in `parse_func.c` [14.0/86a4dc1][rule 4a src]
/// * Rule 4c: Line 1062 - Line 1104 of `func_select_candidate()` in `parse_func.c`
///   [14.0/86a4dc1][rule 4c src]
/// * Rule 4d: Line 1106 - Line 1153 of `func_select_candidate()` in `parse_func.c`
///   [14.0/86a4dc1][rule 4d src]
///
/// [rule 2 oper src]: https://github.com/postgres/postgres/blob/86a4dc1e6f29d1992a2afa3fac1a0b0a6e84568c/src/backend/catalog/namespace.c#L1516-L1611
/// [rule 2 func src]: https://github.com/postgres/postgres/blob/86a4dc1e6f29d1992a2afa3fac1a0b0a6e84568c/src/backend/parser/parse_func.c#L1427-L1439
/// [rule 4a src]: https://github.com/postgres/postgres/blob/86a4dc1e6f29d1992a2afa3fac1a0b0a6e84568c/src/backend/parser/parse_func.c#L907-L947
/// [rule 4c src]: https://github.com/postgres/postgres/blob/86a4dc1e6f29d1992a2afa3fac1a0b0a6e84568c/src/backend/parser/parse_func.c#L1062-L1104
/// [rule 4d src]: https://github.com/postgres/postgres/blob/86a4dc1e6f29d1992a2afa3fac1a0b0a6e84568c/src/backend/parser/parse_func.c#L1106-L1153
fn top_matches<'a, 'b>(
    candidates: &'a [FuncSign],
    inputs: &'b [Option<DataTypeName>],
) -> Vec<&'a FuncSign> {
    let mut best_exact = 0;
    let mut best_preferred = 0;
    let mut best_candidates = Vec::new();

    for sig in candidates {
        let mut n_exact = 0;
        let mut n_preferred = 0;
        let mut castable = true;
        for (formal, actual) in sig.inputs_type.iter().zip_eq(inputs) {
            let Some(actual) = actual else { continue };
            if formal == actual {
                n_exact += 1;
            } else if !implicit_ok(*actual, *formal, false) {
                castable = false;
                break;
            }
            if is_preferred(*formal) {
                n_preferred += 1;
            }
        }
        if !castable {
            continue;
        }
        if n_exact > best_exact || n_exact == best_exact && n_preferred > best_preferred {
            best_exact = n_exact;
            best_preferred = n_preferred;
            best_candidates.clear();
        }
        if n_exact == best_exact && n_preferred == best_preferred {
            best_candidates.push(sig);
        }
    }
    best_candidates
}

/// Attempt to narrow down candidates by selecting type categories for unknowns. This covers Rule 4e
/// in [`PostgreSQL`](https://www.postgresql.org/docs/current/typeconv-func.html).
///
/// This is done in 2 phases:
/// * First, check each unknown position individually and try to find a type category for it.
///   * If any candidate accept varchar, varchar is selected;
///   * otherwise, if all candidate accept the same category, select this category.
///     * Also record whether any candidate accept the preferred type within this category.
/// * When all unknown positions are assigned their type categories, discard a candidate if at any
///   position
///   * it does not agree with the type category assignment;
///   * the assigned type category contains a preferred type but the candidate is not preferred.
///
/// If the first phase fails or the second phase gives an empty set, this attempt preserves original
/// list untouched.
///
/// Corresponding implementation in `PostgreSQL`:
/// * Line 1164 - Line 1298 of `func_select_candidate()` in `parse_func.c` [14.0/86a4dc1][rule 4e
///   src]
///
/// [rule 4e src]: https://github.com/postgres/postgres/blob/86a4dc1e6f29d1992a2afa3fac1a0b0a6e84568c/src/backend/parser/parse_func.c#L1164-L1298
fn narrow_category<'a, 'b>(
    candidates: Vec<&'a FuncSign>,
    inputs: &'b [Option<DataTypeName>],
) -> Vec<&'a FuncSign> {
    const BIASED_TYPE: DataTypeName = DataTypeName::Varchar;
    let Ok(categories) = inputs.iter().enumerate().map(|(i, actual)| {
        // This closure returns
        // * Err(()) when a category cannot be selected
        // * Ok(None) when actual argument is non-null and can skip selection
        // * Ok(Some(t)) when the selected category is `t`
        //
        // Here `t` is actually just one type within that selected category, rather than the
        // category itself. It is selected to be the [`super::least_restrictive`] over all
        // candidates. This makes sure that `t` is the preferred type if any candidate accept it.
        if actual.is_some() {
            return Ok(None);
        }
        let mut category = Ok(candidates[0].inputs_type[i]);
        for sig in &candidates[1..] {
            let formal = sig.inputs_type[i];
            if formal == BIASED_TYPE || category == Ok(BIASED_TYPE) {
                category = Ok(BIASED_TYPE);
                break;
            }
            // formal != BIASED_TYPE && category.is_err():
            // - Category conflict err can only be solved by a later varchar. Skip this candidate.
            let Ok(selected) = category else { continue };
            // least_restrictive or mark temporary conflict err
            if implicit_ok(formal, selected, true) {
                // noop
            } else if implicit_ok(selected, formal, false) {
                category = Ok(formal);
            } else {
                category = Err(());
            }
        }
        category.map(Some)
    }).try_collect::<_, Vec<_>, _>() else {
        // First phase failed.
        return candidates;
    };
    let cands_temp = candidates
        .iter()
        .filter(|sig| {
            sig.inputs_type
                .iter()
                .zip_eq(&categories)
                .all(|(formal, category)| {
                    // category.is_none() means the actual argument is non-null and skipped category
                    // selection.
                    let Some(selected) = category else { return true };
                    *formal == *selected
                        || !is_preferred(*selected) && implicit_ok(*formal, *selected, false)
                })
        })
        .copied()
        .collect_vec();
    match cands_temp.is_empty() {
        true => candidates,
        false => cands_temp,
    }
}

/// Attempt to narrow down candidates by assuming all arguments are same type. This covers Rule 4f
/// in [`PostgreSQL`](https://www.postgresql.org/docs/current/typeconv-func.html).
///
/// If all non-null arguments are same type, assume all unknown arguments are of this type as well.
/// Discard a candidate if its parameter type cannot be casted from this type.
///
/// If the condition is not met or the result is empty, this attempt preserves original list
/// untouched.
///
/// Note this rule cannot replace special treatment given to binary operators in [Rule 2], because
/// this runs after varchar-biased Rule 4e ([`narrow_category`]), and has no preference between
/// exact-match and castable-match.
///
/// Corresponding implementation in `PostgreSQL`:
/// * Line 1300 - Line 1355 of `func_select_candidate()` in `parse_func.c` [14.0/86a4dc1][rule 4f
///   src]
///
/// [rule 4f src]: https://github.com/postgres/postgres/blob/86a4dc1e6f29d1992a2afa3fac1a0b0a6e84568c/src/backend/parser/parse_func.c#L1300-L1355
/// [Rule 2]: https://www.postgresql.org/docs/current/typeconv-oper.html#:~:text=then%20assume%20it%20is%20the%20same%20type%20as%20the%20other%20argument%20for%20this%20check
fn narrow_same_type<'a, 'b>(
    candidates: Vec<&'a FuncSign>,
    inputs: &'b [Option<DataTypeName>],
) -> Vec<&'a FuncSign> {
    let Ok(Some(same_type)) = inputs.iter().try_fold(None, |acc, cur| match (acc, cur) {
        (None, t) => Ok(*t),
        (t, None) => Ok(t),
        (Some(l), Some(r)) if l == *r => Ok(Some(l)),
        _ => Err(())
    }) else {
        return candidates;
    };
    let cands_temp = candidates
        .iter()
        .filter(|sig| {
            sig.inputs_type
                .iter()
                .all(|formal| implicit_ok(same_type, *formal, true))
        })
        .copied()
        .collect_vec();
    match cands_temp.is_empty() {
        true => candidates,
        false => cands_temp,
    }
}

struct TypeDebug<'a>(&'a Option<DataTypeName>);
impl<'a> std::fmt::Debug for TypeDebug<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(t) => t.fmt(f),
            None => write!(f, "unknown"),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct FuncSign {
    pub func: ExprType,
    pub inputs_type: Vec<DataTypeName>,
    pub ret_type: DataTypeName,
}

#[derive(Default)]
pub struct FuncSigMap(HashMap<(ExprType, usize), Vec<FuncSign>>);
impl FuncSigMap {
    fn insert(&mut self, func: ExprType, param_types: Vec<DataTypeName>, ret_type: DataTypeName) {
        let arity = param_types.len();
        let inputs_type = param_types.into_iter().map(Into::into).collect();
        let sig = FuncSign {
            func,
            inputs_type,
            ret_type,
        };
        self.0.entry((func, arity)).or_default().push(sig)
    }
}

fn build_binary_cmp_funcs(map: &mut FuncSigMap, exprs: &[ExprType], args: &[DataTypeName]) {
    for (e, lt, rt) in iproduct!(exprs, args, args) {
        map.insert(*e, vec![*lt, *rt], DataTypeName::Boolean);
    }
}

fn build_binary_atm_funcs(map: &mut FuncSigMap, exprs: &[ExprType], args: &[DataTypeName]) {
    for e in exprs {
        for (li, lt) in args.iter().enumerate() {
            for (ri, rt) in args.iter().enumerate() {
                let ret = if li <= ri { rt } else { lt };
                map.insert(*e, vec![*lt, *rt], *ret);
            }
        }
    }
}

fn build_unary_atm_funcs(map: &mut FuncSigMap, exprs: &[ExprType], args: &[DataTypeName]) {
    for (e, arg) in iproduct!(exprs, args) {
        map.insert(*e, vec![*arg], *arg);
    }
}

fn build_commutative_funcs(
    map: &mut FuncSigMap,
    expr: ExprType,
    arg0: DataTypeName,
    arg1: DataTypeName,
    ret: DataTypeName,
) {
    map.insert(expr, vec![arg0, arg1], ret);
    map.insert(expr, vec![arg1, arg0], ret);
}

fn build_round_funcs(map: &mut FuncSigMap, expr: ExprType) {
    map.insert(expr, vec![DataTypeName::Float64], DataTypeName::Float64);
    map.insert(expr, vec![DataTypeName::Decimal], DataTypeName::Decimal);
}

/// This function builds type derived map for all built-in functions that take a fixed number
/// of arguments.  They can be determined to have one or more type signatures since some are
/// compatible with more than one type.
/// Type signatures and arities of variadic functions are checked
/// [elsewhere](crate::expr::FunctionCall::new).
fn build_type_derive_map() -> FuncSigMap {
    use {DataTypeName as T, ExprType as E};
    let mut map = FuncSigMap::default();
    let all_types = [
        T::Boolean,
        T::Int16,
        T::Int32,
        T::Int64,
        T::Decimal,
        T::Float32,
        T::Float64,
        T::Varchar,
        T::Date,
        T::Timestamp,
        T::Timestampz,
        T::Time,
        T::Interval,
    ];
    let num_types = [
        T::Int16,
        T::Int32,
        T::Int64,
        T::Decimal,
        T::Float32,
        T::Float64,
    ];

    // logical expressions
    for e in [E::Not, E::IsTrue, E::IsNotTrue, E::IsFalse, E::IsNotFalse] {
        map.insert(e, vec![T::Boolean], T::Boolean);
    }
    for e in [E::And, E::Or] {
        map.insert(e, vec![T::Boolean, T::Boolean], T::Boolean);
    }
    map.insert(E::BoolOut, vec![T::Boolean], T::Varchar);

    // comparison expressions
    for e in [E::IsNull, E::IsNotNull] {
        for t in all_types {
            map.insert(e, vec![t], T::Boolean);
        }
    }
    let cmp_exprs = &[
        E::Equal,
        E::NotEqual,
        E::LessThan,
        E::LessThanOrEqual,
        E::GreaterThan,
        E::GreaterThanOrEqual,
        E::IsDistinctFrom,
        E::IsNotDistinctFrom,
    ];
    build_binary_cmp_funcs(&mut map, cmp_exprs, &num_types);
    build_binary_cmp_funcs(&mut map, cmp_exprs, &[T::Struct]);
    build_binary_cmp_funcs(&mut map, cmp_exprs, &[T::List]);
    build_binary_cmp_funcs(&mut map, cmp_exprs, &[T::Date, T::Timestamp, T::Timestampz]);
    build_binary_cmp_funcs(&mut map, cmp_exprs, &[T::Time, T::Interval]);
    for e in cmp_exprs {
        for t in [T::Boolean, T::Varchar] {
            map.insert(*e, vec![t, t], T::Boolean);
        }
    }

    let unary_atm_exprs = &[E::Abs, E::Neg];

    build_unary_atm_funcs(&mut map, unary_atm_exprs, &num_types);
    build_binary_atm_funcs(
        &mut map,
        &[E::Add, E::Subtract, E::Multiply, E::Divide],
        &num_types,
    );
    build_binary_atm_funcs(
        &mut map,
        &[E::Modulus],
        &[T::Int16, T::Int32, T::Int64, T::Decimal],
    );
    map.insert(E::RoundDigit, vec![T::Decimal, T::Int32], T::Decimal);

    // build bitwise operator
    // bitwise operator
    let integral_types = [T::Int16, T::Int32, T::Int64]; // reusable for and/or/xor/not

    build_binary_atm_funcs(
        &mut map,
        &[E::BitwiseAnd, E::BitwiseOr, E::BitwiseXor],
        &integral_types,
    );

    // Shift Operator is not using `build_binary_atm_funcs` because
    // allowed rhs is different from allowed lhs
    // return type is lhs rather than larger of the two
    for (e, lt, rt) in iproduct!(
        &[E::BitwiseShiftLeft, E::BitwiseShiftRight],
        &integral_types,
        &[T::Int16, T::Int32]
    ) {
        map.insert(*e, vec![*lt, *rt], *lt);
    }

    build_unary_atm_funcs(&mut map, &[E::BitwiseNot], &[T::Int16, T::Int32, T::Int64]);

    build_round_funcs(&mut map, E::Round);
    build_round_funcs(&mut map, E::Ceil);
    build_round_funcs(&mut map, E::Floor);

    // temporal expressions
    for (base, delta) in [
        (T::Date, T::Int32),
        (T::Timestamp, T::Interval),
        (T::Timestampz, T::Interval),
        (T::Time, T::Interval),
    ] {
        build_commutative_funcs(&mut map, E::Add, base, delta, base);
        map.insert(E::Subtract, vec![base, delta], base);
        map.insert(E::Subtract, vec![base, base], delta);
    }
    map.insert(E::Add, vec![T::Interval, T::Interval], T::Interval);
    map.insert(E::Subtract, vec![T::Interval, T::Interval], T::Interval);

    // date + interval = timestamp, date - interval = timestamp
    build_commutative_funcs(&mut map, E::Add, T::Date, T::Interval, T::Timestamp);
    map.insert(E::Subtract, vec![T::Date, T::Interval], T::Timestamp);
    // date + time = timestamp
    build_commutative_funcs(&mut map, E::Add, T::Date, T::Time, T::Timestamp);
    // interval * float8 = interval, interval / float8 = interval
    for t in num_types {
        build_commutative_funcs(&mut map, E::Multiply, T::Interval, t, T::Interval);
        map.insert(E::Divide, vec![T::Interval, t], T::Interval);
    }

    for t in [T::Timestamp, T::Time, T::Date] {
        map.insert(E::Extract, vec![T::Varchar, t], T::Decimal);
    }
    for t in [T::Timestamp, T::Date] {
        map.insert(E::TumbleStart, vec![t, T::Interval], T::Timestamp);
    }

    // string expressions
    for e in [E::Trim, E::Ltrim, E::Rtrim, E::Lower, E::Upper, E::Md5] {
        map.insert(e, vec![T::Varchar], T::Varchar);
    }
    for e in [E::Trim, E::Ltrim, E::Rtrim] {
        map.insert(e, vec![T::Varchar, T::Varchar], T::Varchar);
    }
    for e in [E::Repeat, E::Substr] {
        map.insert(e, vec![T::Varchar, T::Int32], T::Varchar);
    }
    map.insert(E::Substr, vec![T::Varchar, T::Int32, T::Int32], T::Varchar);
    for e in [E::Replace, E::Translate] {
        map.insert(e, vec![T::Varchar, T::Varchar, T::Varchar], T::Varchar);
    }
    map.insert(
        E::Overlay,
        vec![T::Varchar, T::Varchar, T::Int32],
        T::Varchar,
    );
    map.insert(
        E::Overlay,
        vec![T::Varchar, T::Varchar, T::Int32, T::Int32],
        T::Varchar,
    );
    for e in [
        E::Length,
        E::Ascii,
        E::CharLength,
        E::OctetLength,
        E::BitLength,
    ] {
        map.insert(e, vec![T::Varchar], T::Int32);
    }
    map.insert(E::Position, vec![T::Varchar, T::Varchar], T::Int32);
    map.insert(E::Like, vec![T::Varchar, T::Varchar], T::Boolean);
    map.insert(
        E::SplitPart,
        vec![T::Varchar, T::Varchar, T::Int32],
        T::Varchar,
    );
    // TODO: Support more `to_char` types.
    map.insert(E::ToChar, vec![T::Timestamp, T::Varchar], T::Varchar);

    map
}

static FUNC_SIG_MAP: LazyLock<FuncSigMap> = LazyLock::new(build_type_derive_map);

/// The table of function signatures.
pub fn func_sigs() -> impl Iterator<Item = &'static FuncSign> {
    FUNC_SIG_MAP.0.values().flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn infer_type_v0(func_type: ExprType, inputs_type: Vec<DataType>) -> Result<DataType> {
        let mut inputs = inputs_type
            .into_iter()
            .map(|t| {
                crate::expr::Literal::new(
                    Some(match t {
                        DataType::Boolean => true.into(),
                        DataType::Int16 => 1i16.into(),
                        DataType::Int32 => 1i32.into(),
                        DataType::Int64 => 1i64.into(),
                        DataType::Float32 => 1f32.into(),
                        DataType::Float64 => 1f64.into(),
                        DataType::Decimal => piestream_common::types::Decimal::NaN.into(),
                        _ => unimplemented!(),
                    }),
                    t,
                )
                .into()
            })
            .collect();
        infer_type(func_type, &mut inputs)
    }

    fn test_simple_infer_type(
        func_type: ExprType,
        inputs_type: Vec<DataType>,
        expected_type_name: DataType,
    ) {
        let ret = infer_type_v0(func_type, inputs_type).unwrap();
        assert_eq!(ret, expected_type_name);
    }

    fn test_infer_type_not_exist(func_type: ExprType, inputs_type: Vec<DataType>) {
        let ret = infer_type_v0(func_type, inputs_type);
        assert!(ret.is_err());
    }

    #[test]
    fn test_arithmetics() {
        use DataType::*;
        let atm_exprs = vec![
            ExprType::Add,
            ExprType::Subtract,
            ExprType::Multiply,
            ExprType::Divide,
        ];
        let num_promote_table = vec![
            (Int16, Int16, Int16),
            (Int16, Int32, Int32),
            (Int16, Int64, Int64),
            (Int16, Decimal, Decimal),
            (Int16, Float32, Float32),
            (Int16, Float64, Float64),
            (Int32, Int16, Int32),
            (Int32, Int32, Int32),
            (Int32, Int64, Int64),
            (Int32, Decimal, Decimal),
            (Int32, Float32, Float32),
            (Int32, Float64, Float64),
            (Int64, Int16, Int64),
            (Int64, Int32, Int64),
            (Int64, Int64, Int64),
            (Int64, Decimal, Decimal),
            (Int64, Float32, Float32),
            (Int64, Float64, Float64),
            (Decimal, Int16, Decimal),
            (Decimal, Int32, Decimal),
            (Decimal, Int64, Decimal),
            (Decimal, Decimal, Decimal),
            (Decimal, Float32, Float32),
            (Decimal, Float64, Float64),
            (Float32, Int16, Float32),
            (Float32, Int32, Float32),
            (Float32, Int64, Float32),
            (Float32, Decimal, Float32),
            (Float32, Float32, Float32),
            (Float32, Float64, Float64),
            (Float64, Int16, Float64),
            (Float64, Int32, Float64),
            (Float64, Int64, Float64),
            (Float64, Decimal, Float64),
            (Float64, Float32, Float64),
            (Float64, Float64, Float64),
        ];
        for (expr, (t1, t2, tr)) in iproduct!(atm_exprs, num_promote_table) {
            test_simple_infer_type(expr, vec![t1, t2], tr);
        }
    }

    #[test]
    fn test_bitwise() {
        use DataType::*;
        let bitwise_exprs = vec![
            ExprType::BitwiseAnd,
            ExprType::BitwiseOr,
            ExprType::BitwiseXor,
        ];
        let num_promote_table = vec![
            (Int16, Int16, Int16),
            (Int16, Int32, Int32),
            (Int16, Int64, Int64),
            (Int32, Int16, Int32),
            (Int32, Int32, Int32),
            (Int32, Int64, Int64),
            (Int64, Int16, Int64),
            (Int64, Int32, Int64),
            (Int64, Int64, Int64),
        ];
        for (expr, (t1, t2, tr)) in iproduct!(bitwise_exprs, num_promote_table) {
            test_simple_infer_type(expr, vec![t1, t2], tr);
        }

        for (expr, (t1, t2, tr)) in iproduct!(
            vec![ExprType::BitwiseShiftLeft, ExprType::BitwiseShiftRight,],
            vec![
                (Int16, Int16, Int16),
                (Int32, Int16, Int32),
                (Int64, Int16, Int64),
                (Int16, Int32, Int16),
                (Int64, Int32, Int64),
                (Int32, Int32, Int32),
            ]
        ) {
            test_simple_infer_type(expr, vec![t1, t2], tr);
        }
    }
    #[test]
    fn test_bool_num_not_exist() {
        let exprs = vec![
            ExprType::Add,
            ExprType::Subtract,
            ExprType::Multiply,
            ExprType::Divide,
            ExprType::Modulus,
            ExprType::Equal,
            ExprType::NotEqual,
            ExprType::LessThan,
            ExprType::LessThanOrEqual,
            ExprType::GreaterThan,
            ExprType::GreaterThanOrEqual,
            ExprType::And,
            ExprType::Or,
            ExprType::Not,
        ];
        let num_types = vec![
            DataType::Int16,
            DataType::Int32,
            DataType::Int64,
            DataType::Float32,
            DataType::Float64,
            DataType::Decimal,
        ];

        for (expr, num_t) in iproduct!(exprs, num_types) {
            test_infer_type_not_exist(expr, vec![num_t, DataType::Boolean]);
        }
    }

    #[test]
    fn test_match_implicit() {
        use DataTypeName as T;
        // func_name and ret_type does not affect the overload resolution logic
        const DUMMY_FUNC: ExprType = ExprType::Add;
        const DUMMY_RET: T = T::Int32;
        let testcases = [
            (
                "Binary special rule prefers arguments of same type.",
                vec![
                    vec![T::Int32, T::Int32],
                    vec![T::Int32, T::Varchar],
                    vec![T::Int32, T::Float64],
                ],
                &[Some(T::Int32), None] as &[_],
                Ok(&[T::Int32, T::Int32] as &[_]),
            ),
            (
                "Without binary special rule, Rule 4e selects varchar.",
                vec![
                    vec![T::Int32, T::Int32, T::Int32],
                    vec![T::Int32, T::Int32, T::Varchar],
                    vec![T::Int32, T::Int32, T::Float64],
                ],
                &[Some(T::Int32), Some(T::Int32), None] as &[_],
                Ok(&[T::Int32, T::Int32, T::Varchar] as &[_]),
            ),
            (
                "Without binary special rule, Rule 4e selects preferred type.",
                vec![
                    vec![T::Int32, T::Int32, T::Int32],
                    vec![T::Int32, T::Int32, T::Float64],
                ],
                &[Some(T::Int32), Some(T::Int32), None] as &[_],
                Ok(&[T::Int32, T::Int32, T::Float64] as &[_]),
            ),
            (
                "Without binary special rule, Rule 4f treats exact-match and cast-match equally.",
                vec![
                    vec![T::Int32, T::Int32, T::Int32],
                    vec![T::Int32, T::Int32, T::Float32],
                ],
                &[Some(T::Int32), Some(T::Int32), None] as &[_],
                Err("not unique"),
            ),
            (
                "`top_matches` ranks by exact count then preferred count",
                vec![
                    vec![T::Float64, T::Float64, T::Float64, T::Timestampz], // 0 exact 3 preferred
                    vec![T::Float64, T::Int32, T::Float32, T::Timestamp],    // 1 exact 1 preferred
                    vec![T::Float32, T::Float32, T::Int32, T::Timestampz],   // 1 exact 0 preferred
                    vec![T::Int32, T::Float64, T::Float32, T::Timestampz],   // 1 exact 1 preferred
                    vec![T::Int32, T::Int16, T::Int32, T::Timestampz], // 2 exact 1 non-castable
                    vec![T::Int32, T::Float64, T::Float32, T::Date],   // 1 exact 1 preferred
                ],
                &[Some(T::Int32), Some(T::Int32), Some(T::Int32), None] as &[_],
                Ok(&[T::Int32, T::Float64, T::Float32, T::Timestampz] as &[_]),
            ),
            (
                "Rule 4e fails and Rule 4f unique.",
                vec![
                    vec![T::Int32, T::Int32, T::Time],
                    vec![T::Int32, T::Int32, T::Int32],
                ],
                &[None, Some(T::Int32), None] as &[_],
                Ok(&[T::Int32, T::Int32, T::Int32] as &[_]),
            ),
            (
                "Rule 4e empty and Rule 4f unique.",
                vec![
                    vec![T::Int32, T::Int32, T::Varchar],
                    vec![T::Int32, T::Int32, T::Int32],
                    vec![T::Varchar, T::Int32, T::Int32],
                ],
                &[None, Some(T::Int32), None] as &[_],
                Ok(&[T::Int32, T::Int32, T::Int32] as &[_]),
            ),
            (
                "Rule 4e varchar resolves prior category conflict.",
                vec![
                    vec![T::Int32, T::Int32, T::Float32],
                    vec![T::Time, T::Int32, T::Int32],
                    vec![T::Varchar, T::Int32, T::Int32],
                ],
                &[None, Some(T::Int32), None] as &[_],
                Ok(&[T::Varchar, T::Int32, T::Int32] as &[_]),
            ),
            (
                "Rule 4f fails.",
                vec![
                    vec![T::Float32, T::Float32, T::Float32, T::Float32],
                    vec![T::Decimal, T::Decimal, T::Int64, T::Decimal],
                ],
                &[Some(T::Int16), Some(T::Int32), None, Some(T::Int64)] as &[_],
                Err("not unique"),
            ),
            (
                "Rule 4f all unknown.",
                vec![
                    vec![T::Float32, T::Float32, T::Float32, T::Float32],
                    vec![T::Decimal, T::Decimal, T::Int64, T::Decimal],
                ],
                &[None, None, None, None] as &[_],
                Err("not unique"),
            ),
        ];
        for (desc, candidates, inputs, expected) in testcases {
            let mut sig_map = FuncSigMap::default();
            candidates
                .into_iter()
                .for_each(|formals| sig_map.insert(DUMMY_FUNC, formals, DUMMY_RET));
            let result = infer_type_name(&sig_map, DUMMY_FUNC, inputs);
            match (expected, result) {
                (Ok(expected), Ok(found)) => {
                    assert_eq!(expected, found.inputs_type, "case `{}`", desc)
                }
                (Ok(_), Err(err)) => panic!("case `{}` unexpected error: {:?}", desc, err),
                (Err(_), Ok(f)) => panic!(
                    "case `{}` expect error but found: {:?}",
                    desc, f.inputs_type
                ),
                (Err(expected), Err(err)) => assert!(
                    err.to_string().contains(expected),
                    "case `{}` expect err `{}` != {:?}",
                    desc,
                    expected,
                    err
                ),
            }
        }
    }
}
