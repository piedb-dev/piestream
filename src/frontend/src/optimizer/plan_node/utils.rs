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

use std::collections::HashMap;

use piestream_common::catalog::{ColumnDesc, Field, OrderedColumnDesc};
use piestream_common::types::DataType;
use piestream_common::util::sort_util::OrderType;

use crate::catalog::column_catalog::ColumnCatalog;
use crate::catalog::{TableCatalog, TableId};

#[derive(Default)]
pub struct TableCatalogBuilder {
    columns: Vec<ColumnCatalog>,
    column_names: HashMap<String, i32>,
    order_descs: Vec<OrderedColumnDesc>,
    pk_indices: Vec<usize>,
}

/// For DRY, mainly used for construct internal table catalog in stateful streaming executors.
/// Be careful of the order of add column.
impl TableCatalogBuilder {
    // TODO: Add more fields if internal table is more configurable.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a column from Field info.
    pub fn add_column_desc_from_field(&mut self, order_type: Option<OrderType>, field: &Field) {
        let column_id = self.cur_col_id();
        // Add column desc.
        let mut column_desc = ColumnDesc::from_field_with_column_id(field, column_id);

        // Avoid column name duplicate.
        self.avoid_duplicate_col_name(&mut column_desc);

        self.columns.push(ColumnCatalog {
            column_desc: column_desc.clone(),
            // All columns in internal table are invisible to batch query.
            is_hidden: false,
        });

        // Ordered column desc must be a pk.
        self.add_order_column(column_desc, order_type);
    }

    /// Add a unnamed column.
    pub fn add_unnamed_column(&mut self, order_type: Option<OrderType>, data_type: DataType) {
        let column_id = self.cur_col_id();

        // Add column desc.
        let mut column_desc = ColumnDesc::unnamed(column_id.into(), data_type);

        // Avoid column name duplicate.
        self.avoid_duplicate_col_name(&mut column_desc);

        self.columns.push(ColumnCatalog {
            column_desc: column_desc.clone(),
            // All columns in internal table are invisible to batch query.
            is_hidden: false,
        });

        self.add_order_column(column_desc, order_type);
    }

    /// Check whether need to add a ordered column. Different from value, order desc equal pk in
    /// semantics and they are encoded as storage key.
    fn add_order_column(&mut self, column_desc: ColumnDesc, order_type: Option<OrderType>) {
        if let Some(order) = order_type {
            self.pk_indices
                .push(i32::from(column_desc.column_id) as usize);
            self.order_descs
                .push(OrderedColumnDesc { column_desc, order });
        }
    }

    /// Check the column name whether exist before. if true, record occurrence and change the name
    /// to avoid duplicate.
    fn avoid_duplicate_col_name(&mut self, column_desc: &mut ColumnDesc) {
        let column_name = column_desc.name.clone();
        if let Some(occurrence) = self.column_names.get_mut(&column_name) {
            column_desc.name = format!("{}_{}", column_name, occurrence);
            *occurrence += 1;
        } else {
            self.column_names.insert(column_name, 0);
        }
    }

    /// Consume builder and create `TableCatalog` (for proto).
    pub fn build(self, distribution_keys: Vec<usize>, append_only: bool) -> TableCatalog {
        TableCatalog {
            id: TableId::placeholder(),
            associated_source_id: None,
            name: String::new(),
            columns: self.columns,
            order_desc: self.order_descs,
            pks: self.pk_indices,
            is_index_on: None,
            distribution_keys,
            appendonly: append_only,
            owner: piestream_common::catalog::DEFAULT_SUPPER_USER.to_string(),
            vnode_mapping: None,
            properties: HashMap::default(),
        }
    }

    /// Allocate column id for next column. Column id allocation is always start from 0.
    pub fn cur_col_id(&self) -> i32 {
        self.columns.len() as i32
    }
}
