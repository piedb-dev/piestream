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

use std::collections::BTreeMap;
use std::fmt::Debug;

use piestream_common::error::ErrorCode::ProtocolError;
use piestream_common::error::{Result, RwError};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

use crate::parser::common::json_parse_value;
use crate::{SourceParser, SourceStreamChunkRowWriter, WriteGuard};

const DEBEZIUM_READ_OP: &str = "r";
const DEBEZIUM_CREATE_OP: &str = "c";
const DEBEZIUM_UPDATE_OP: &str = "u";
const DEBEZIUM_DELETE_OP: &str = "d";

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebeziumEvent {
    pub payload: Payload,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub before: Option<BTreeMap<String, Value>>,
    pub after: Option<BTreeMap<String, Value>>,
    pub op: String,
    #[serde(rename = "ts_ms")]
    pub ts_ms: i64,
}

#[derive(Debug)]
pub struct DebeziumJsonParser;

impl SourceParser for DebeziumJsonParser {
    fn parse(&self, payload: &[u8], writer: SourceStreamChunkRowWriter<'_>) -> Result<WriteGuard> {
        let event: DebeziumEvent = serde_json::from_slice(payload)
            .map_err(|e| RwError::from(ProtocolError(e.to_string())))?;

        let mut payload = event.payload;

        match payload.op.as_str() {
            DEBEZIUM_UPDATE_OP => {
                let before = payload.before.as_mut().ok_or_else(|| {
                    RwError::from(ProtocolError(
                        "before is missing for updating event".to_string(),
                    ))
                })?;

                let after = payload.after.as_mut().ok_or_else(|| {
                    RwError::from(ProtocolError(
                        "after is missing for updating event".to_string(),
                    ))
                })?;

                writer.update(|column| {
                    let before = json_parse_value(&column.data_type, before.get(&column.name))?;
                    let after = json_parse_value(&column.data_type, after.get(&column.name))?;

                    Ok((before, after))
                })
            }
            DEBEZIUM_CREATE_OP | DEBEZIUM_READ_OP => {
                let after = payload.after.as_ref().ok_or_else(|| {
                    RwError::from(ProtocolError(
                        "after is missing for creating event".to_string(),
                    ))
                })?;

                writer.insert(|column| {
                    json_parse_value(&column.data_type, after.get(&column.name)).map_err(Into::into)
                })
            }
            DEBEZIUM_DELETE_OP => {
                let before = payload.before.as_ref().ok_or_else(|| {
                    RwError::from(ProtocolError(
                        "before is missing for delete event".to_string(),
                    ))
                })?;

                writer.delete(|column| {
                    json_parse_value(&column.data_type, before.get(&column.name))
                        .map_err(Into::into)
                })
            }
            _ => Err(RwError::from(ProtocolError(format!(
                "unknown debezium op: {}",
                payload.op
            )))),
        }
    }
}

#[cfg(test)]
mod test {
    use std::convert::TryInto;

    use piestream_common::array::{Op, Row};
    use piestream_common::catalog::ColumnId;
    use piestream_common::types::{DataType, ScalarImpl};

    use crate::parser::debezium::json::DebeziumJsonParser;
    use crate::{SourceColumnDesc, SourceParser, SourceStreamChunkBuilder};

    fn get_test_columns() -> Vec<SourceColumnDesc> {
        let descs = vec![
            SourceColumnDesc {
                name: "id".to_string(),
                data_type: DataType::Int32,
                column_id: ColumnId::from(0),
                skip_parse: false,
                fields: vec![],
            },
            SourceColumnDesc {
                name: "name".to_string(),
                data_type: DataType::Varchar,
                column_id: ColumnId::from(1),
                skip_parse: false,
                fields: vec![],
            },
            SourceColumnDesc {
                name: "description".to_string(),
                data_type: DataType::Varchar,
                column_id: ColumnId::from(2),
                skip_parse: false,
                fields: vec![],
            },
            SourceColumnDesc {
                name: "weight".to_string(),
                data_type: DataType::Float64,
                column_id: ColumnId::from(3),
                skip_parse: false,
                fields: vec![],
            },
        ];

        descs
    }

    fn parse_one(
        parser: impl SourceParser,
        columns: Vec<SourceColumnDesc>,
        payload: &[u8],
    ) -> Vec<(Op, Row)> {
        let mut builder = SourceStreamChunkBuilder::with_capacity(columns, 2);
        {
            let writer = builder.row_writer();
            parser.parse(payload, writer).unwrap();
        }
        let chunk = builder.finish();
        chunk
            .rows()
            .map(|(op, row_ref)| (op, row_ref.to_owned_row()))
            .collect::<Vec<_>>()
    }

