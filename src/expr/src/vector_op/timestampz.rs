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

use num_traits::ToPrimitive;
use piestream_common::types::OrderedF64;

use crate::{ExprError, Result};

#[inline(always)]
pub fn f64_sec_to_timestampz(elem: OrderedF64) -> Result<i64> {
    // TODO(#4515): handle +/- infinity
    (elem * 1e6)
        .round() // TODO(#5576): should round to even
        .to_i64()
        .ok_or(ExprError::NumericOutOfRange)
}
