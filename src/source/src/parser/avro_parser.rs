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
use std::ffi::OsStr;
use std::fmt::Debug;
use std::future::Future;
use std::path::Path;

use apache_avro::types::Value;
use apache_avro::{Reader, Schema};
use chrono::{Datelike, NaiveDate};
use itertools::Itertools;
use piestream_common::array::{ListValue, StructValue};
use piestream_common::error::ErrorCode::{InternalError, InvalidConfigValue, ProtocolError};
use piestream_common::error::{Result, RwError};
use piestream_common::types::{
    DataType, Datum, NaiveDateTimeWrapper, NaiveDateWrapper, OrderedF32, OrderedF64, ScalarImpl,
};
use piestream_connector::aws_utils::{default_conn_config, s3_client, AwsConfigV2};
use piestream_pb::plan_common::ColumnDesc;
use url::Url;

use crate::{SourceParser, SourceStreamChunkRowWriter, WriteGuard};

const AVRO_SCHEMA_LOCATION_S3_REGION: &str = "region";

pub fn unix_epoch_days() -> i32 {
    NaiveDate::from_ymd(1970, 1, 1).num_days_from_ce()
}

#[derive(Debug)]
pub struct AvroParser {
    schema: Schema,
}

impl AvroParser {
    pub async fn new(schema_location: &str, props: HashMap<String, String>) -> Result<Self> {
        let url = Url::parse(schema_location)
            .map_err(|e| InternalError(format!("failed to parse url ({}): {}", schema_location, e)))
            .unwrap();
        let url_schema = url.scheme();
        let schema_path = url.path();
        let arvo_schema =
            match url_schema {
                "file" => {
                    load_schema_async(
                        |path, _props| async move { read_schema_from_local(path) },
                        schema_path.to_string(),
                        None,
                    )
                    .await
                }
                "s3" => load_schema_async(
                    |path, props| async move { read_schema_from_s3(path, props.unwrap()).await },
                    // note that Url parse bucket as domain, so must pass the origin
                    // schema_location
                    schema_location.to_string(),
                    Some(props),
                )
                .await,
                _ => Err(RwError::from(ProtocolError(format!(
                    "path scheme {} is not supported",
                    url_schema
                )))),
            };
        if let Ok(schema) = arvo_schema {
            Ok(Self { schema })
        } else {
            Err(arvo_schema.err().unwrap())
        }
    }

    pub fn map_to_columns(&self) -> Result<Vec<ColumnDesc>> {
        // there must be a record at top level
        if let Schema::Record { fields, .. } = &self.schema {
            let mut index = 0;
            let fields = fields
                .iter()
                .map(|field| {
                    Self::avro_field_to_column_desc(&field.name, &field.schema, &mut index)
                })
                .collect::<Result<Vec<_>>>()?;
            tracing::info!("fields is {:?}", fields);
            Ok(fields)
        } else {
            Err(RwError::from(InternalError(
                "schema invalid, record required".into(),
            )))
        }
    }

    fn avro_field_to_column_desc(
        name: &str,
        schema: &Schema,
        index: &mut i32,
    ) -> Result<ColumnDesc> {
        let data_type = Self::avro_type_mapping(schema)?;
        match schema {
            Schema::Record {
                name: schema_name,
                fields,
                ..
            } => {
                let vec_column = fields
                    .iter()
                    .map(|f| Self::avro_field_to_column_desc(&f.name, &f.schema, index))
                    .collect::<Result<Vec<_>>>()?;
                *index += 1;
                Ok(ColumnDesc {
                    column_type: Some(data_type.to_protobuf()),
                    column_id: *index,
                    name: name.to_owned(),
                    field_descs: vec_column,
                    type_name: schema_name.to_string(),
                })
            }
            _ => {
                *index += 1;
                Ok(ColumnDesc {
                    column_type: Some(data_type.to_protobuf()),
                    column_id: *index,
                    name: name.to_owned(),
                    ..Default::default()
                })
            }
        }
    }

