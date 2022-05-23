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

use madsim::collections::HashMap;
use risingwave_common::array::Row;
use risingwave_common::catalog::ColumnId;
use risingwave_common::error::Result;
use risingwave_common::util::hash_util::CRC32FastBuilder;
use risingwave_common::util::ordered::*;
use risingwave_common::util::sort_util::OrderType;
use risingwave_storage::storage_value::{StorageValue, ValueMeta};
use risingwave_storage::{Keyspace, StateStore};

use crate::executor::managed_state::flush_status::HashMapFlushStatus as FlushStatus;

/// `ManagedMViewState` buffers recent mutations. Data will be written
/// to backend storage on calling `flush`.
pub struct ManagedMViewState<S: StateStore> {
    keyspace: Keyspace<S>,

    /// Column IDs of each column in the input schema
    column_ids: Vec<ColumnId>,

    /// Ordering of primary key (for assertion)
    order_types: Vec<OrderType>,

    /// Serializer to serialize keys from input rows
    key_serializer: OrderedRowSerializer,

    /// Cached key/values
    cache: HashMap<Row, FlushStatus<Row>>,
}

impl<S: StateStore> ManagedMViewState<S> {
    /// Create a [`ManagedMViewState`].
    pub fn new(
        keyspace: Keyspace<S>,
        column_ids: Vec<ColumnId>,
        order_types: Vec<OrderType>,
    ) -> Self {
        // TODO(eric): refactor this later...
        Self {
            keyspace,
            column_ids,
            cache: HashMap::new(),
            order_types: order_types.clone(),
            key_serializer: OrderedRowSerializer::new(order_types),
        }
    }

    /// Put a key into the managed mview state. `arrange_keys` is composed of group keys and
    /// primary keys.
    pub fn put(&mut self, pk: Row, value: Row) {
        assert_eq!(self.order_types.len(), pk.size());
        assert_eq!(self.column_ids.len(), value.size());

        FlushStatus::do_insert(self.cache.entry(pk), value);
    }

    /// Delete a key from the managed mview state. `arrange_keys` is composed of group keys and
    /// primary keys.
    pub fn delete(&mut self, pk: Row) {
        assert_eq!(self.order_types.len(), pk.size());

        FlushStatus::do_delete(self.cache.entry(pk));
    }

    pub async fn flush(&mut self, epoch: u64) -> Result<()> {
        let mut batch = self.keyspace.state_store().start_write_batch();
        batch.reserve(self.cache.len() * self.column_ids.len());
        let mut local = batch.prefixify(&self.keyspace);
        let hash_builder = CRC32FastBuilder {};

        for (arrange_keys, cells) in self.cache.drain() {
            let row = cells.into_option();
            let arrange_key_buf = serialize_pk(&arrange_keys, &self.key_serializer)?;
            let bytes = serialize_pk_and_row_state(&arrange_key_buf, &row, &self.column_ids)?;

            // We compute vnode on arrange keys in materialized view since materialized views are
            // grouped by arrange keys.
            let vnode = arrange_keys.hash_row(&hash_builder).to_vnode();
            let value_meta = ValueMeta::with_vnode(vnode);

            for (key, value) in bytes {
                match value {
                    Some(val) => local.put(key, StorageValue::new_put(value_meta, val)),
                    None => local.delete_with_value_meta(key, value_meta),
                }
            }
        }
        batch.ingest(epoch).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use risingwave_common::catalog::schema_test_utils;
    use risingwave_common::util::sort_util::OrderType;
    use risingwave_storage::memory::MemoryStateStore;

    use super::*;

    #[madsim::test]
    async fn test_mview_state() {
        // Only assert pk and columns can be successfully put/delete/flush,
        // and the amount of rows is expected.
        let state_store = MemoryStateStore::new();
        let _schema = schema_test_utils::ii();
        let keyspace = Keyspace::executor_root(state_store.clone(), 0x42);

        let mut state = ManagedMViewState::new(
            keyspace.clone(),
            vec![0.into(), 1.into()],
            vec![OrderType::Ascending],
        );
        let mut epoch: u64 = 0;
        state.put(
            Row(vec![Some(1_i32.into())]),
            Row(vec![Some(1_i32.into()), Some(11_i32.into())]),
        );
        state.put(
            Row(vec![Some(2_i32.into())]),
            Row(vec![Some(2_i32.into()), Some(22_i32.into())]),
        );
        state.put(
            Row(vec![Some(3_i32.into())]),
            Row(vec![Some(3_i32.into()), Some(33_i32.into())]),
        );
        state.delete(Row(vec![Some(2_i32.into())]));

        state.flush(epoch).await.unwrap();
        let data = keyspace.scan(None, epoch).await.unwrap();
        // cell-based storage has 6 cells
        assert_eq!(data.len(), 6);

        epoch += 1;
        state.delete(Row(vec![Some(3_i32.into())]));
        state.flush(epoch).await.unwrap();
        let data = keyspace.scan(None, epoch).await.unwrap();
        assert_eq!(data.len(), 3);
    }
}
