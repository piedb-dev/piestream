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
use std::{fmt, vec};

use itertools::Itertools;
use piestream_common::catalog::{ColumnDesc, Field, Schema};
use piestream_common::util::sort_util::OrderType;

use crate::catalog::column_catalog::ColumnCatalog;
use crate::catalog::{FragmentId, TableCatalog, TableId};
use crate::optimizer::property::{Direction, FieldOrder};
use crate::utils::WithOptions;

#[derive(Default)]
pub struct TableCatalogBuilder {
    /// All columns in this table
    columns: Vec<ColumnCatalog>,
    pk: Vec<FieldOrder>,
    properties: WithOptions,
    value_indices: Option<Vec<usize>>,
    vnode_col_idx: Option<usize>,
    column_names: HashMap<String, i32>,
}

/// For DRY, mainly used for construct internal table catalog in stateful streaming executors.
/// Be careful of the order of add column.
impl TableCatalogBuilder {
    // TODO: Add more fields if internal table is more configurable.
    pub fn new(properties: WithOptions) -> Self {
        Self {
            properties,
            ..Default::default()
        }
    }

    /// Add a column from Field info, return the column index of the table
    pub fn add_column(&mut self, field: &Field) -> usize {
        let column_idx = self.columns.len();
        let column_id = column_idx as i32;
        // Add column desc.
        let mut column_desc = ColumnDesc::from_field_with_column_id(field, column_id);

        // Avoid column name duplicate.
        self.avoid_duplicate_col_name(&mut column_desc);

        self.columns.push(ColumnCatalog {
            column_desc: column_desc.clone(),
            // All columns in internal table are invisible to batch query.
            is_hidden: false,
        });
        column_idx
    }

    /// Check whether need to add a ordered column. Different from value, order desc equal pk in
    /// semantics and they are encoded as storage key.
    pub fn add_order_column(&mut self, index: usize, order_type: OrderType) {
        self.pk.push(FieldOrder {
            index,
            direct: match order_type {
                OrderType::Ascending => Direction::Asc,
                OrderType::Descending => Direction::Desc,
            },
        });
    }

    pub fn set_vnode_col_idx(&mut self, vnode_col_idx: usize) {
        self.vnode_col_idx = Some(vnode_col_idx);
    }

    pub fn set_value_indices(&mut self, value_indices: Vec<usize>) {
        self.value_indices = Some(value_indices);
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
    pub fn build(self, distribution_key: Vec<usize>) -> TableCatalog {
        TableCatalog {
            id: TableId::placeholder(),
            associated_source_id: None,
            name: String::new(),
            columns: self.columns.clone(),
            pk: self.pk,
            stream_key: vec![],
            distribution_key,
            is_index: false,
            appendonly: false,
            owner: piestream_common::catalog::DEFAULT_SUPER_USER_ID,
            properties: self.properties,
            // TODO(zehua): replace it with FragmentId::placeholder()
            fragment_id: FragmentId::MAX - 1,
            vnode_col_idx: self.vnode_col_idx,
            value_indices: self
                .value_indices
                .unwrap_or_else(|| (0..self.columns.len()).collect_vec()),
            definition: "".into(),
        }
    }

    pub fn columns(&self) -> &[ColumnCatalog] {
        &self.columns
    }
}

#[derive(Clone, Copy)]
pub struct IndicesDisplay<'a> {
    pub indices: &'a [usize],
    pub input_schema: &'a Schema,
}

impl fmt::Display for IndicesDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl fmt::Debug for IndicesDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_list();
        for i in self.indices {
            f.entry(&format_args!(
                "{}",
                self.input_schema.fields.get(*i).unwrap().name
            ));
        }
        f.finish()
    }
}
