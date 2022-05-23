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
use std::io::Write;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use parking_lot::RwLock;
use pgwire::pg_response::PgResponse;
use pgwire::pg_server::{BoxedError, Session, SessionManager};
use risingwave_common::catalog::{TableId, DEFAULT_DATABASE_NAME, DEFAULT_SCHEMA_NAME};
use risingwave_common::error::Result;
use risingwave_pb::catalog::table::OptionalAssociatedSourceId;
use risingwave_pb::catalog::{
    Database as ProstDatabase, Schema as ProstSchema, Source as ProstSource, Table as ProstTable,
};
use risingwave_pb::stream_plan::StreamFragmentGraph;
use risingwave_sqlparser::ast::Statement;
use risingwave_sqlparser::parser::Parser;
use tempfile::{Builder, NamedTempFile};

use crate::binder::Binder;
use crate::catalog::catalog_service::CatalogWriter;
use crate::catalog::root_catalog::Catalog;
use crate::catalog::{DatabaseId, SchemaId};
use crate::meta_client::FrontendMetaClient;
use crate::optimizer::PlanRef;
use crate::planner::Planner;
use crate::session::{FrontendEnv, OptimizerContext, SessionImpl};
use crate::FrontendOpts;

/// An embedded frontend without starting meta and without starting frontend as a tcp server.
pub struct LocalFrontend {
    pub opts: FrontendOpts,
    env: FrontendEnv,
}

impl SessionManager for LocalFrontend {
    type Session = SessionImpl;

    fn connect(&self, _database: &str) -> std::result::Result<Arc<Self::Session>, BoxedError> {
        Ok(self.session_ref())
    }
}

impl LocalFrontend {
    pub async fn new(opts: FrontendOpts) -> Self {
        let env = FrontendEnv::mock();
        Self { opts, env }
    }

    pub async fn run_sql(
        &self,
        sql: impl Into<String>,
    ) -> std::result::Result<PgResponse, Box<dyn std::error::Error + Send + Sync>> {
        let sql = sql.into();
        self.session_ref().run_statement(sql.as_str()).await
    }

    pub async fn query_formatted_result(&self, sql: impl Into<String>) -> Vec<String> {
        self.run_sql(sql)
            .await
            .unwrap()
            .iter()
            .map(|row| format!("{:?}", row))
            .collect()
    }

    /// Convert a sql (must be an `Query`) into an unoptimized batch plan.
    pub async fn to_batch_plan(&self, sql: impl Into<String>) -> Result<PlanRef> {
        let statements = Parser::parse_sql(&sql.into()).unwrap();
        let statement = statements.get(0).unwrap();
        if let Statement::Query(query) = statement {
            let session = self.session_ref();

            let bound = {
                let mut binder = Binder::new(
                    session.env().catalog_reader().read_guard(),
                    session.database().to_string(),
                );
                binder.bind(Statement::Query(query.clone()))?
            };
            Planner::new(OptimizerContext::new(session).into())
                .plan(bound)
                .unwrap()
                .gen_batch_query_plan()
        } else {
            unreachable!()
        }
    }

    pub fn session_ref(&self) -> Arc<SessionImpl> {
        Arc::new(SessionImpl::new(
            self.env.clone(),
            DEFAULT_DATABASE_NAME.to_string(),
        ))
    }
}

pub struct MockCatalogWriter {
    catalog: Arc<RwLock<Catalog>>,
    id: AtomicU32,
    table_id_to_schema_id: RwLock<HashMap<u32, SchemaId>>,
    schema_id_to_database_id: RwLock<HashMap<u32, DatabaseId>>,
}

#[async_trait::async_trait]
impl CatalogWriter for MockCatalogWriter {
    async fn create_database(&self, db_name: &str) -> Result<()> {
        self.catalog.write().create_database(ProstDatabase {
            name: db_name.to_string(),
            id: self.gen_id(),
        });
        Ok(())
    }

    async fn create_schema(&self, db_id: DatabaseId, schema_name: &str) -> Result<()> {
        let id = self.gen_id();
        self.catalog.write().create_schema(ProstSchema {
            id,
            name: schema_name.to_string(),
            database_id: db_id,
        });
        self.add_schema_id(id, db_id);
        Ok(())
    }

    async fn create_materialized_view(
        &self,
        mut table: ProstTable,
        _graph: StreamFragmentGraph,
    ) -> Result<()> {
        table.id = self.gen_id();
        self.catalog.write().create_table(&table);
        self.add_table_or_source_id(table.id, table.schema_id, table.database_id);
        Ok(())
    }

    async fn create_materialized_source(
        &self,
        source: ProstSource,
        mut table: ProstTable,
        graph: StreamFragmentGraph,
    ) -> Result<()> {
        let source_id = self.create_source_inner(source)?;
        table.optional_associated_source_id =
            Some(OptionalAssociatedSourceId::AssociatedSourceId(source_id));
        self.create_materialized_view(table, graph).await?;
        Ok(())
    }