    fn avro_type_mapping(schema: &Schema) -> Result<DataType> {
        let data_type = match schema {
            Schema::String => DataType::Varchar,
            Schema::Int => DataType::Int32,
            Schema::Long => DataType::Int64,
            Schema::Boolean => DataType::Boolean,
            Schema::Float => DataType::Float32,
            Schema::Double => DataType::Float64,
            Schema::Date => DataType::Date,
            Schema::TimestampMillis => DataType::Timestamp,
            Schema::Enum { .. } => DataType::Varchar,
            Schema::Record { fields, .. } => {
                let struct_fields = fields
                    .iter()
                    .map(|f| Self::avro_type_mapping(&f.schema))
                    .collect::<Result<Vec<_>>>()?;
                let struct_names = fields.iter().map(|f| f.name.clone()).collect_vec();
                DataType::new_struct(struct_fields, struct_names)
            }
            Schema::Array(item_schema) => {
                let item_type = Self::avro_type_mapping(item_schema.as_ref())?;
                DataType::List {
                    datatype: Box::new(item_type),
                }
            }
            _ => {
                return Err(RwError::from(InternalError(format!(
                    "unsupported type in Avro: {:?}",
                    schema
                ))));
            }
        };

        Ok(data_type)
    }
}

/// Convert Avro value to datum.For now, support the following [Avro type](https://avro.apache.org/docs/current/spec.html).
///  - boolean
///  - int : i32
///  - long: i64
///  - float: f32
///  - double: f64
///  - string: String
///  - Date (the number of days from the unix epoch, 1970-1-1 UTC)
///  - Timestamp (the number of milliseconds from the unix epoch,  1970-1-1 00:00:00.000 UTC)
#[inline]
fn from_avro_value(value: Value) -> Result<Datum> {
    let v = match value {
        Value::Boolean(b) => ScalarImpl::Bool(b),
        Value::String(s) => ScalarImpl::Utf8(s),
        Value::Int(i) => ScalarImpl::Int32(i),
        Value::Long(i) => ScalarImpl::Int64(i),
        Value::Float(f) => ScalarImpl::Float32(OrderedF32::from(f)),
        Value::Double(f) => ScalarImpl::Float64(OrderedF64::from(f)),
        Value::Date(days) => ScalarImpl::NaiveDate(
            NaiveDateWrapper::with_days(days + unix_epoch_days()).map_err(|e| {
                let err_msg = format!("avro parse error.wrong date value {}, err {:?}", days, e);
                RwError::from(InternalError(err_msg))
            })?,
        ),
        Value::TimestampMillis(millis) => ScalarImpl::NaiveDateTime(
            NaiveDateTimeWrapper::with_secs_nsecs(
                millis / 1000,
                (millis % 1000) as u32 * 1_000_000,
            )
            .map_err(|e| {
                let err_msg = format!(
                    "avro parse error.wrong timestamp mills value {}, err {:?}",
                    millis, e
                );
                RwError::from(InternalError(err_msg))
            })?,
        ),
        Value::Enum(_, symbol) => ScalarImpl::Utf8(symbol),
        Value::Record(descs) => {
            let rw_values = descs
                .into_iter()
                .map(|(_, value)| from_avro_value(value))
                .collect::<Result<Vec<Datum>>>()?;
            ScalarImpl::Struct(StructValue::new(rw_values))
        }
        Value::Array(values) => {
            let rw_values = values
                .into_iter()
                .map(from_avro_value)
                .collect::<Result<Vec<Datum>>>()?;
            ScalarImpl::List(ListValue::new(rw_values))
        }
        _ => {
            let err_msg = format!("avro parse error.unsupported value {:?}", value);
            return Err(RwError::from(InternalError(err_msg)));
        }
    };

    Ok(Some(v))
}

impl SourceParser for AvroParser {
    fn parse(&self, payload: &[u8], writer: SourceStreamChunkRowWriter<'_>) -> Result<WriteGuard> {
        match Reader::with_schema(&self.schema, payload) {
            Ok(mut reader) => match reader.next() {
                Some(Ok(Value::Record(fields))) => writer.insert(|column| {
                    let tuple = fields.iter().find(|val| column.name.eq(&val.0)).unwrap();
                    from_avro_value(tuple.1.clone()).map_err(|e| {
                        tracing::error!(
                            "failed to process value ({}): {}",
                            String::from_utf8_lossy(payload),
                            e
                        );
                        e
                    })
                }),
                Some(Ok(_)) => Err(RwError::from(ProtocolError(
                    "avro parse unexpected value".to_string(),
                ))),
                Some(Err(e)) => Err(RwError::from(ProtocolError(e.to_string()))),
                None => Err(RwError::from(ProtocolError(
                    "avro parse unexpected eof".to_string(),
                ))),
            },
            Err(e) => Err(RwError::from(ProtocolError(e.to_string()))),
        }
    }
}

