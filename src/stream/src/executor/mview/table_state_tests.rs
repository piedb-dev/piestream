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

use risingwave_common::array::Row;
use risingwave_common::catalog::{ColumnDesc, ColumnId};
use risingwave_common::types::DataType;
use risingwave_common::util::sort_util::OrderType;
use risingwave_storage::memory::MemoryStateStore;
use risingwave_storage::table::cell_based_table::CellBasedTable;
use risingwave_storage::table::state_table::StateTable;
use risingwave_storage::table::TableIter;
use risingwave_storage::Keyspace;

// use crate::executor::mview::ManagedMViewState;

#[madsim::test]
async fn test_cell_based_table_iter() {
    let state_store = MemoryStateStore::new();
    // let pk_columns = vec![0, 1]; leave a message to indicate pk columns
    let order_types = vec![OrderType::Ascending, OrderType::Descending];
    let keyspace = Keyspace::executor_root(state_store, 0x42);
    let column_ids = vec![ColumnId::from(0), ColumnId::from(1), ColumnId::from(2)];
    let column_descs = vec![
        ColumnDesc::unnamed(column_ids[0], DataType::Int32),
        ColumnDesc::unnamed(column_ids[1], DataType::Int32),
        ColumnDesc::unnamed(column_ids[2], DataType::Int32),
    ];

    let mut state = StateTable::new(keyspace.clone(), column_descs.clone(), order_types.clone());
    let table = CellBasedTable::new_for_test(keyspace.clone(), column_descs, order_types);
    let epoch: u64 = 0;

    state
        .insert(
            Row(vec![Some(1_i32.into()), Some(11_i32.into())]),
            Row(vec![
                Some(1_i32.into()),
                Some(11_i32.into()),
                Some(111_i32.into()),
            ]),
        )
        .unwrap();
    state
        .insert(
            Row(vec![Some(2_i32.into()), Some(22_i32.into())]),
            Row(vec![
                Some(2_i32.into()),
                Some(22_i32.into()),
                Some(222_i32.into()),
            ]),
        )
        .unwrap();
    state
        .delete(
            Row(vec![Some(2_i32.into()), Some(22_i32.into())]),
            Row(vec![
                Some(2_i32.into()),
                Some(22_i32.into()),
                Some(222_i32.into()),
            ]),
        )
        .unwrap();
    state.commit(epoch).await.unwrap();

    let epoch = u64::MAX;
    let mut iter = table.iter(epoch).await.unwrap();

    let res = iter.next().await.unwrap();
    assert!(res.is_some());
    assert_eq!(
        Row(vec![
            Some(1_i32.into()),
            Some(11_i32.into()),
            Some(111_i32.into())
        ]),
        res.unwrap()
    );

    let res = iter.next().await.unwrap();
    assert!(res.is_none());
}

