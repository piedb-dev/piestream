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

use std::borrow::Cow;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::ops::{Index, RangeBounds};

use futures::{pin_mut, Stream, StreamExt};
use futures_async_stream::try_stream;
use piestream_common::array::Row;
use piestream_common::catalog::{ColumnDesc, TableId};
use piestream_common::util::ordered::{serialize_pk, OrderedRowSerializer};
use piestream_common::util::sort_util::OrderType;
use piestream_hummock_sdk::key::range_of_prefix;

use super::mem_table::{MemTable, RowOp};
use super::storage_table::{StorageTableBase, READ_WRITE};
use super::Distribution;
use crate::encoding::cell_based_row_serializer::CellBasedRowSerializer;
use crate::encoding::dedup_pk_cell_based_row_serializer::DedupPkCellBasedRowSerializer;
use crate::encoding::Encoding;
use crate::error::{StorageError, StorageResult};
use crate::StateStore;

/// Identical to `StateTable`. Used when we want to
/// rows to have dedup pk cell encoding.
pub type DedupPkStateTable<S> = StateTableBase<S, DedupPkCellBasedRowSerializer>;

/// `StateTable` is the interface accessing relational data in KV(`StateStore`) with encoding.
pub type StateTable<S> = StateTableBase<S, CellBasedRowSerializer>;

/// `StateTableBase` is the interface accessing relational data in KV(`StateStore`) with
/// encoding, using `RowSerializer` for row to cell serializing.
#[derive(Clone)]
pub struct StateTableBase<S: StateStore, E: Encoding> {
    /// buffer key/values
    mem_table: MemTable,

    /// Relation layer
    storage_table: StorageTableBase<S, E, READ_WRITE>,
}

impl<S: StateStore, E: Encoding> StateTableBase<S, E> {
    /// Note: `dist_key_indices` is ignored, use `new_with[out]_distribution` instead.
    // TODO: remove this after all state table usages are replaced by `new_with[out]_distribution`.
    pub fn new(
        store: S,
        table_id: TableId,
        columns: Vec<ColumnDesc>,
        order_types: Vec<OrderType>,
        _dist_key_indices: Option<Vec<usize>>,
        pk_indices: Vec<usize>,
    ) -> Self {
        Self::new_without_distribution(store, table_id, columns, order_types, pk_indices)
    }

    /// Create a state table without distribution, used for singleton executors and tests.
    pub fn new_without_distribution(
        store: S,
        table_id: TableId,
        columns: Vec<ColumnDesc>,
        order_types: Vec<OrderType>,
        pk_indices: Vec<usize>,
    ) -> Self {
        Self::new_with_distribution(
            store,
            table_id,
            columns,
            order_types,
            pk_indices,
            Distribution::fallback(),
        )
    }

    /// Create a state table with distribution specified with `distribution`. Should use
    /// `Distribution::fallback()` for singleton executors and tests.
    pub fn new_with_distribution(
        store: S,
        table_id: TableId,
        columns: Vec<ColumnDesc>,
        order_types: Vec<OrderType>,
        pk_indices: Vec<usize>,
        distribution: Distribution,
    ) -> Self {
        Self {
            mem_table: MemTable::new(),
            storage_table: StorageTableBase::new(
                store,
                table_id,
                columns,
                order_types,
                pk_indices,
                distribution,
            ),
        }
    }

    /// Get the underlying [` StorageTableBase`]. Should only be used for tests.
    pub fn storage_table(&self) -> &StorageTableBase<S, E, READ_WRITE> {
        &self.storage_table
    }

    fn pk_serializer(&self) -> &OrderedRowSerializer {
        self.storage_table.pk_serializer()
    }

    // TODO: remove, should not be exposed to user
    pub fn pk_indices(&self) -> &[usize] {
        self.storage_table.pk_indices()
    }

    pub fn is_dirty(&self) -> bool {
        self.mem_table.is_dirty()
    }

