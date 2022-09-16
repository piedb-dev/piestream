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
use piestream_common::catalog::{ColumnDesc, TableId};
use piestream_common::types::DataType;
use piestream_common::util::sort_util::OrderType;
use piestream_storage::memory::MemoryStateStore;
use piestream_storage::table::state_table::StateTable;
use piestream_storage::table::storage_table::{StorageTable, READ_ONLY};

pub async fn gen_basic_table(row_count: usize) -> StorageTable<MemoryStateStore, READ_ONLY> {
    let state_store = MemoryStateStore::new();

    let order_types = vec![OrderType::Ascending, OrderType::Descending];
    let column_ids = vec![0.into(), 1.into(), 2.into()];
    let column_descs = vec![
        ColumnDesc::unnamed(column_ids[0], DataType::Int32),
        ColumnDesc::unnamed(column_ids[1], DataType::Int32),
        ColumnDesc::unnamed(column_ids[2], DataType::Int32),
    ];
    let pk_indices = vec![0_usize, 1_usize];
    let mut state = StateTable::new(
        state_store,
        TableId::from(0x42),
        column_descs.clone(),
        order_types,
        None,
        pk_indices,
    );
    let table = state.storage_table().clone();
    let epoch: u64 = 0;

    for idx in 0..row_count {
        let idx = idx as i32;
        state
            .insert(Row(vec![
                Some(idx.into()),
                Some(idx.into()),
                Some(idx.into()),
            ]))
            .unwrap();
    }
    state.commit(epoch).await.unwrap();

    table.into()
}