/// Read schema from s3 bucket.
/// S3 file location format: <s3://bucket_name/file_name>
pub async fn read_schema_from_s3(
    location: String,
    properties: HashMap<String, String>,
) -> Result<String> {
    let s3_url = Url::parse(location.as_str());
    if let Ok(url) = s3_url {
        let bucket = if let Some(bucket) = url.domain() {
            bucket
        } else {
            return Err(RwError::from(InternalError(format!(
                "Illegal Avro schema path {}",
                url
            ))));
        };
        if properties.get(AVRO_SCHEMA_LOCATION_S3_REGION).is_none() {
            return Err(RwError::from(InvalidConfigValue {
                config_entry: AVRO_SCHEMA_LOCATION_S3_REGION.to_string(),
                config_value: "NONE".to_string(),
            }));
        }
        let key = url.path().replace('/', "");
        let config = AwsConfigV2::from(properties.clone());
        let sdk_config = config.load_config(None).await;
        let s3_client = s3_client(&sdk_config, Some(default_conn_config()));
        let schema_content = s3_client
            .get_object()
            .bucket(bucket.to_string())
            .key(key)
            .send()
            .await;
        match schema_content {
            Ok(response) => {
                let body = response.body.collect().await;
                if let Ok(body_bytes) = body {
                    let schema_bytes = body_bytes.into_bytes().to_vec();
                    let schema_str = std::str::from_utf8(&schema_bytes);
                    if let Ok(str) = schema_str {
                        Ok(str.to_string())
                    } else {
                        let schema_decode_err =
                            anyhow::Error::from(schema_str.err().unwrap()).to_string();
                        Err(RwError::from(InternalError(format!(
                            "Avro schema not valid utf8 {}",
                            schema_decode_err
                        ))))
                    }
                } else {
                    let read_schema_err = body.err().unwrap().to_string();
                    Err(RwError::from(InternalError(format!(
                        "Read Avro schema file from s3 {}",
                        read_schema_err
                    ))))
                }
            }
            Err(err) => Err(RwError::from(InternalError(err.to_string()))),
        }
    } else {
        Err(RwError::from(InternalError(format!(
            "Illegal S3 Path {}",
            s3_url.err().unwrap()
        ))))
    }
}

/// Read avro schema file from local file.For on-premise or testing.
pub fn read_schema_from_local(path: String) -> Result<String> {
    let content_rs = std::fs::read_to_string(path.as_str());
    if let Ok(content) = content_rs {
        Ok(content)
    } else {
        Err(content_rs.err().unwrap().into())
    }
}