    /// Get a single row from state table. This function will return a Cow. If the value is from
    /// memtable, it will be a [`Cow::Borrowed`]. If is from storage table, it will be an owned
    /// value. To convert `Option<Cow<Row>>` to `Option<Row>`, just call `into_owned`.
    pub async fn get_row<'a>(
        &'a self,
        pk: &'a Row,
        epoch: u64,
    ) -> StorageResult<Option<Cow<'a, Row>>> {
        let pk_bytes = serialize_pk(pk, self.pk_serializer());
        let mem_table_res = self.mem_table.get_row_op(&pk_bytes);
        //数据在本地状态表是不需要epoch
        match mem_table_res {
            Some(row_op) => match row_op {
                RowOp::Insert(row) => Ok(Some(Cow::Borrowed(row))),
                RowOp::Delete(_) => Ok(None),
                RowOp::Update((_, row)) => Ok(Some(Cow::Borrowed(row))),
            },
            //状态表中没查询到，在查存储表
            None => Ok(self.storage_table.get_row(pk, epoch).await?.map(Cow::Owned)),
        }
    }

    ///通过pk字段值来查询
    pub async fn get_owned_row(&self, pk: &Row, epoch: u64) -> StorageResult<Option<Row>> {
        Ok(self.get_row(pk, epoch).await?.map(|r| r.into_owned()))
    }

    /// Insert a row into state table. Must provide a full row corresponding to the column desc of
    /// the table.
    pub fn insert(&mut self, value: Row) -> StorageResult<()> {
        let mut datums = vec![];
        for pk_index in self.pk_indices() {
            datums.push(value.index(*pk_index).clone());
        }
        //pk值封装成row
        let pk = Row::new(datums);
        //pk_serializer()是排序方式，升序还是降序， serialize_pk pk字段值增加是否有值标志位，并序列化
        let pk_bytes = serialize_pk(&pk, self.pk_serializer());
        println!("pk_bytes()={:?} pk_bytes.len()={:?}", pk_bytes, pk_bytes.len());
        self.mem_table.insert(pk_bytes, value);
        Ok(())
    }

    /// Insert a row into state table. Must provide a full row of old value corresponding to the
    /// column desc of the table.
    pub fn delete(&mut self, old_value: Row) -> StorageResult<()> {
        let mut datums = vec![];
        for pk_index in self.pk_indices() {
            datums.push(old_value.index(*pk_index).clone());
        }
        let pk = Row::new(datums);
        let pk_bytes = serialize_pk(&pk, self.pk_serializer());
        self.mem_table.delete(pk_bytes, old_value);
        Ok(())
    }

    /// Update a row. The old and new value should have the same pk.
    pub fn update(&mut self, old_value: Row, new_value: Row) -> StorageResult<()> {
        let pk = old_value.by_indices(self.pk_indices());
        debug_assert_eq!(pk, new_value.by_indices(self.pk_indices()));
        let pk_bytes = serialize_pk(&pk, self.pk_serializer());
        self.mem_table.update(pk_bytes, old_value, new_value);
        Ok(())
    }

    //状态表内容提交到存储表
    pub async fn commit(&mut self, new_epoch: u64) -> StorageResult<()> {
        let mem_table = std::mem::take(&mut self.mem_table).into_parts();
        self.storage_table
            .batch_write_rows(mem_table, new_epoch)
            .await?;
        Ok(())
    }
}

/// Iterator functions.
impl<S: StateStore> StateTable<S> {
    async fn iter_with_encoded_key_range<'a, R>(
        &'a self,
        encoded_key_range: R,
        epoch: u64,
    ) -> StorageResult<RowStream<'a, S>>
    where
        R: RangeBounds<Vec<u8>> + Send + Clone + 'a,
    {
        let storage_table_iter = self
            .storage_table
            .streaming_iter_with_encoded_key_range(encoded_key_range.clone(), epoch)
            .await?;
        let mem_table_iter = self.mem_table.iter(encoded_key_range);

        Ok(StateTableRowIter::new(mem_table_iter, storage_table_iter).into_stream())
    }

    /// This function scans rows from the relational table.
    pub async fn iter(&self, epoch: u64) -> StorageResult<RowStream<'_, S>> {
        self.iter_with_pk_bounds::<_, Row>(.., epoch).await
    }

    /// This function scans rows from the relational table with specific `pk_bounds`.
    pub async fn iter_with_pk_bounds<R, B>(
        &self,
        pk_bounds: R,
        epoch: u64,
    ) -> StorageResult<RowStream<'_, S>>
    where
        R: RangeBounds<B> + Send + Clone + 'static,
        B: AsRef<Row> + Send + Clone + 'static,
    {
        
        let encoded_start_key = pk_bounds
            .start_bound()
            .map(|pk| serialize_pk(pk.as_ref(), self.pk_serializer()));
        println!("encoded_start_key={:?}", encoded_start_key);
        let encoded_end_key = pk_bounds
            .end_bound()
            .map(|pk| serialize_pk(pk.as_ref(), self.pk_serializer()));
        println!("encoded_end_key={:?}", encoded_end_key);
        let encoded_key_range = (encoded_start_key, encoded_end_key);
        println!("encoded_key_range={:?}", encoded_key_range);
        self.iter_with_encoded_key_range(encoded_key_range, epoch)
            .await
    }

    /// This function scans rows from the relational table with specific `pk_prefix`.
    pub async fn iter_with_pk_prefix<'a>(
        &'a self,
        pk_prefix: &'a Row,
        epoch: u64,
    ) -> StorageResult<RowStream<'a, S>> {
        let prefix_serializer = self.pk_serializer().prefix(pk_prefix.size());
        let encoded_prefix = serialize_pk(pk_prefix, &prefix_serializer);
        let encoded_key_range = range_of_prefix(&encoded_prefix);

        self.iter_with_encoded_key_range(encoded_key_range, epoch)
            .await
    }
}