#[madsim::test]
async fn test_multi_cell_based_table_iter() {
    let state_store = MemoryStateStore::new();
    // let pk_columns = vec![0, 1]; leave a message to indicate pk columns
    let order_types = vec![OrderType::Ascending, OrderType::Descending];

    let keyspace_1 = Keyspace::executor_root(state_store.clone(), 0x1111);
    let keyspace_2 = Keyspace::executor_root(state_store.clone(), 0x2222);
    let epoch: u64 = 0;

    let column_ids = vec![ColumnId::from(0), ColumnId::from(1), ColumnId::from(2)];
    let column_descs_1 = vec![
        ColumnDesc::unnamed(column_ids[0], DataType::Int32),
        ColumnDesc::unnamed(column_ids[1], DataType::Int32),
        ColumnDesc::unnamed(column_ids[2], DataType::Int32),
    ];
    let column_descs_2 = vec![
        ColumnDesc::unnamed(column_ids[0], DataType::Varchar),
        ColumnDesc::unnamed(column_ids[1], DataType::Varchar),
        ColumnDesc::unnamed(column_ids[2], DataType::Varchar),
    ];

    let mut state_1 = StateTable::new(
        keyspace_1.clone(),
        column_descs_1.clone(),
        order_types.clone(),
    );
    let mut state_2 = StateTable::new(
        keyspace_2.clone(),
        column_descs_2.clone(),
        order_types.clone(),
    );

    let table_1 =
        CellBasedTable::new_for_test(keyspace_1.clone(), column_descs_1, order_types.clone());
    let table_2 = CellBasedTable::new_for_test(keyspace_2.clone(), column_descs_2, order_types);

    state_1
        .insert(
            Row(vec![Some(1_i32.into()), Some(11_i32.into())]),
            Row(vec![
                Some(1_i32.into()),
                Some(11_i32.into()),
                Some(111_i32.into()),
            ]),
        )
        .unwrap();
    state_1
        .insert(
            Row(vec![Some(2_i32.into()), Some(22_i32.into())]),
            Row(vec![
                Some(2_i32.into()),
                Some(22_i32.into()),
                Some(222_i32.into()),
            ]),
        )
        .unwrap();
    state_1
        .delete(
            Row(vec![Some(2_i32.into()), Some(22_i32.into())]),
            Row(vec![
                Some(2_i32.into()),
                Some(22_i32.into()),
                Some(222_i32.into()),
            ]),
        )
        .unwrap();

    state_2
        .insert(
            Row(vec![
                Some("1".to_string().into()),
                Some("11".to_string().into()),
            ]),
            Row(vec![
                Some("1".to_string().into()),
                Some("11".to_string().into()),
                Some("111".to_string().into()),
            ]),
        )
        .unwrap();
    state_2
        .insert(
            Row(vec![
                Some("2".to_string().into()),
                Some("22".to_string().into()),
            ]),
            Row(vec![
                Some("2".to_string().into()),
                Some("22".to_string().into()),
                Some("222".to_string().into()),
            ]),
        )
        .unwrap();
    state_2
        .delete(
            Row(vec![
                Some("2".to_string().into()),
                Some("22".to_string().into()),
            ]),
            Row(vec![
                Some("2".to_string().into()),
                Some("22".to_string().into()),
                Some("222".to_string().into()),
            ]),
        )
        .unwrap();

    state_1.commit(epoch).await.unwrap();
    state_2.commit(epoch).await.unwrap();

    let mut iter_1 = table_1.iter(epoch).await.unwrap();
    let mut iter_2 = table_2.iter(epoch).await.unwrap();

    let res_1_1 = iter_1.next().await.unwrap();
    assert!(res_1_1.is_some());
    assert_eq!(
        Row(vec![
            Some(1_i32.into()),
            Some(11_i32.into()),
            Some(111_i32.into()),
        ]),
        res_1_1.unwrap()
    );
    let res_1_2 = iter_1.next().await.unwrap();
    assert!(res_1_2.is_none());

    let res_2_1 = iter_2.next().await.unwrap();
    assert!(res_2_1.is_some());
    assert_eq!(
        Row(vec![
            Some("1".to_string().into()),
            Some("11".to_string().into()),
            Some("111".to_string().into())
        ]),
        res_2_1.unwrap()
    );
    let res_2_2 = iter_2.next().await.unwrap();
    assert!(res_2_2.is_none());
}

#[madsim::test]
async fn test_cell_based_scan_empty_column_ids_cardinality() {
    let state_store = MemoryStateStore::new();
    let column_ids = vec![ColumnId::from(0), ColumnId::from(1), ColumnId::from(2)];
    let column_descs = vec![
        ColumnDesc::unnamed(column_ids[0], DataType::Int32),
        ColumnDesc::unnamed(column_ids[1], DataType::Int32),
        ColumnDesc::unnamed(column_ids[2], DataType::Int32),
    ];
    // let pk_columns = vec![0, 1]; leave a message to indicate pk columns
    let order_types = vec![OrderType::Ascending, OrderType::Descending];
    let keyspace = Keyspace::executor_root(state_store, 0x42);

    let mut state = StateTable::new(keyspace.clone(), column_descs.clone(), order_types.clone());
    let table = CellBasedTable::new_for_test(keyspace.clone(), column_descs, order_types);
    let epoch: u64 = 0;

    state
        .insert(
            Row(vec![Some(1_i32.into()), Some(11_i32.into())]),
            Row(vec![
                Some(1_i32.into()),
                Some(11_i32.into()),
                Some(111_i32.into()),
            ]),
        )
        .unwrap();
    state
        .insert(
            Row(vec![Some(2_i32.into()), Some(22_i32.into())]),
            Row(vec![
                Some(2_i32.into()),
                Some(22_i32.into()),
                Some(222_i32.into()),
            ]),
        )
        .unwrap();
    state.commit(epoch).await.unwrap();

    let chunk = {
        let mut iter = table.iter(u64::MAX).await.unwrap();
        iter.collect_data_chunk(&table, None)
            .await
            .unwrap()
            .unwrap()
    };
    assert_eq!(chunk.cardinality(), 2);
}