    #[test]
    fn test_debezium_json_parser_read() {
        //     "before": null,
        //     "after": {
        //       "id": 101,
        //       "name": "scooter",
        //       "description": "Small 2-wheel scooter",
        //       "weight": 1.234
        //     },
        let data = br#"{"schema":{"type":"struct","fields":[{"type":"struct","fields":[{"type":"int32","optional":false,"field":"id"},{"type":"string","optional":false,"field":"name"},{"type":"string","optional":true,"field":"description"},{"type":"double","optional":true,"field":"weight"}],"optional":true,"name":"dbserver1.inventory.products.Value","field":"before"},{"type":"struct","fields":[{"type":"int32","optional":false,"field":"id"},{"type":"string","optional":false,"field":"name"},{"type":"string","optional":true,"field":"description"},{"type":"double","optional":true,"field":"weight"}],"optional":true,"name":"dbserver1.inventory.products.Value","field":"after"},{"type":"struct","fields":[{"type":"string","optional":false,"field":"version"},{"type":"string","optional":false,"field":"connector"},{"type":"string","optional":false,"field":"name"},{"type":"int64","optional":false,"field":"ts_ms"},{"type":"string","optional":true,"name":"io.debezium.data.Enum","version":1,"parameters":{"allowed":"true,last,false"},"default":"false","field":"snapshot"},{"type":"string","optional":false,"field":"db"},{"type":"string","optional":true,"field":"sequence"},{"type":"string","optional":true,"field":"table"},{"type":"int64","optional":false,"field":"server_id"},{"type":"string","optional":true,"field":"gtid"},{"type":"string","optional":false,"field":"file"},{"type":"int64","optional":false,"field":"pos"},{"type":"int32","optional":false,"field":"row"},{"type":"int64","optional":true,"field":"thread"},{"type":"string","optional":true,"field":"query"}],"optional":false,"name":"io.debezium.connector.mysql.Source","field":"source"},{"type":"string","optional":false,"field":"op"},{"type":"int64","optional":true,"field":"ts_ms"},{"type":"struct","fields":[{"type":"string","optional":false,"field":"id"},{"type":"int64","optional":false,"field":"total_order"},{"type":"int64","optional":false,"field":"data_collection_order"}],"optional":true,"field":"transaction"}],"optional":false,"name":"dbserver1.inventory.products.Envelope"},"payload":{"before":null,"after":{"id":101,"name":"scooter","description":"Small 2-wheel scooter","weight":1.234},"source":{"version":"1.7.1.Final","connector":"mysql","name":"dbserver1","ts_ms":1639547113601,"snapshot":"true","db":"inventory","sequence":null,"table":"products","server_id":0,"gtid":null,"file":"mysql-bin.000003","pos":156,"row":0,"thread":null,"query":null},"op":"r","ts_ms":1639547113602,"transaction":null}}"#;
        let parser = DebeziumJsonParser;
        let columns = get_test_columns();

        let [(_op, row)]: [_; 1] = parse_one(parser, columns, data).try_into().unwrap();

        assert!(row[0].eq(&Some(ScalarImpl::Int32(101))));
        assert!(row[1].eq(&Some(ScalarImpl::Utf8("scooter".to_string()))));
        assert!(row[2].eq(&Some(ScalarImpl::Utf8("Small 2-wheel scooter".to_string()))));
        assert!(row[3].eq(&Some(ScalarImpl::Float64(1.234.into()))));
    }