pub type RowStream<'a, S: StateStore> = impl Stream<Item = StorageResult<Cow<'a, Row>>>;

struct StateTableRowIter<'a, M, C> {
    mem_table_iter: M,
    storage_table_iter: C,
    _phantom: PhantomData<&'a ()>,
}

/// `StateTableRowIter` is able to read the just written data (uncommited data).
/// It will merge the result of `mem_table_iter` and `storage_streaming_iter`.
impl<'a, M, C> StateTableRowIter<'a, M, C>
where
    M: Iterator<Item = (&'a Vec<u8>, &'a RowOp)>,
    C: Stream<Item = StorageResult<(Vec<u8>, Row)>>,
{
    fn new(mem_table_iter: M, storage_table_iter: C) -> Self {
        Self {
            mem_table_iter,
            storage_table_iter,
            _phantom: PhantomData,
        }
    }

    /// This function scans kv pairs from the `shared_storage`(`storage_table`) and
    /// memory(`mem_table`) with optional pk_bounds. If pk_bounds is
    /// (Included(prefix),Excluded(next_key(prefix))), all kv pairs within corresponding prefix will
    /// be scanned. If a record exist in both `storage_table` and `mem_table`, result
    /// `mem_table` is returned according to the operation(RowOp) on it.
    #[try_stream(ok = Cow<'a, Row>, error = StorageError)]
    async fn into_stream(self) {
        let storage_table_iter = self.storage_table_iter.fuse().peekable();
        pin_mut!(storage_table_iter);

        let mut mem_table_iter = self.mem_table_iter.fuse().peekable();

        loop {
            match (
                storage_table_iter.as_mut().peek().await,
                mem_table_iter.peek(),
            ) {
                (None, None) => break,
                // The mem table side has come to an end, return data from the shared storage.
                (Some(_), None) => {
                    let (_, row) = storage_table_iter.next().await.unwrap()?;
                    yield Cow::Owned(row);
                }
                // The stream side has come to an end, return data from the mem table.
                (None, Some(_)) => {
                    let (_, row_op) = mem_table_iter.next().unwrap();
                    match row_op {
                        RowOp::Insert(row) | RowOp::Update((_, row)) => {
                            yield Cow::Borrowed(row);
                        }
                        _ => {}
                    }
                }
                (Some(Ok((storage_pk, _))), Some((mem_table_pk, _))) => {
                    match storage_pk.cmp(mem_table_pk) {
                        Ordering::Less => {
                            // yield data from storage table
                            let (_, row) = storage_table_iter.next().await.unwrap()?;
                            yield Cow::Owned(row);
                        }
                        Ordering::Equal => {
                            // both memtable and storage contain the key, so we advance both
                            // iterators and return the data in memory.
                            let (_, row_op) = mem_table_iter.next().unwrap();
                            let (_, old_row_in_storage) =
                                storage_table_iter.next().await.unwrap()?;
                            match row_op {
                                RowOp::Insert(row) => {
                                    yield Cow::Borrowed(row);
                                }
                                RowOp::Delete(_) => {}
                                RowOp::Update((old_row, new_row)) => {
                                    debug_assert!(old_row == &old_row_in_storage);
                                    yield Cow::Borrowed(new_row);
                                }
                            }
                        }
                        Ordering::Greater => {
                            // yield data from mem table
                            let (_, row_op) = mem_table_iter.next().unwrap();
                            match row_op {
                                RowOp::Insert(row) => {
                                    yield Cow::Borrowed(row);
                                }
                                RowOp::Delete(_) => {}
                                RowOp::Update(_) => unreachable!(
                                    "memtable update should always be paired with a storage key"
                                ),
                            }
                        }
                    }
                }
                (Some(Err(_)), Some(_)) => {
                    // Throw the error.
                    return Err(storage_table_iter.next().await.unwrap().unwrap_err());
                }
            }
        }
    }
}
