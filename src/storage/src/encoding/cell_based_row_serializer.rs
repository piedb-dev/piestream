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

use itertools::Itertools;
use piestream_common::array::Row;
use piestream_common::catalog::{ColumnDesc, ColumnId};
use piestream_common::error::Result;
use piestream_common::types::VirtualNode;
use piestream_common::util::ordered::serialize_pk_and_row;

use crate::encoding::{Encoding, KeyBytes, ValueBytes};

#[derive(Clone)]
pub struct CellBasedRowSerializer {
    column_ids: Vec<ColumnId>,
}

impl CellBasedRowSerializer {
    pub fn new(column_ids: Vec<ColumnId>) -> Self {
        Self { column_ids }
    }
}

impl Encoding for CellBasedRowSerializer {
    fn create_cell_based_serializer(
        _pk_indices: &[usize],
        _column_descs: &[ColumnDesc],
        column_ids: &[ColumnId],
    ) -> Self {
        Self {
            column_ids: column_ids.to_vec(),
        }
    }

    /// Serialize key and value. The `row` must be in the same order with the column ids in this
    /// serializer.
    fn cell_based_serialize(
        &mut self,
        vnode: VirtualNode,
        pk: &[u8],
        row: Row,
    ) -> Result<Vec<(KeyBytes, ValueBytes)>> {
        // TODO: avoid this allocation
        let key = [vnode.to_be_bytes().as_slice(), pk].concat();
        //flatten回去掉some层
        /*
            [Some(([0, 1, 128, 0, 0, 1, 128, 0, 0, 0], [1, 0, 0, 0])), Some(([0, 1, 128, 0, 0, 1, 128, 0, 0, 1], [11, 0, 0, 0])), Some(([0, 1, 128, 0, 0, 1, 128, 0, 0, 2], [111, 0, 0, 0])), Some(([0, 1, 128, 0, 0, 1, 127, 255, 255, 255], []))]
            执行into_iter()。flatten()。collect_vec()
            [([0, 1, 128, 0, 0, 1, 128, 0, 0, 0], [1, 0, 0, 0]), ([0, 1, 128, 0, 0, 1, 128, 0, 0, 1], [11, 0, 0, 0]), ([0, 1, 128, 0, 0, 1, 128, 0, 0, 2], [111, 0, 0, 0]), ([0, 1, 128, 0, 0, 1, 127, 255, 255, 255], [])]
result=[Some(([0, 1, 128, 0, 0, 3, 128, 0, 0, 0], [3, 0, 0, 0])), Some(([0, 1, 128, 0, 0, 3, 128, 0, 0, 1], [33, 0, 0, 0])), Some(([0, 1, 128, 0, 0, 3, 128, 0, 0, 2], [77, 1, 0, 0])), Some(([0, 1, 128, 0, 0, 3, 127, 255, 255, 255], []))]
        */
        let res = serialize_pk_and_row(&key, &row, &self.column_ids)?
            .into_iter()
            .flatten()
            .collect_vec();
        println!("********res={:?}", res);
        Ok(res)
    }

    /// Serialize key and value. Each column id will occupy a position in Vec. For `column_ids` that
    /// doesn't correspond to a cell, the position will be None. Aparts from user-specified
    /// `column_ids`, there will also be a `SENTINEL_CELL_ID` at the end.
    fn cell_based_serialize_without_filter(
        &mut self,
        vnode: VirtualNode,
        pk: &[u8],
        row: Row,
    ) -> Result<Vec<Option<(KeyBytes, ValueBytes)>>> {
        // TODO: avoid this allocation 
        //key带上了vnode信息
        let key = [vnode.to_be_bytes().as_slice(), pk].concat();
        let res = serialize_pk_and_row(&key, &row, &self.column_ids)?;
        Ok(res)
    }

    /// Get column ids used by cell serializer to serialize.
    /// TODO: This should probably not be exposed to user.
    fn column_ids(&self) -> &[ColumnId] {
        &self.column_ids
    }
}
