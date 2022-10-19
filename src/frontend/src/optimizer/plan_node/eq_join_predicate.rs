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

use std::fmt;

use piestream_common::catalog::Schema;

use crate::expr::{ExprType, FunctionCall, InputRef, InputRefDisplay};
use crate::utils::{ColIndexMapping, Condition, ConditionDisplay};

/// The join predicate used in optimizer
#[derive(Debug, Clone)]
pub struct EqJoinPredicate {
    /// Other conditions, linked with `AND` conjunction.
    other_cond: Condition,

    /// The equal columns indexes(in the input schema) both sides,
    /// the first is from the left table and the second is from the right table.
    /// The third is `null_safe` flag.
    eq_keys: Vec<(InputRef, InputRef, bool)>,

    left_cols_num: usize,
}

impl fmt::Display for EqJoinPredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        let mut eq_keys = self.eq_keys().iter();
        if let Some((k1, k2, null_safe)) = eq_keys.next() {
            write!(
                f,
                "{} {} {}",
                k1,
                if *null_safe {
                    "IS NOT DISTINCT FROM"
                } else {
                    "="
                },
                k2
            )?;
        }
        for (k1, k2, null_safe) in eq_keys {
            write!(
                f,
                "AND {} {} {}",
                k1,
                if *null_safe {
                    "IS NOT DISTINCT FROM"
                } else {
                    "="
                },
                k2
            )?;
        }
        if !self.other_cond.always_true() {
            write!(f, " AND {}", self.other_cond)?;
        }

        Ok(())
    }
}

impl EqJoinPredicate {
    /// The new method for `JoinPredicate` without any analysis, check or rewrite.
    pub fn new(
        other_cond: Condition,
        eq_keys: Vec<(InputRef, InputRef, bool)>,
        left_cols_num: usize,
    ) -> Self {
        Self {
            other_cond,
            eq_keys,
            left_cols_num,
        }
    }

    /// `create` will analyze the on clause condition and construct a `JoinPredicate`.
    /// e.g.
    /// ```sql
    ///   select a.v1, a.v2, b.v1, b.v2 from a,b on a.v1 = a.v2 and a.v1 = b.v1 and a.v2 > b.v2
    /// ```
    /// will call the `create` function with `left_colsnum` = 2 and `on_clause` is (supposed
    /// `input_ref` count start from 0)
    /// ```sql
    /// input_ref(0) = input_ref(1) and input_ref(0) = input_ref(2) and input_ref(1) > input_ref(3)
    /// ```
    /// And the `create functions` should return `JoinPredicate`
    /// ```sql
    ///   other_conds = Vec[input_ref(0) = input_ref(1), input_ref(1) > input_ref(3)],
    ///   keys= Vec[(0,2)]
    /// ```
    pub fn create(left_cols_num: usize, right_cols_num: usize, on_clause: Condition) -> Self {
        let (eq_keys, other_cond) = on_clause.split_eq_keys(left_cols_num, right_cols_num);
        Self::new(other_cond, eq_keys, left_cols_num)
    }

    /// Get join predicate's eq conds.
    pub fn eq_cond(&self) -> Condition {
        Condition {
            conjunctions: self
                .eq_keys
                .iter()
                .cloned()
                .map(|(l, r, null_safe)| {
                    FunctionCall::new(
                        if null_safe {
                            ExprType::IsNotDistinctFrom
                        } else {
                            ExprType::Equal
                        },
                        vec![l.into(), r.into()],
                    )
                    .unwrap()
                    .into()
                })
                .collect(),
        }
    }

    pub fn non_eq_cond(&self) -> Condition {
        self.other_cond.clone()
    }

    pub fn all_cond(&self) -> Condition {
        let cond = self.eq_cond();
        cond.and(self.non_eq_cond())
    }

    pub fn has_eq(&self) -> bool {
        !self.eq_keys.is_empty()
    }

    pub fn has_non_eq(&self) -> bool {
        !self.other_cond.always_true()
    }

    /// Get a reference to the join predicate's other cond.
    pub fn other_cond(&self) -> &Condition {
        &self.other_cond
    }

    /// Get a mutable reference to the join predicate's other cond.
    pub fn other_cond_mut(&mut self) -> &mut Condition {
        &mut self.other_cond
    }

    /// Get a reference to the join predicate's eq keys.
    pub fn eq_keys(&self) -> &[(InputRef, InputRef, bool)] {
        self.eq_keys.as_ref()
    }

    pub fn eq_indexes(&self) -> Vec<(usize, usize)> {
        self.eq_keys
            .iter()
            .map(|(left, right, _)| (left.index(), right.index() - self.left_cols_num))
            .collect()
    }

    pub fn left_eq_indexes(&self) -> Vec<usize> {
        self.eq_keys
            .iter()
            .map(|(left, _, _)| left.index())
            .collect()
    }

    /// return the eq keys column index **based on the right input schema**
    pub fn right_eq_indexes(&self) -> Vec<usize> {
        self.eq_keys
            .iter()
            .map(|(_, right, _)| right.index() - self.left_cols_num)
            .collect()
    }

    pub fn null_safes(&self) -> Vec<bool> {
        self.eq_keys
            .iter()
            .map(|(_, _, null_safe)| *null_safe)
            .collect()
    }

    /// return the eq columns index mapping from right inputs to left inputs
    pub fn r2l_eq_columns_mapping(
        &self,
        left_cols_num: usize,
        right_cols_num: usize,
    ) -> ColIndexMapping {
        let mut map = vec![None; right_cols_num];
        for (left, right, _) in self.eq_keys() {
            map[right.index - left_cols_num] = Some(left.index);
        }
        ColIndexMapping::new(map)
    }
}

pub struct EqJoinPredicateDisplay<'a> {
    pub eq_join_predicate: &'a EqJoinPredicate,
    pub input_schema: &'a Schema,
}

impl EqJoinPredicateDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        let that = self.eq_join_predicate;
        let mut eq_keys = that.eq_keys().iter();
        if let Some((k1, k2, null_safe)) = eq_keys.next() {
            write!(
                f,
                "{} {} {}",
                InputRefDisplay {
                    input_ref: k1,
                    input_schema: self.input_schema
                },
                if *null_safe {
                    "IS NOT DISTINCT FROM"
                } else {
                    "="
                },
                InputRefDisplay {
                    input_ref: k2,
                    input_schema: self.input_schema
                }
            )?;
        }
        for (k1, k2, null_safe) in eq_keys {
            write!(
                f,
                " AND {} {} {}",
                InputRefDisplay {
                    input_ref: k1,
                    input_schema: self.input_schema
                },
                if *null_safe {
                    "IS NOT DISTINCT FROM"
                } else {
                    "="
                },
                InputRefDisplay {
                    input_ref: k2,
                    input_schema: self.input_schema
                }
            )?;
        }
        if !that.other_cond.always_true() {
            write!(
                f,
                " AND {}",
                ConditionDisplay {
                    condition: &that.other_cond,
                    input_schema: self.input_schema
                }
            )?;
        }

        Ok(())
    }
}

impl fmt::Display for EqJoinPredicateDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

impl fmt::Debug for EqJoinPredicateDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}
