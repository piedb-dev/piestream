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
use pgwire::pg_response::{PgResponse, StatementType};
use piestream_common::error::Result;
use piestream_pb::catalog::source::Info;
use piestream_pb::catalog::{Source as ProstSource, StreamSourceInfo};
use piestream_pb::plan_common::{ColumnCatalog as ProstColumnCatalog, RowFormatType};
use piestream_source::ProtobufParser;
use piestream_sqlparser::ast::{CreateSourceStatement, ObjectName, ProtobufSchema, SourceSchema};

use super::create_table::{bind_sql_columns, gen_materialized_source_plan};
use super::util::handle_with_properties;
use crate::binder::Binder;
use crate::catalog::check_schema_writable;
use crate::catalog::column_catalog::ColumnCatalog;
use crate::session::{OptimizerContext, SessionImpl};
use crate::stream_fragmenter::StreamFragmenter;

pub(crate) fn make_prost_source(
    session: &SessionImpl,
    name: ObjectName,
    source_info: Info,
) -> Result<ProstSource> {
    //name=ObjectName([Ident { value: "t", quote_style: None }])
    let (schema_name, name) = Binder::resolve_table_name(name)?;
    println!("schema_name={:?} name={:?}", schema_name, name);
    check_schema_writable(&schema_name)?;

    let (database_id, schema_id) = session
        .env()
        .catalog_reader()
        .read_guard()
        .check_relation_name_duplicated(session.database(), &schema_name, &name)?;

    Ok(ProstSource {
        id: 0,
        schema_id,
        database_id,
        name,
        info: Some(source_info),
        owner: session.user_name().to_string(),
    })
}

/// Map a protobuf schema to a relational schema.
fn extract_protobuf_table_schema(schema: &ProtobufSchema) -> Result<Vec<ProstColumnCatalog>> {
    let parser = ProtobufParser::new(&schema.row_schema_location.0, &schema.message_name.0)?;
    let column_descs = parser.map_to_columns()?;

    Ok(column_descs
        .into_iter()
        .map(|col| ProstColumnCatalog {
            column_desc: Some(col),
            is_hidden: false,
        })
        .collect_vec())
}