#[madsim::test]
async fn test_get_row_by_scan() {
    let state_store = MemoryStateStore::new();
    let column_ids = vec![ColumnId::from(0), ColumnId::from(1), ColumnId::from(2)];
    let column_descs = vec![
        ColumnDesc::unnamed(column_ids[0], DataType::Int32),
        ColumnDesc::unnamed(column_ids[1], DataType::Int32),
        ColumnDesc::unnamed(column_ids[2], DataType::Int32),
    ];

    let order_types = vec![OrderType::Ascending, OrderType::Descending];
    let keyspace = Keyspace::executor_root(state_store, 0x42);
    let mut state = StateTable::new(keyspace.clone(), column_descs.clone(), order_types.clone());
    let table = CellBasedTable::new_for_test(keyspace.clone(), column_descs, order_types);
    let epoch: u64 = 0;

    state
        .insert(
            Row(vec![Some(1_i32.into()), Some(11_i32.into())]),
            Row(vec![Some(1_i32.into()), None, None]),
        )
        .unwrap();
    state
        .insert(
            Row(vec![Some(2_i32.into()), Some(22_i32.into())]),
            Row(vec![Some(2_i32.into()), None, Some(222_i32.into())]),
        )
        .unwrap();
    state
        .insert(
            Row(vec![Some(3_i32.into()), Some(33_i32.into())]),
            Row(vec![Some(3_i32.into()), None, None]),
        )
        .unwrap();

    state
        .delete(
            Row(vec![Some(2_i32.into()), Some(22_i32.into())]),
            Row(vec![Some(2_i32.into()), None, Some(222_i32.into())]),
        )
        .unwrap();
    state.commit(epoch).await.unwrap();

    let epoch = u64::MAX;

    let get_row1_res = table
        .get_row_by_scan(&Row(vec![Some(1_i32.into()), Some(11_i32.into())]), epoch)
        .await
        .unwrap();
    assert_eq!(
        get_row1_res,
        Some(Row(vec![Some(1_i32.into()), None, None,]))
    );

    let get_row2_res = table
        .get_row_by_scan(&Row(vec![Some(2_i32.into()), Some(22_i32.into())]), epoch)
        .await
        .unwrap();
    assert_eq!(get_row2_res, None);

    let get_row3_res = table
        .get_row_by_scan(&Row(vec![Some(3_i32.into()), Some(33_i32.into())]), epoch)
        .await
        .unwrap();
    assert_eq!(
        get_row3_res,
        Some(Row(vec![Some(3_i32.into()), None, None]))
    );

    let get_no_exist_res = table
        .get_row_by_scan(&Row(vec![Some(0_i32.into()), Some(00_i32.into())]), epoch)
        .await
        .unwrap();
    assert_eq!(get_no_exist_res, None);
}