    #[test]
    fn test_debezium_json_parser_insert() {
        //     "before": null,
        //     "after": {
        //       "id": 102,
        //       "name": "car battery",
        //       "description": "12V car battery",
        //       "weight": 8.1
        //     },
        let data = br#"{"schema":{"type":"struct","fields":[{"type":"struct","fields":[{"type":"int32","optional":false,"field":"id"},{"type":"string","optional":false,"field":"name"},{"type":"string","optional":true,"field":"description"},{"type":"double","optional":true,"field":"weight"}],"optional":true,"name":"dbserver1.inventory.products.Value","field":"before"},{"type":"struct","fields":[{"type":"int32","optional":false,"field":"id"},{"type":"string","optional":false,"field":"name"},{"type":"string","optional":true,"field":"description"},{"type":"double","optional":true,"field":"weight"}],"optional":true,"name":"dbserver1.inventory.products.Value","field":"after"},{"type":"struct","fields":[{"type":"string","optional":false,"field":"version"},{"type":"string","optional":false,"field":"connector"},{"type":"string","optional":false,"field":"name"},{"type":"int64","optional":false,"field":"ts_ms"},{"type":"string","optional":true,"name":"io.debezium.data.Enum","version":1,"parameters":{"allowed":"true,last,false"},"default":"false","field":"snapshot"},{"type":"string","optional":false,"field":"db"},{"type":"string","optional":true,"field":"sequence"},{"type":"string","optional":true,"field":"table"},{"type":"int64","optional":false,"field":"server_id"},{"type":"string","optional":true,"field":"gtid"},{"type":"string","optional":false,"field":"file"},{"type":"int64","optional":false,"field":"pos"},{"type":"int32","optional":false,"field":"row"},{"type":"int64","optional":true,"field":"thread"},{"type":"string","optional":true,"field":"query"}],"optional":false,"name":"io.debezium.connector.mysql.Source","field":"source"},{"type":"string","optional":false,"field":"op"},{"type":"int64","optional":true,"field":"ts_ms"},{"type":"struct","fields":[{"type":"string","optional":false,"field":"id"},{"type":"int64","optional":false,"field":"total_order"},{"type":"int64","optional":false,"field":"data_collection_order"}],"optional":true,"field":"transaction"}],"optional":false,"name":"dbserver1.inventory.products.Envelope"},"payload":{"before":null,"after":{"id":102,"name":"car battery","description":"12V car battery","weight":8.1},"source":{"version":"1.7.1.Final","connector":"mysql","name":"dbserver1","ts_ms":1639551564000,"snapshot":"false","db":"inventory","sequence":null,"table":"products","server_id":223344,"gtid":null,"file":"mysql-bin.000003","pos":717,"row":0,"thread":null,"query":null},"op":"c","ts_ms":1639551564960,"transaction":null}}"#;
        let parser = DebeziumJsonParser;
        let columns = get_test_columns();
        let [(op, row)]: [_; 1] = parse_one(parser, columns, data).try_into().unwrap();
        assert_eq!(op, Op::Insert);

        assert!(row[0].eq(&Some(ScalarImpl::Int32(102))));
        assert!(row[1].eq(&Some(ScalarImpl::Utf8("car battery".to_string()))));
        assert!(row[2].eq(&Some(ScalarImpl::Utf8("12V car battery".to_string()))));
        assert!(row[3].eq(&Some(ScalarImpl::Float64(8.1.into()))));
    }

    #[test]
    fn test_debezium_json_parser_delete() {
        //     "before": {
        //       "id": 101,
        //       "name": "scooter",
        //       "description": "Small 2-wheel scooter",
        //       "weight": 1.234
        //     },
        //     "after": null,
        let data = br#"{"schema":{"type":"struct","fields":[{"type":"struct","fields":[{"type":"int32","optional":false,"field":"id"},{"type":"string","optional":false,"field":"name"},{"type":"string","optional":true,"field":"description"},{"type":"double","optional":true,"field":"weight"}],"optional":true,"name":"dbserver1.inventory.products.Value","field":"before"},{"type":"struct","fields":[{"type":"int32","optional":false,"field":"id"},{"type":"string","optional":false,"field":"name"},{"type":"string","optional":true,"field":"description"},{"type":"double","optional":true,"field":"weight"}],"optional":true,"name":"dbserver1.inventory.products.Value","field":"after"},{"type":"struct","fields":[{"type":"string","optional":false,"field":"version"},{"type":"string","optional":false,"field":"connector"},{"type":"string","optional":false,"field":"name"},{"type":"int64","optional":false,"field":"ts_ms"},{"type":"string","optional":true,"name":"io.debezium.data.Enum","version":1,"parameters":{"allowed":"true,last,false"},"default":"false","field":"snapshot"},{"type":"string","optional":false,"field":"db"},{"type":"string","optional":true,"field":"sequence"},{"type":"string","optional":true,"field":"table"},{"type":"int64","optional":false,"field":"server_id"},{"type":"string","optional":true,"field":"gtid"},{"type":"string","optional":false,"field":"file"},{"type":"int64","optional":false,"field":"pos"},{"type":"int32","optional":false,"field":"row"},{"type":"int64","optional":true,"field":"thread"},{"type":"string","optional":true,"field":"query"}],"optional":false,"name":"io.debezium.connector.mysql.Source","field":"source"},{"type":"string","optional":false,"field":"op"},{"type":"int64","optional":true,"field":"ts_ms"},{"type":"struct","fields":[{"type":"string","optional":false,"field":"id"},{"type":"int64","optional":false,"field":"total_order"},{"type":"int64","optional":false,"field":"data_collection_order"}],"optional":true,"field":"transaction"}],"optional":false,"name":"dbserver1.inventory.products.Envelope"},"payload":{"before":{"id":101,"name":"scooter","description":"Small 2-wheel scooter","weight":1.234},"after":null,"source":{"version":"1.7.1.Final","connector":"mysql","name":"dbserver1","ts_ms":1639551767000,"snapshot":"false","db":"inventory","sequence":null,"table":"products","server_id":223344,"gtid":null,"file":"mysql-bin.000003","pos":1045,"row":0,"thread":null,"query":null},"op":"d","ts_ms":1639551767775,"transaction":null}}"#;
        let parser = DebeziumJsonParser {};
        let columns = get_test_columns();
        let [(op, row)]: [_; 1] = parse_one(parser, columns, data).try_into().unwrap();

        assert_eq!(op, Op::Delete);

        assert!(row[0].eq(&Some(ScalarImpl::Int32(101))));
        assert!(row[1].eq(&Some(ScalarImpl::Utf8("scooter".to_string()))));
        assert!(row[2].eq(&Some(ScalarImpl::Utf8("Small 2-wheel scooter".to_string()))));
        assert!(row[3].eq(&Some(ScalarImpl::Float64(1.234.into()))));
    }