pub async fn handle_create_source(
    context: OptimizerContext,
    is_materialized: bool,
    stmt: CreateSourceStatement,
) -> Result<PgResponse> {
    //println!("stmt={:?}", stmt.clone());
    /*
    test_create_source_handler测试用例打印：
        stmt=CreateSourceStatement { if_not_exists: false, columns: [], constraints: [], source_name: ObjectName([Ident { value: "t", quote_style: None }]), 
        with_properties: WithProperties([SqlOption { name: Ident { value: "kafka.topic", quote_style: Some('\'') }, value: SingleQuotedString("abc") }, 
        SqlOption { name: Ident { value: "kafka.servers", quote_style: Some('\'') }, value: SingleQuotedString("localhost:1001") }]), 
        source_schema: Protobuf(ProtobufSchema { message_name: AstString(".test.TestRecord"), 
        row_schema_location: AstString("file:///var/folders/j5/ff3xfxsd3gx1qkgkqnqht89r0000gn/T/tempSkDmc.proto") }) }
     */
    let with_properties = handle_with_properties("create_source", stmt.with_properties.0)?;
    /*
        println!("with_properties={:?}", with_properties);
        with_properties={"kafka.topic": "abc", "kafka.servers": "localhost:1001"}
    */
    
    let source = match &stmt.source_schema {
        SourceSchema::Protobuf(protobuf_schema) => {
            //先默认增加int64隐形字段，可以理解为行id
            let mut columns = vec![ColumnCatalog::row_id_column().to_protobuf()];
            //通过源配置解析字以及protobuf解析字段信息
            columns.extend(extract_protobuf_table_schema(protobuf_schema)?.into_iter());
            StreamSourceInfo {
                properties: with_properties.clone(),
                row_format: RowFormatType::Protobuf as i32,
                row_schema_location: protobuf_schema.row_schema_location.0.clone(),
                row_id_index: 0,
                columns,
                pk_column_ids: vec![0],
            }
        }
        SourceSchema::Json => StreamSourceInfo {
            properties: with_properties.clone(),
            row_format: RowFormatType::Json as i32,
            row_schema_location: "".to_string(),
            row_id_index: 0,
            columns: bind_sql_columns(stmt.columns)?,
            pk_column_ids: vec![0],
        },
    };

    let session = context.session_ctx.clone();
    let source = make_prost_source(&session, stmt.source_name, Info::StreamSource(source))?;
    let catalog_writer = session.env().catalog_writer();
    //是否需要创建视图，物化源需要创建视图，非物化源不需要
    if is_materialized {
        let (graph, table) = {
            let (plan, table) = gen_materialized_source_plan(
                context.into(),
                source.clone(),
                session.user_name().to_string(),
                with_properties.clone(),
            )?;
            let plan = plan.to_stream_prost();
            let graph = StreamFragmenter::build_graph(plan);

            (graph, table)
        };

        catalog_writer
            .create_materialized_source(source, table, graph)
            .await?;
    } else {
        catalog_writer.create_source(source).await?;
    }
    Ok(PgResponse::empty_result(StatementType::CREATE_SOURCE))
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;

    use piestream_common::catalog::{DEFAULT_DATABASE_NAME, DEFAULT_SCHEMA_NAME};
    use piestream_common::types::DataType;

    use crate::catalog::row_id_column_name;
    use crate::test_utils::{create_proto_file, LocalFrontend, PROTO_FILE_DATA};

    #[tokio::test]
    async fn test_create_source_handler() {
        //PROTO_FILE_DATA内容写入临时文件
        let proto_file = create_proto_file(PROTO_FILE_DATA);
        //ROW FORMAT PROTOBUF MESSAGE '.test.TestRecord' 指定row格式来自protobuf文件test.TestRecord结构体
        let sql = format!(
            r#"CREATE SOURCE t
    WITH ('kafka.topic' = 'abc', 'kafka.servers' = 'localhost:1001')
    ROW FORMAT PROTOBUF MESSAGE '.test.TestRecord' ROW SCHEMA LOCATION 'file://{}'"#,
            proto_file.path().to_str().unwrap()
        );
        //构建本地前端服务,LocalFrontend里都是封装的本地服务（test_utils.rs里）（catalog等等），实际线上运行都是rpc服务，这点大家关注下
        let frontend = LocalFrontend::new(Default::default()).await;
        //执行创建source语句，实际创建源是meta节点
        frontend.run_sql(sql).await.unwrap();

        let session = frontend.session_ref();
        let catalog_reader = session.env().catalog_reader();

        // Check source exists.
        let source = catalog_reader
            .read_guard()
            .get_source_by_name(DEFAULT_DATABASE_NAME, DEFAULT_SCHEMA_NAME, "t")
            .unwrap()
            .clone();
        assert_eq!(source.name, "t");

        // Only check stream source
        let catalogs = source.columns;
        let mut columns = vec![];

        // Get all column descs
        for catalog in catalogs {
            //println!("\ncolumn_desc.flatten.len={:?}\n", catalog.column_desc.flatten().len());
            columns.append(&mut catalog.column_desc.flatten());
        }
        let columns = columns
            .iter()
            .map(|col| (col.name.as_str(), col.data_type.clone()))
            .collect::<HashMap<&str, DataType>>();

        let city_type = DataType::Struct {
            fields: vec![DataType::Varchar, DataType::Varchar].into(),
        };
        //row_id_col_name是系统自动增加的隐藏字段
        let row_id_col_name = row_id_column_name();
        let expected_columns = maplit::hashmap! {
            row_id_col_name.as_str() => DataType::Int64,
            "id" => DataType::Int32,
            "country.zipcode" => DataType::Varchar,
            "zipcode" => DataType::Int64,
            "country.city.address" => DataType::Varchar,
            "country.address" => DataType::Varchar,
            "country.city" => city_type.clone(),
            "country.city.zipcode" => DataType::Varchar,
            "rate" => DataType::Float32,
            "country" => DataType::Struct {fields:vec![DataType::Varchar,city_type,DataType::Varchar].into()},
        };
        println!("columns={:?}", columns);
        assert_eq!(columns, expected_columns);
    }
}
