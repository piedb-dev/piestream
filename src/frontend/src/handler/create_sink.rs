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

use pgwire::pg_response::{PgResponse, StatementType};
use risingwave_common::error::Result;
use risingwave_pb::catalog::Sink as ProstSink;
use risingwave_sqlparser::ast::CreateSinkStatement;

use super::util::handle_with_properties;
use crate::binder::Binder;
use crate::catalog::{DatabaseId, SchemaId};
use crate::session::OptimizerContext;

pub(crate) fn make_prost_sink(
    database_id: DatabaseId,
    schema_id: SchemaId,
    name: String,
    mv_name: String,
    properties: HashMap<String, String>,
    owner: String,
) -> Result<ProstSink> {
    Ok(ProstSink {
        id: 0,
        schema_id,
        database_id,
        name,
        mv_name,
        properties,
        owner,
    })
}

pub async fn handle_create_sink(
    context: OptimizerContext,
    stmt: CreateSinkStatement,
) -> Result<PgResponse> {
    let session = context.session_ctx.clone();

    let (schema_name, sink_name) = Binder::resolve_table_name(stmt.sink_name.clone())?;
    let (database_id, schema_id) = session
        .env()
        .catalog_reader()
        .read_guard()
        .check_relation_name_duplicated(session.database(), &schema_name, sink_name.as_str())?;

    let mv_name = stmt.materialized_view.to_string();
    let sink = make_prost_sink(
        database_id,
        schema_id,
        stmt.sink_name.to_string(),
        mv_name,
        handle_with_properties("create_sink", stmt.with_properties.0)?,
        session.user_name().to_string(),
    )?;

    let catalog_writer = session.env().catalog_writer();
    catalog_writer.create_sink(sink).await?;

    Ok(PgResponse::empty_result(StatementType::CREATE_SINK))
}

#[cfg(test)]
pub mod tests {
    use risingwave_common::catalog::{DEFAULT_DATABASE_NAME, DEFAULT_SCHEMA_NAME};

    use crate::test_utils::{create_proto_file, LocalFrontend, PROTO_FILE_DATA};

    #[tokio::test]
    async fn test_create_sink_handler() {
        let proto_file = create_proto_file(PROTO_FILE_DATA);
        let sql = format!(
            r#"CREATE SOURCE t1
    WITH ('kafka.topic' = 'abc', 'kafka.servers' = 'localhost:1001')
    ROW FORMAT PROTOBUF MESSAGE '.test.TestRecord' ROW SCHEMA LOCATION 'file://{}';"#,
            proto_file.path().to_str().unwrap()
        );
        let frontend = LocalFrontend::new(Default::default()).await;
        frontend.run_sql(sql).await.unwrap();

        let sql = "create materialized view mv1 as select t1.country from t1;";
        frontend.run_sql(sql).await.unwrap();

        let sql = r#"CREATE SINK snk1 FROM mv1
                    WITH ('sink' = 'mysql', 'mysql.endpoint' = '127.0.0.1:3306', 'mysql.table' =
                        '<table_name>', 'mysql.database' = '<database_name>', 'mysql.user' = '<user_name>',
                        'mysql.password' = '<password>');"#.to_string();
        frontend.run_sql(sql).await.unwrap();

        let session = frontend.session_ref();
        let catalog_reader = session.env().catalog_reader();

        // Check source exists.
        let source = catalog_reader
            .read_guard()
            .get_source_by_name(DEFAULT_DATABASE_NAME, DEFAULT_SCHEMA_NAME, "t1")
            .unwrap()
            .clone();
        assert_eq!(source.name, "t1");

        // Check table exists.
        let table = catalog_reader
            .read_guard()
            .get_table_by_name(DEFAULT_DATABASE_NAME, DEFAULT_SCHEMA_NAME, "mv1")
            .unwrap()
            .clone();
        assert_eq!(table.name(), "mv1");

        // Check sink exists.
        let sink = catalog_reader.read_guard().get_sink_id_by_name(
            DEFAULT_DATABASE_NAME,
            DEFAULT_SCHEMA_NAME,
            "snk1",
        );
        assert!(sink.is_ok());
    }
}
