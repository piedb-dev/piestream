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

use std::collections::HashMap;

use piestream_common::catalog::{valid_table_name, TableId, PG_CATALOG_SCHEMA_NAME};
use piestream_pb::catalog::{
    Schema as ProstSchema, Sink as ProstSink, Source as ProstSource, Table as ProstTable,
};
use piestream_pb::stream_plan::source_node::SourceType;

use super::source_catalog::SourceCatalog;
use crate::catalog::system_catalog::SystemCatalog;
use crate::catalog::table_catalog::TableCatalog;
use crate::catalog::SchemaId;

pub type SourceId = u32;
pub type SinkId = u32;

#[derive(Clone, Debug)]
pub struct SchemaCatalog {
    id: SchemaId,
    name: String,
    table_by_name: HashMap<String, TableCatalog>,
    table_name_by_id: HashMap<TableId, String>,
    source_by_name: HashMap<String, SourceCatalog>,
    source_name_by_id: HashMap<SourceId, String>,

    // TODO(nanderstabel): Can be replaced with a bijective map: https://crates.io/crates/bimap.
    sink_id_by_name: HashMap<String, SinkId>,
    sink_name_by_id: HashMap<SinkId, String>,

    // This field only available when schema is "pg_catalog". Meanwhile, others will be empty.
    system_table_by_name: HashMap<String, SystemCatalog>,
    owner: String,
}

impl SchemaCatalog {
    pub fn create_table(&mut self, prost: &ProstTable) {
        let name = prost.name.clone();
        let id = prost.id.into();
        let table: TableCatalog = prost.into();

        self.table_by_name.try_insert(name.clone(), table).unwrap();
        self.table_name_by_id.try_insert(id, name).unwrap();
    }

    pub fn create_sys_table(&mut self, sys_table: SystemCatalog) {
        assert_eq!(self.name, PG_CATALOG_SCHEMA_NAME);
        self.system_table_by_name
            .try_insert(sys_table.name.clone(), sys_table)
            .unwrap();
    }

    pub fn drop_table(&mut self, id: TableId) {
        let name = self.table_name_by_id.remove(&id).unwrap();
        self.table_by_name.remove(&name).unwrap();
    }

    pub fn create_source(&mut self, prost: ProstSource) {
        let name = prost.name.clone();
        let id = prost.id;

        self.source_by_name
            .try_insert(name.clone(), SourceCatalog::from(&prost))
            .unwrap();
        self.source_name_by_id.try_insert(id, name).unwrap();
    }

    pub fn drop_source(&mut self, id: SourceId) {
        let name = self.source_name_by_id.remove(&id).unwrap();
        self.source_by_name.remove(&name).unwrap();
    }

    pub fn create_sink(&mut self, prost: ProstSink) {
        let name = prost.name.clone();
        let id = prost.id;

        self.sink_id_by_name.try_insert(name.clone(), id).unwrap();
        self.sink_name_by_id.try_insert(id, name).unwrap();
    }

    pub fn drop_sink(&mut self, id: SinkId) {
        let name = self.sink_name_by_id.remove(&id).unwrap();
        self.sink_id_by_name.remove(&name).unwrap();
    }

    pub fn iter_table(&self) -> impl Iterator<Item = &TableCatalog> {
        self.table_by_name
            .iter()
            .filter(|(_, v)| {
                // Internally, a table with an associated source can be
                // MATERIALIZED SOURCE or TABLE.
                v.associated_source_id.is_some()
                    && self.get_source_by_name(v.name()).unwrap().source_type == SourceType::Table
            })
            .map(|(_, v)| v)
    }

    /// Iterate all materialized views, excluding the indexs.
    pub fn iter_mv(&self) -> impl Iterator<Item = &TableCatalog> {
        self.table_by_name
            .iter()
            .filter(|(_, v)| {
                v.associated_source_id.is_none()
                    && v.is_index_on.is_none()
                    && valid_table_name(&v.name)
            })
            .map(|(_, v)| v)
    }

    /// Iterate all indexs, excluding the materialized views.
    pub fn iter_index(&self) -> impl Iterator<Item = &TableCatalog> {
        self.table_by_name
            .iter()
            .filter(|(_, v)| v.associated_source_id.is_none() && v.is_index_on.is_some())
            .map(|(_, v)| v)
    }

    /// Iterate all sources, including the materialized sources.
    pub fn iter_source(&self) -> impl Iterator<Item = &SourceCatalog> {
        self.source_by_name
            .iter()
            .filter(|(_, v)| matches!(v.source_type, SourceType::Source))
            .map(|(_, v)| v)
    }

    /// Iterate the materialized sources.
    pub fn iter_materialized_source(&self) -> impl Iterator<Item = &SourceCatalog> {
        self.source_by_name
            .iter()
            .filter(|(name, v)| {
                matches!(v.source_type, SourceType::Source)
                    && self.table_by_name.get(*name).is_some()
            })
            .map(|(_, v)| v)
    }

    /// Iterate all sinks.
    pub fn iter_sink(&self) -> impl Iterator<Item = &u32> {
        self.sink_id_by_name.iter().map(|(_, v)| v)
    }

    pub fn iter_system_tables(&self) -> impl Iterator<Item = &SystemCatalog> {
        self.system_table_by_name.iter().map(|(_, v)| v)
    }

    pub fn get_table_by_name(&self, table_name: &str) -> Option<&TableCatalog> {
        self.table_by_name.get(table_name)
    }

    pub fn get_source_by_name(&self, source_name: &str) -> Option<&SourceCatalog> {
        self.source_by_name.get(source_name)
    }

    pub fn get_sink_id_by_name(&self, sink_name: &str) -> Option<SinkId> {
        self.sink_id_by_name.get(sink_name).cloned()
    }

    pub fn get_system_table_by_name(&self, table_name: &str) -> Option<&SystemCatalog> {
        self.system_table_by_name.get(table_name)
    }

    pub fn id(&self) -> SchemaId {
        self.id
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn owner(&self) -> String {
        self.owner.clone()
    }
}

impl From<&ProstSchema> for SchemaCatalog {
    fn from(schema: &ProstSchema) -> Self {
        Self {
            id: schema.id,
            name: schema.name.clone(),
            table_by_name: HashMap::new(),
            table_name_by_id: HashMap::new(),
            source_by_name: HashMap::new(),
            source_name_by_id: HashMap::new(),
            sink_id_by_name: HashMap::new(),
            sink_name_by_id: HashMap::new(),
            system_table_by_name: HashMap::new(),
            owner: schema.owner.clone(),
        }
    }
}