    async fn create_source(&self, source: ProstSource) -> Result<()> {
        self.create_source_inner(source).map(|_| ())
    }

    async fn drop_materialized_source(&self, source_id: u32, table_id: TableId) -> Result<()> {
        let (database_id, schema_id) = self.drop_table_or_source_id(source_id);
        self.drop_table_or_source_id(table_id.table_id);
        self.catalog
            .write()
            .drop_table(database_id, schema_id, table_id);
        self.catalog
            .write()
            .drop_source(database_id, schema_id, source_id);
        Ok(())
    }

    async fn drop_source(&self, source_id: u32) -> Result<()> {
        let (database_id, schema_id) = self.drop_table_or_source_id(source_id);
        self.catalog
            .write()
            .drop_source(database_id, schema_id, source_id);
        Ok(())
    }

    async fn drop_database(&self, database_id: u32) -> Result<()> {
        self.catalog.write().drop_database(database_id);
        Ok(())
    }

    async fn drop_schema(&self, schema_id: u32) -> Result<()> {
        let database_id = self.drop_schema_id(schema_id);
        self.catalog.write().drop_schema(database_id, schema_id);
        Ok(())
    }

    async fn drop_materialized_view(&self, table_id: TableId) -> Result<()> {
        let (database_id, schema_id) = self.drop_table_or_source_id(table_id.table_id);
        self.drop_table_or_source_id(table_id.table_id);
        self.catalog
            .write()
            .drop_table(database_id, schema_id, table_id);
        Ok(())
    }
}

impl MockCatalogWriter {
    pub fn new(catalog: Arc<RwLock<Catalog>>) -> Self {
        catalog.write().create_database(ProstDatabase {
            name: DEFAULT_DATABASE_NAME.to_string(),
            id: 0,
        });
        catalog.write().create_schema(ProstSchema {
            id: 0,
            name: DEFAULT_SCHEMA_NAME.to_string(),
            database_id: 0,
        });
        let mut map: HashMap<u32, DatabaseId> = HashMap::new();
        map.insert(0_u32, 0_u32);
        Self {
            catalog,
            id: AtomicU32::new(0),
            table_id_to_schema_id: Default::default(),
            schema_id_to_database_id: RwLock::new(map),
        }
    }

    fn gen_id(&self) -> u32 {
        // Since the 0 value is `dev` schema and database, so jump out the 0 value.
        self.id.fetch_add(1, Ordering::SeqCst) + 1
    }

    fn add_table_or_source_id(&self, table_id: u32, schema_id: SchemaId, _database_id: DatabaseId) {
        self.table_id_to_schema_id
            .write()
            .insert(table_id, schema_id);
    }

    fn drop_table_or_source_id(&self, table_id: u32) -> (DatabaseId, SchemaId) {
        let schema_id = self
            .table_id_to_schema_id
            .write()
            .remove(&table_id)
            .unwrap();
        (self.get_database_id_by_schema(schema_id), schema_id)
    }

    fn add_schema_id(&self, schema_id: u32, database_id: DatabaseId) {
        self.schema_id_to_database_id
            .write()
            .insert(schema_id, database_id);
    }

    fn drop_schema_id(&self, schema_id: u32) -> DatabaseId {
        self.schema_id_to_database_id
            .write()
            .remove(&schema_id)
            .unwrap()
    }

    fn create_source_inner(&self, mut source: ProstSource) -> Result<u32> {
        source.id = self.gen_id();
        self.catalog.write().create_source(source.clone());
        self.add_table_or_source_id(source.id, source.schema_id, source.database_id);
        Ok(source.id)
    }

    fn get_database_id_by_schema(&self, schema_id: u32) -> DatabaseId {
        *self
            .schema_id_to_database_id
            .read()
            .get(&schema_id)
            .unwrap()
    }
}

pub struct MockFrontendMetaClient {}

#[async_trait::async_trait]
impl FrontendMetaClient for MockFrontendMetaClient {
    async fn pin_snapshot(&self, _epoch: u64) -> Result<u64> {
        Ok(0)
    }

    async fn flush(&self) -> Result<()> {
        Ok(())
    }

    async fn unpin_snapshot(&self, _epoch: u64) -> Result<()> {
        Ok(())
    }
}
pub static PROTO_FILE_DATA: &str = r#"
    syntax = "proto3";
    package test;
    message TestRecord {
      int32 id = 1;
      Country country = 3;
      int64 zipcode = 4;
      float rate = 5;
    }
    message Country {
      string address = 1;
      City city = 2;
      string zipcode = 3;
    }
    message City {
      string address = 1;
      string zipcode = 2;
    }"#;

/// Returns the file.
/// (`NamedTempFile` will automatically delete the file when it goes out of scope.)
pub fn create_proto_file(proto_data: &str) -> NamedTempFile {
    let temp_file = Builder::new()
        .prefix("temp")
        .suffix(".proto")
        .rand_bytes(5)
        .tempfile()
        .unwrap();

    let mut file = temp_file.as_file();
    file.write_all(proto_data.as_ref())
        .expect("writing binary to test file");
    temp_file
}