#[madsim::test]
async fn test_get_row_by_muti_get() {
    let state_store = MemoryStateStore::new();
    let column_ids = vec![ColumnId::from(0), ColumnId::from(1), ColumnId::from(2)];
    let column_descs = vec![
        ColumnDesc::unnamed(column_ids[0], DataType::Int32),
        ColumnDesc::unnamed(column_ids[1], DataType::Int32),
        ColumnDesc::unnamed(column_ids[2], DataType::Int32),
    ];

    let order_types = vec![OrderType::Ascending, OrderType::Descending];
    let keyspace = Keyspace::executor_root(state_store, 0x42);
    let mut state = StateTable::new(keyspace.clone(), column_descs.clone(), order_types.clone());
    let table = CellBasedTable::new_for_test(keyspace.clone(), column_descs, order_types);
    let epoch: u64 = 0;

    state
        .insert(
            Row(vec![Some(1_i32.into()), Some(11_i32.into())]),
            Row(vec![Some(1_i32.into()), None, None]),
        )
        .unwrap();
    state
        .insert(
            Row(vec![Some(2_i32.into()), Some(22_i32.into())]),
            Row(vec![Some(2_i32.into()), None, Some(222_i32.into())]),
        )
        .unwrap();
    state
        .insert(
            Row(vec![Some(3_i32.into()), Some(33_i32.into())]),
            Row(vec![Some(3_i32.into()), None, None]),
        )
        .unwrap();
    state
        .insert(
            Row(vec![Some(4_i32.into()), Some(44_i32.into())]),
            Row(vec![None, None, None]),
        )
        .unwrap();

    state
        .delete(
            Row(vec![Some(2_i32.into()), Some(22_i32.into())]),
            Row(vec![Some(2_i32.into()), None, Some(222_i32.into())]),
        )
        .unwrap();
    state.commit(epoch).await.unwrap();

    let epoch = u64::MAX;

    let get_row1_res = table
        .get_row(&Row(vec![Some(1_i32.into()), Some(11_i32.into())]), epoch)
        .await
        .unwrap();
    assert_eq!(
        get_row1_res,
        Some(Row(vec![Some(1_i32.into()), None, None,]))
    );

    let get_row2_res = table
        .get_row(&Row(vec![Some(2_i32.into()), Some(22_i32.into())]), epoch)
        .await
        .unwrap();
    assert_eq!(get_row2_res, None);

    let get_row3_res = table
        .get_row(&Row(vec![Some(3_i32.into()), Some(33_i32.into())]), epoch)
        .await
        .unwrap();
    assert_eq!(
        get_row3_res,
        Some(Row(vec![Some(3_i32.into()), None, None]))
    );

    let get_row4_res = table
        .get_row(&Row(vec![Some(4_i32.into()), Some(44_i32.into())]), epoch)
        .await
        .unwrap();
    assert_eq!(get_row4_res, Some(Row(vec![None, None, None])));

    let get_no_exist_res = table
        .get_row(&Row(vec![Some(0_i32.into()), Some(00_i32.into())]), epoch)
        .await
        .unwrap();
    assert_eq!(get_no_exist_res, None);
}

#[madsim::test]
async fn test_get_row_for_string() {
    let state_store = MemoryStateStore::new();
    let order_types = vec![OrderType::Ascending, OrderType::Descending];
    let keyspace = Keyspace::executor_root(state_store, 0x42);
    let column_ids = vec![ColumnId::from(1), ColumnId::from(4), ColumnId::from(7)];
    let column_descs = vec![
        ColumnDesc::unnamed(column_ids[0], DataType::Varchar),
        ColumnDesc::unnamed(column_ids[1], DataType::Varchar),
        ColumnDesc::unnamed(column_ids[2], DataType::Varchar),
    ];
    let mut state = StateTable::new(keyspace.clone(), column_descs.clone(), order_types.clone());
    let table = CellBasedTable::new_for_test(keyspace.clone(), column_descs, order_types);
    let epoch: u64 = 0;

    state
        .insert(
            Row(vec![
                Some("1".to_string().into()),
                Some("11".to_string().into()),
            ]),
            Row(vec![
                Some("1".to_string().into()),
                Some("11".to_string().into()),
                Some("111".to_string().into()),
            ]),
        )
        .unwrap();
    state
        .insert(
            Row(vec![
                Some("4".to_string().into()),
                Some("44".to_string().into()),
            ]),
            Row(vec![
                Some("4".to_string().into()),
                Some("44".to_string().into()),
                Some("444".to_string().into()),
            ]),
        )
        .unwrap();
    state
        .delete(
            Row(vec![
                Some("4".to_string().into()),
                Some("44".to_string().into()),
            ]),
            Row(vec![
                Some("4".to_string().into()),
                Some("44".to_string().into()),
                Some("444".to_string().into()),
            ]),
        )
        .unwrap();
    state.commit(epoch).await.unwrap();

    let epoch = u64::MAX;
    let get_row1_res = table
        .get_row(
            &Row(vec![
                Some("1".to_string().into()),
                Some("11".to_string().into()),
            ]),
            epoch,
        )
        .await
        .unwrap();
    assert_eq!(
        get_row1_res,
        Some(Row(vec![
            Some("1".to_string().into()),
            Some("11".to_string().into()),
            Some("111".to_string().into()),
        ]))
    );

    let get_row2_res = table
        .get_row(
            &Row(vec![
                Some("4".to_string().into()),
                Some("44".to_string().into()),
            ]),
            epoch,
        )
        .await
        .unwrap();
    assert_eq!(get_row2_res, None);
}