    #[test]
    fn test_debezium_json_parser_update() {
        //     "before": {
        //       "id": 102,
        //       "name": "car battery",
        //       "description": "12V car battery",
        //       "weight": 8.1
        //     },
        //     "after": {
        //       "id": 102,
        //       "name": "car battery",
        //       "description": "24V car battery",
        //       "weight": 9.1
        //     },
        let data = br#"{"schema":{"type":"struct","fields":[{"type":"struct","fields":[{"type":"int32","optional":false,"field":"id"},{"type":"string","optional":false,"field":"name"},{"type":"string","optional":true,"field":"description"},{"type":"double","optional":true,"field":"weight"}],"optional":true,"name":"dbserver1.inventory.products.Value","field":"before"},{"type":"struct","fields":[{"type":"int32","optional":false,"field":"id"},{"type":"string","optional":false,"field":"name"},{"type":"string","optional":true,"field":"description"},{"type":"double","optional":true,"field":"weight"}],"optional":true,"name":"dbserver1.inventory.products.Value","field":"after"},{"type":"struct","fields":[{"type":"string","optional":false,"field":"version"},{"type":"string","optional":false,"field":"connector"},{"type":"string","optional":false,"field":"name"},{"type":"int64","optional":false,"field":"ts_ms"},{"type":"string","optional":true,"name":"io.debezium.data.Enum","version":1,"parameters":{"allowed":"true,last,false"},"default":"false","field":"snapshot"},{"type":"string","optional":false,"field":"db"},{"type":"string","optional":true,"field":"sequence"},{"type":"string","optional":true,"field":"table"},{"type":"int64","optional":false,"field":"server_id"},{"type":"string","optional":true,"field":"gtid"},{"type":"string","optional":false,"field":"file"},{"type":"int64","optional":false,"field":"pos"},{"type":"int32","optional":false,"field":"row"},{"type":"int64","optional":true,"field":"thread"},{"type":"string","optional":true,"field":"query"}],"optional":false,"name":"io.debezium.connector.mysql.Source","field":"source"},{"type":"string","optional":false,"field":"op"},{"type":"int64","optional":true,"field":"ts_ms"},{"type":"struct","fields":[{"type":"string","optional":false,"field":"id"},{"type":"int64","optional":false,"field":"total_order"},{"type":"int64","optional":false,"field":"data_collection_order"}],"optional":true,"field":"transaction"}],"optional":false,"name":"dbserver1.inventory.products.Envelope"},"payload":{"before":{"id":102,"name":"car battery","description":"12V car battery","weight":8.1},"after":{"id":102,"name":"car battery","description":"24V car battery","weight":9.1},"source":{"version":"1.7.1.Final","connector":"mysql","name":"dbserver1","ts_ms":1639551901000,"snapshot":"false","db":"inventory","sequence":null,"table":"products","server_id":223344,"gtid":null,"file":"mysql-bin.000003","pos":1382,"row":0,"thread":null,"query":null},"op":"u","ts_ms":1639551901165,"transaction":null}}"#;
        let parser = DebeziumJsonParser {};
        let columns = get_test_columns();

        let [(op1, row1), (op2, row2)]: [_; 2] =
            parse_one(parser, columns, data).try_into().unwrap();

        assert_eq!(op1, Op::UpdateDelete);
        assert_eq!(op2, Op::UpdateInsert);

        assert!(row1[0].eq(&Some(ScalarImpl::Int32(102))));
        assert!(row1[1].eq(&Some(ScalarImpl::Utf8("car battery".to_string()))));
        assert!(row1[2].eq(&Some(ScalarImpl::Utf8("12V car battery".to_string()))));
        assert!(row1[3].eq(&Some(ScalarImpl::Float64(8.1.into()))));

        assert!(row2[0].eq(&Some(ScalarImpl::Int32(102))));
        assert!(row2[1].eq(&Some(ScalarImpl::Utf8("car battery".to_string()))));
        assert!(row2[2].eq(&Some(ScalarImpl::Utf8("24V car battery".to_string()))));
        assert!(row2[3].eq(&Some(ScalarImpl::Float64(9.1.into()))));
    }
}