pub async fn load_schema_async<F, Fut>(
    f: F,
    schema_path: String,
    properties: Option<HashMap<String, String>>,
) -> Result<Schema>
where
    F: Fn(String, Option<HashMap<String, String>>) -> Fut,
    Fut: Future<Output = Result<String>>,
{
    let file_extension = Path::new(schema_path.as_str())
        .extension()
        .and_then(OsStr::to_str);

    if let Some(extension) = file_extension {
        if !extension.eq("avsc") {
            Err(RwError::from(InternalError(
                "Please specify the correct schema file XXX.avsc".to_string(),
            )))
        } else {
            let read_schema_rs = f(schema_path, properties).await;
            let schema_content = if let Ok(content) = read_schema_rs {
                content
            } else {
                return Err(RwError::from(InternalError(format!(
                    "Load Avro schema file error {}",
                    anyhow::Error::from(read_schema_rs.err().unwrap())
                ))));
            };
            let schema_rs = Schema::parse_str(schema_content.as_str());
            if let Ok(avro_schema) = schema_rs {
                Ok(avro_schema)
            } else {
                let schema_parse_err = schema_rs.err().unwrap();
                Err(RwError::from(InternalError(format!(
                    "Avro schema parse error {}",
                    anyhow::Error::from(schema_parse_err)
                ))))
            }
        }
    } else {
        Err(RwError::from(InternalError(format!(
            "Illegal Avro schema path. {}",
            schema_path
        ))))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::env;
    use std::ops::Sub;

    use apache_avro::types::{Record, Value};
    use apache_avro::{Codec, Schema, Writer};
    use chrono::NaiveDate;
    use piestream_common::array::Op;
    use piestream_common::catalog::ColumnId;
    use piestream_common::error;
    use piestream_common::types::{DataType, NaiveDateTimeWrapper, NaiveDateWrapper, ScalarImpl};

    use crate::parser::avro_parser::{
        load_schema_async, read_schema_from_local, read_schema_from_s3, unix_epoch_days, AvroParser,
    };
    use crate::{SourceColumnDesc, SourceParser, SourceStreamChunkBuilder};

    fn test_data_path(file_name: &str) -> String {
        let curr_dir = env::current_dir().unwrap().into_os_string();
        curr_dir.into_string().unwrap() + "/src/test_data/" + file_name
    }

    #[tokio::test]
    async fn test_read_schema_from_local() {
        let schema_path = test_data_path("complex-schema.avsc");
        let content_rs = read_schema_from_local(schema_path);
        assert!(content_rs.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_load_schema_from_s3() {
        let schema_location = "s3://dd-storage-s3/complex-schema.avsc".to_string();
        let mut s3_config_props = HashMap::new();
        s3_config_props.insert("region".to_string(), "cn-north-1".to_string());
        let schema_rs = load_schema_async(
            |path, props| read_schema_from_s3(path, props.unwrap()),
            schema_location,
            Some(s3_config_props.clone()),
        )
        .await;
        assert!(schema_rs.is_ok());
        println!("schema_rs = {:?}", schema_rs);
    }

    #[tokio::test]
    async fn test_load_schema_from_local() {
        let schema_location = test_data_path("complex-schema.avsc");
        let schema_rs = load_schema_async(
            |path, _props| async move { read_schema_from_local(path) },
            schema_location,
            None,
        )
        .await;
        assert!(schema_rs.is_ok());
        println!("schema rs = {:?}", schema_rs);
    }

    async fn new_avro_parser_from_local(file_name: &str) -> error::Result<AvroParser> {
        let schema_path = "file://".to_owned() + &test_data_path(file_name);
        AvroParser::new(schema_path.as_str(), HashMap::new()).await
    }

    #[tokio::test]
    async fn test_avro_parser() {
        let avro_parser_rs = new_avro_parser_from_local("simple-schema.avsc").await;
        assert!(avro_parser_rs.is_ok());
        let avro_parser = avro_parser_rs.unwrap();
        let schema = &avro_parser.schema;
        let record = build_avro_data(schema);
        assert_eq!(record.fields.len(), 8);
        let mut writer = Writer::with_codec(schema, Vec::new(), Codec::Snappy);
        let append_rs = writer.append(record.clone());
        assert!(append_rs.is_ok());
        let flush = writer.flush().unwrap();
        assert!(flush > 0);
        let input_data = writer.into_inner().unwrap();
        let columns = build_rw_columns();
        let mut builder = SourceStreamChunkBuilder::with_capacity(columns, 1);
        {
            let writer = builder.row_writer();
            avro_parser.parse(&input_data[..], writer).unwrap();
        }
        let chunk = builder.finish();
        let (op, row) = chunk.rows().next().unwrap();
        assert_eq!(op, Op::Insert);
        let row = row.to_owned_row();
        for (i, field) in record.fields.iter().enumerate() {
            let value = field.clone().1;
            match value {
                Value::String(str) => {
                    assert_eq!(row[i], Some(ScalarImpl::Utf8(str)));
                }
                Value::Boolean(bool_val) => {
                    assert_eq!(row[i], Some(ScalarImpl::Bool(bool_val)));
                }
                Value::Int(int_val) => {
                    assert_eq!(row[i], Some(ScalarImpl::Int32(int_val)));
                }
                Value::Long(i64_val) => {
                    assert_eq!(row[i], Some(ScalarImpl::Int64(i64_val)));
                }
                Value::Float(f32_val) => {
                    assert_eq!(row[i], Some(ScalarImpl::Float32(f32_val.into())));
                }
                Value::Double(f64_val) => {
                    assert_eq!(row[i], Some(ScalarImpl::Float64(f64_val.into())));
                }
                Value::Date(days) => {
                    let date = Some(ScalarImpl::NaiveDate(
                        NaiveDateWrapper::with_days(days + unix_epoch_days()).unwrap(),
                    ));
                    assert_eq!(row[i], date);
                }
                Value::TimestampMillis(millis) => {
                    let datetime = Some(ScalarImpl::NaiveDateTime(
                        NaiveDateTimeWrapper::with_secs_nsecs(
                            millis / 1000,
                            (millis % 1000) as u32 * 1_000_000,
                        )
                        .unwrap(),
                    ));
                    assert_eq!(row[i], datetime);
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }

    fn build_rw_columns() -> Vec<SourceColumnDesc> {
        vec![
            SourceColumnDesc {
                name: "id".to_string(),
                data_type: DataType::Int32,
                column_id: ColumnId::from(0),
                skip_parse: false,
                fields: vec![],
            },
            SourceColumnDesc {
                name: "sequence_id".to_string(),
                data_type: DataType::Int64,
                column_id: ColumnId::from(1),
                skip_parse: false,
                fields: vec![],
            },
            SourceColumnDesc {
                name: "name".to_string(),
                data_type: DataType::Varchar,
                column_id: ColumnId::from(2),
                skip_parse: false,
                fields: vec![],
            },
            SourceColumnDesc {
                name: "score".to_string(),
                data_type: DataType::Float32,
                column_id: ColumnId::from(3),
                skip_parse: false,
                fields: vec![],
            },
            SourceColumnDesc {
                name: "avg_score".to_string(),
                data_type: DataType::Float64,
                column_id: ColumnId::from(4),
                skip_parse: false,
                fields: vec![],
            },
            SourceColumnDesc {
                name: "is_lasted".to_string(),
                data_type: DataType::Boolean,
                column_id: ColumnId::from(5),
                skip_parse: false,
                fields: vec![],
            },
            SourceColumnDesc {
                name: "entrance_date".to_string(),
                data_type: DataType::Date,
                column_id: ColumnId::from(6),
                skip_parse: false,
                fields: vec![],
            },
            SourceColumnDesc {
                name: "birthday".to_string(),
                data_type: DataType::Timestamp,
                column_id: ColumnId::from(7),
                skip_parse: false,
                fields: vec![],
            },
        ]
    }

    fn build_avro_data(schema: &Schema) -> Record<'_> {
        let mut record = Record::new(schema).unwrap();
        if let Schema::Record {
            name: _, fields, ..
        } = schema.clone()
        {
            for field in &fields {
                match field.schema {
                    Schema::String => {
                        record.put(field.name.as_str(), "str_value".to_string());
                    }
                    Schema::Int => {
                        record.put(field.name.as_str(), 32_i32);
                    }
                    Schema::Long => {
                        record.put(field.name.as_str(), 64_i64);
                    }
                    Schema::Float => {
                        record.put(field.name.as_str(), 32_f32);
                    }
                    Schema::Double => {
                        record.put(field.name.as_str(), 64_f64);
                    }
                    Schema::Boolean => {
                        record.put(field.name.as_str(), true);
                    }
                    Schema::Date => {
                        let original_date = NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 0);
                        let naive_date = NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 0);
                        let num_days = naive_date.sub(original_date).num_days() as i32;
                        record.put(field.name.as_str(), Value::Date(num_days));
                    }
                    Schema::TimestampMillis => {
                        let datetime = NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 0);
                        let timestamp_mills = Value::TimestampMillis(datetime.timestamp() * 1_000);
                        record.put(field.name.as_str(), timestamp_mills);
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
        }
        record
    }

    #[tokio::test]
    async fn test_map_to_columns() {
        let avro_parser_rs = new_avro_parser_from_local("simple-schema.avsc")
            .await
            .unwrap();
        println!("{:?}", avro_parser_rs.map_to_columns().unwrap());
    }

    #[tokio::test]
    async fn test_new_avro_parser() {
        let avro_parser_rs = new_avro_parser_from_local("simple-schema.avsc").await;
        assert!(avro_parser_rs.is_ok());
        let avro_parser = avro_parser_rs.unwrap();
        println!("avro_parser = {:?}", avro_parser);
    }
}
