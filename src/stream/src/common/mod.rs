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

pub use builder::*;
pub use column_mapping::*;
pub use infallible_expr::*;
use piestream_common::array::Row;
use piestream_storage::table::streaming_table::state_table::{RowStream, StateTable};
use piestream_storage::StateStore;

use crate::executor::StreamExecutorResult;

mod builder;
mod column_mapping;
mod infallible_expr;

pub async fn iter_state_table<'a, S: StateStore>(
    state_table: &'a StateTable<S>,
    prefix: Option<&'a Row>,
) -> StreamExecutorResult<RowStream<'a, S>> {
    Ok(if let Some(group_key) = prefix {
        state_table.iter_with_pk_prefix(group_key).await?
    } else {
        state_table.iter().await?
    })
}