#[madsim::test]
async fn test_shuffled_column_id_for_get_row() {
    let state_store = MemoryStateStore::new();
    let column_ids = vec![ColumnId::from(3), ColumnId::from(2), ColumnId::from(1)];
    let column_descs = vec![
        ColumnDesc::unnamed(column_ids[0], DataType::Int32),
        ColumnDesc::unnamed(column_ids[1], DataType::Int32),
        ColumnDesc::unnamed(column_ids[2], DataType::Int32),
    ];

    let order_types = vec![OrderType::Ascending, OrderType::Descending];
    let keyspace = Keyspace::executor_root(state_store, 0x42);
    let mut state = StateTable::new(keyspace.clone(), column_descs.clone(), order_types.clone());
    let table = CellBasedTable::new_for_test(keyspace.clone(), column_descs, order_types);
    let epoch: u64 = 0;

    state
        .insert(
            Row(vec![Some(1_i32.into()), Some(11_i32.into())]),
            Row(vec![Some(1_i32.into()), None, None]),
        )
        .unwrap();
    state
        .insert(
            Row(vec![Some(2_i32.into()), Some(22_i32.into())]),
            Row(vec![Some(2_i32.into()), None, Some(222_i32.into())]),
        )
        .unwrap();
    state
        .insert(
            Row(vec![Some(3_i32.into()), Some(33_i32.into())]),
            Row(vec![Some(3_i32.into()), None, None]),
        )
        .unwrap();
    state
        .insert(
            Row(vec![Some(4_i32.into()), Some(44_i32.into())]),
            Row(vec![None, None, None]),
        )
        .unwrap();

    state
        .delete(
            Row(vec![Some(2_i32.into()), Some(22_i32.into())]),
            Row(vec![Some(2_i32.into()), None, Some(222_i32.into())]),
        )
        .unwrap();
    state.commit(epoch).await.unwrap();

    let epoch = u64::MAX;

    let get_row1_res = table
        .get_row(&Row(vec![Some(1_i32.into()), Some(11_i32.into())]), epoch)
        .await
        .unwrap();
    assert_eq!(
        get_row1_res,
        Some(Row(vec![Some(1_i32.into()), None, None,]))
    );

    let get_row2_res = table
        .get_row(&Row(vec![Some(2_i32.into()), Some(22_i32.into())]), epoch)
        .await
        .unwrap();
    assert_eq!(get_row2_res, None);

    let get_row3_res = table
        .get_row(&Row(vec![Some(3_i32.into()), Some(33_i32.into())]), epoch)
        .await
        .unwrap();
    assert_eq!(
        get_row3_res,
        Some(Row(vec![Some(3_i32.into()), None, None]))
    );

    let get_row4_res = table
        .get_row(&Row(vec![Some(4_i32.into()), Some(44_i32.into())]), epoch)
        .await
        .unwrap();
    assert_eq!(get_row4_res, Some(Row(vec![None, None, None])));

    let get_no_exist_res = table
        .get_row(&Row(vec![Some(0_i32.into()), Some(00_i32.into())]), epoch)
        .await
        .unwrap();
    assert_eq!(get_no_exist_res, None);
}
