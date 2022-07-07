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

pub mod pg_cast;
pub mod pg_matviews_info;
pub mod pg_namespace;
pub mod pg_type;

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use itertools::Itertools;
use piestream_common::array::Row;
use piestream_common::catalog::{ColumnDesc, SysCatalogReader, TableId, DEFAULT_SUPPER_USER};
use piestream_common::error::{ErrorCode, Result};
use piestream_common::types::{DataType, ScalarImpl};
use serde_json::json;

use crate::catalog::catalog_service::CatalogReader;
use crate::catalog::column_catalog::ColumnCatalog;
use crate::catalog::pg_catalog::pg_cast::*;
use crate::catalog::pg_catalog::pg_matviews_info::*;
use crate::catalog::pg_catalog::pg_namespace::*;
use crate::catalog::pg_catalog::pg_type::*;
use crate::catalog::system_catalog::SystemCatalog;
use crate::meta_client::FrontendMetaClient;
use crate::scheduler::worker_node_manager::WorkerNodeManagerRef;
use crate::session::AuthContext;
use crate::user::user_service::UserInfoReader;

#[allow(dead_code)]
pub struct SysCatalogReaderImpl {
    // Read catalog info: database/schema/source/table.
    catalog_reader: CatalogReader,
    // Read user info.
    user_info_reader: UserInfoReader,
    // Read cluster info.
    worker_node_manager: WorkerNodeManagerRef,
    // Read from meta.
    meta_client: Arc<dyn FrontendMetaClient>,
    auth_context: Arc<AuthContext>,
}

impl SysCatalogReaderImpl {
    pub fn new(
        catalog_reader: CatalogReader,
        user_info_reader: UserInfoReader,
        worker_node_manager: WorkerNodeManagerRef,
        meta_client: Arc<dyn FrontendMetaClient>,
        auth_context: Arc<AuthContext>,
    ) -> Self {
        Self {
            catalog_reader,
            user_info_reader,
            worker_node_manager,
            meta_client,
            auth_context,
        }
    }
}

#[async_trait]
impl SysCatalogReader for SysCatalogReaderImpl {
    async fn read_table(&self, table_name: &str) -> Result<Vec<Row>> {
        // read static data.
        if table_name == PG_TYPE_TABLE_NAME {
            Ok(PG_TYPE_DATA_ROWS.clone())
        } else if table_name == PG_CAST_TABLE_NAME {
            Ok(PG_CAST_DATA_ROWS.clone())
        } else if table_name == PG_NAMESPACE_TABLE_NAME {
            self.read_namespace()
        } else if table_name == PG_MATVIEWS_INFO_TABLE_NAME {
            self.read_mviews_info().await
        } else {
            Err(ErrorCode::ItemNotFound(format!("Invalid system table: {}", table_name)).into())
        }
    }
}

impl SysCatalogReaderImpl {
    fn read_namespace(&self) -> Result<Vec<Row>> {
        let reader = self.catalog_reader.read_guard();
        let schemas = reader.get_all_schema_info(&self.auth_context.database)?;
        Ok(schemas
            .iter()
            .map(|schema| {
                Row::new(vec![
                    Some(ScalarImpl::Int32(schema.id as i32)),
                    Some(ScalarImpl::Utf8(schema.name.clone())),
                    Some(ScalarImpl::Utf8(schema.owner.clone())),
                ])
            })
            .collect_vec())
    }

    async fn read_mviews_info(&self) -> Result<Vec<Row>> {
        let mut table_ids = Vec::new();
        {
            let reader = self.catalog_reader.read_guard();
            let schemas = reader.get_all_schema_names(&self.auth_context.database)?;
            for schema in &schemas {
                reader
                    .get_schema_by_name(&self.auth_context.database, schema)?
                    .iter_mv()
                    .for_each(|t| {
                        table_ids.push(t.id.table_id);
                    });
            }
        }

        let table_fragments = self.meta_client.list_table_fragments(&table_ids).await?;
        let mut rows = Vec::new();
        let reader = self.catalog_reader.read_guard();
        let schemas = reader.get_all_schema_names(&self.auth_context.database)?;
        for schema in &schemas {
            reader
                .get_schema_by_name(&self.auth_context.database, schema)?
                .iter_mv()
                .for_each(|t| {
                    if let Some(fragments) = table_fragments.get(&t.id.table_id) {
                        rows.push(Row::new(vec![
                            Some(ScalarImpl::Int32(t.id.table_id as i32)),
                            Some(ScalarImpl::Utf8(t.name.clone())),
                            Some(ScalarImpl::Utf8(schema.clone())),
                            Some(ScalarImpl::Utf8(t.owner.clone())),
                            Some(ScalarImpl::Utf8(json!(fragments).to_string())),
                        ]));
                    }
                });
        }

        Ok(rows)
    }
}

// TODO: support struct column and type name when necessary.
type PgCatalogColumnsDef<'a> = (DataType, &'a str);

/// `def_sys_catalog` defines a table with given id, name and columns.
macro_rules! def_sys_catalog {
    ($id:expr, $name:ident, $columns:expr) => {
        SystemCatalog {
            id: TableId::new($id),
            name: $name.to_string(),
            columns: $columns
                .iter()
                .enumerate()
                .map(|(idx, col)| ColumnCatalog {
                    column_desc: ColumnDesc {
                        column_id: (idx as i32).into(),
                        data_type: col.0.clone(),
                        name: col.1.to_string(),
                        field_descs: vec![],
                        type_name: "".to_string(),
                    },
                    is_hidden: false,
                })
                .collect::<Vec<_>>(),
            pks: vec![0], // change this when multi pks needed in some system table.
            owner: DEFAULT_SUPPER_USER.to_string(),
        }
    };
}

lazy_static::lazy_static! {
    /// `PG_CATALOG_MAP` includes all system catalogs. If you added a new system catalog, be
    /// sure to add a corresponding entry here.
    pub(crate) static ref PG_CATALOG_MAP: HashMap<String, SystemCatalog> =
        [
            (PG_TYPE_TABLE_NAME.to_string(), def_sys_catalog!(1, PG_TYPE_TABLE_NAME, PG_TYPE_COLUMNS)),
            (PG_NAMESPACE_TABLE_NAME.to_string(), def_sys_catalog!(2, PG_NAMESPACE_TABLE_NAME, PG_NAMESPACE_COLUMNS)),
            (PG_CAST_TABLE_NAME.to_string(), def_sys_catalog!(3, PG_CAST_TABLE_NAME, PG_CAST_COLUMNS)),
            (PG_MATVIEWS_INFO_TABLE_NAME.to_string(), def_sys_catalog!(4, PG_MATVIEWS_INFO_TABLE_NAME, PG_MATVIEWS_INFO_COLUMNS)),
        ].into();
}

pub fn get_all_pg_catalogs() -> Vec<SystemCatalog> {
    PG_CATALOG_MAP.values().cloned().collect()
}
