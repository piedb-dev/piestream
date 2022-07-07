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

use std::collections::BTreeMap;
use std::fmt::Debug;

use itertools::Itertools;
use piestream_common::array::Op;
use piestream_common::array::Op::{UpdateDelete, UpdateInsert};
use piestream_common::error::ErrorCode::ProtocolError;
use piestream_common::error::{Result, RwError};
use piestream_common::types::Datum;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

use crate::parser::common::json_parse_value;
use crate::{Event, SourceColumnDesc, SourceParser};

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
pub struct DebeziumJsonParser {}

impl DebeziumJsonParser {
    fn value_to_datums(columns: &[SourceColumnDesc], map: &BTreeMap<String, Value>) -> Vec<Datum> {
        columns
            .iter()
            .map(|column| {
                if column.skip_parse {
                    None
                } else {
                    json_parse_value(column, map.get(&column.name)).ok()
                }
            })
            .collect::<Vec<Datum>>()
    }
}

impl SourceParser for DebeziumJsonParser {
    fn parse(&self, payload: &[u8], columns: &[SourceColumnDesc]) -> Result<Event> {
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

                let mut filtered_before = BTreeMap::new();
                let mut filtered_after = BTreeMap::new();

                for col in columns {
                    if let Some(value) = before.remove(col.name.as_str()) {
                        filtered_before.insert(col.name.clone(), value);
                    }

                    if let Some(value) = after.remove(col.name.as_str()) {
                        filtered_after.insert(col.name.clone(), value);
                    }
                }

                if filtered_before
                    .iter()
                    .zip_eq(&filtered_after)
                    .all(|((_, v1), (_, v2))| v1.eq(v2))
                {
                    return Ok(Event {
                        ops: vec![],
                        rows: vec![],
                    });
                }

                Ok(Event {
                    ops: vec![UpdateDelete, UpdateInsert],
                    rows: vec![
                        Self::value_to_datums(columns, &filtered_before),
                        Self::value_to_datums(columns, &filtered_after),
                    ],
                })
            }
            DEBEZIUM_CREATE_OP | DEBEZIUM_READ_OP => Ok(Event {
                ops: vec![Op::Insert],
                rows: vec![Self::value_to_datums(
                    columns,
                    payload.after.as_ref().ok_or_else(|| {
                        RwError::from(ProtocolError(
                            "after is missing for creating event".to_string(),
                        ))
                    })?,
                )],
            }),
            DEBEZIUM_DELETE_OP => Ok(Event {
                ops: vec![Op::Delete],
                rows: vec![Self::value_to_datums(
                    columns,
                    payload.before.as_ref().ok_or_else(|| {
                        RwError::from(ProtocolError(
                            "before is missing for deleting event".to_string(),
                        ))
                    })?,
                )],
            }),
            _ => Err(RwError::from(ProtocolError(format!(
                "unknown debezium op: {}",
                payload.op
            )))),
        }
    }
}

#[cfg(test)]
mod test {
    use piestream_common::array::Op;
    use piestream_common::catalog::ColumnId;
    use piestream_common::types::{DataType, ScalarImpl};

    use crate::parser::debezium::json::DebeziumJsonParser;
    use crate::{SourceColumnDesc, SourceParser};

    fn get_test_columns() -> Vec<SourceColumnDesc> {
        let descs = vec![
            SourceColumnDesc {
                name: "id".to_string(),
                data_type: DataType::Int32,
                column_id: ColumnId::from(0),
                skip_parse: false,
            },
            SourceColumnDesc {
                name: "name".to_string(),
                data_type: DataType::Varchar,
                column_id: ColumnId::from(1),
                skip_parse: false,
            },
            SourceColumnDesc {
                name: "description".to_string(),
                data_type: DataType::Varchar,
                column_id: ColumnId::from(2),
                skip_parse: false,
            },
            SourceColumnDesc {
                name: "weight".to_string(),
                data_type: DataType::Float64,
                column_id: ColumnId::from(3),
                skip_parse: false,
            },
        ];

        descs
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
        let data = r#"{"schema":{"type":"struct","fields":[{"type":"struct","fields":[{"type":"int32","optional":false,"field":"id"},{"type":"string","optional":false,"field":"name"},{"type":"string","optional":true,"field":"description"},{"type":"double","optional":true,"field":"weight"}],"optional":true,"name":"dbserver1.inventory.products.Value","field":"before"},{"type":"struct","fields":[{"type":"int32","optional":false,"field":"id"},{"type":"string","optional":false,"field":"name"},{"type":"string","optional":true,"field":"description"},{"type":"double","optional":true,"field":"weight"}],"optional":true,"name":"dbserver1.inventory.products.Value","field":"after"},{"type":"struct","fields":[{"type":"string","optional":false,"field":"version"},{"type":"string","optional":false,"field":"connector"},{"type":"string","optional":false,"field":"name"},{"type":"int64","optional":false,"field":"ts_ms"},{"type":"string","optional":true,"name":"io.debezium.data.Enum","version":1,"parameters":{"allowed":"true,last,false"},"default":"false","field":"snapshot"},{"type":"string","optional":false,"field":"db"},{"type":"string","optional":true,"field":"sequence"},{"type":"string","optional":true,"field":"table"},{"type":"int64","optional":false,"field":"server_id"},{"type":"string","optional":true,"field":"gtid"},{"type":"string","optional":false,"field":"file"},{"type":"int64","optional":false,"field":"pos"},{"type":"int32","optional":false,"field":"row"},{"type":"int64","optional":true,"field":"thread"},{"type":"string","optional":true,"field":"query"}],"optional":false,"name":"io.debezium.connector.mysql.Source","field":"source"},{"type":"string","optional":false,"field":"op"},{"type":"int64","optional":true,"field":"ts_ms"},{"type":"struct","fields":[{"type":"string","optional":false,"field":"id"},{"type":"int64","optional":false,"field":"total_order"},{"type":"int64","optional":false,"field":"data_collection_order"}],"optional":true,"field":"transaction"}],"optional":false,"name":"dbserver1.inventory.products.Envelope"},"payload":{"before":null,"after":{"id":101,"name":"scooter","description":"Small 2-wheel scooter","weight":1.234},"source":{"version":"1.7.1.Final","connector":"mysql","name":"dbserver1","ts_ms":1639547113601,"snapshot":"true","db":"inventory","sequence":null,"table":"products","server_id":0,"gtid":null,"file":"mysql-bin.000003","pos":156,"row":0,"thread":null,"query":null},"op":"r","ts_ms":1639547113602,"transaction":null}}"#;
        let parser = DebeziumJsonParser {};
        let columns = get_test_columns();
        let result = parser.parse(data.as_ref(), columns.as_ref()).unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.ops.len(), 1);
        assert_eq!(result.ops[0], Op::Insert);

        let row = result.rows.first().unwrap();
        assert_eq!(row.capacity(), 4);

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
        let data = r#"{"schema":{"type":"struct","fields":[{"type":"struct","fields":[{"type":"int32","optional":false,"field":"id"},{"type":"string","optional":false,"field":"name"},{"type":"string","optional":true,"field":"description"},{"type":"double","optional":true,"field":"weight"}],"optional":true,"name":"dbserver1.inventory.products.Value","field":"before"},{"type":"struct","fields":[{"type":"int32","optional":false,"field":"id"},{"type":"string","optional":false,"field":"name"},{"type":"string","optional":true,"field":"description"},{"type":"double","optional":true,"field":"weight"}],"optional":true,"name":"dbserver1.inventory.products.Value","field":"after"},{"type":"struct","fields":[{"type":"string","optional":false,"field":"version"},{"type":"string","optional":false,"field":"connector"},{"type":"string","optional":false,"field":"name"},{"type":"int64","optional":false,"field":"ts_ms"},{"type":"string","optional":true,"name":"io.debezium.data.Enum","version":1,"parameters":{"allowed":"true,last,false"},"default":"false","field":"snapshot"},{"type":"string","optional":false,"field":"db"},{"type":"string","optional":true,"field":"sequence"},{"type":"string","optional":true,"field":"table"},{"type":"int64","optional":false,"field":"server_id"},{"type":"string","optional":true,"field":"gtid"},{"type":"string","optional":false,"field":"file"},{"type":"int64","optional":false,"field":"pos"},{"type":"int32","optional":false,"field":"row"},{"type":"int64","optional":true,"field":"thread"},{"type":"string","optional":true,"field":"query"}],"optional":false,"name":"io.debezium.connector.mysql.Source","field":"source"},{"type":"string","optional":false,"field":"op"},{"type":"int64","optional":true,"field":"ts_ms"},{"type":"struct","fields":[{"type":"string","optional":false,"field":"id"},{"type":"int64","optional":false,"field":"total_order"},{"type":"int64","optional":false,"field":"data_collection_order"}],"optional":true,"field":"transaction"}],"optional":false,"name":"dbserver1.inventory.products.Envelope"},"payload":{"before":null,"after":{"id":102,"name":"car battery","description":"12V car battery","weight":8.1},"source":{"version":"1.7.1.Final","connector":"mysql","name":"dbserver1","ts_ms":1639551564000,"snapshot":"false","db":"inventory","sequence":null,"table":"products","server_id":223344,"gtid":null,"file":"mysql-bin.000003","pos":717,"row":0,"thread":null,"query":null},"op":"c","ts_ms":1639551564960,"transaction":null}}"#;
        let parser = DebeziumJsonParser {};
        let columns = get_test_columns();
        let result = parser.parse(data.as_ref(), columns.as_ref()).unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.ops.len(), 1);
        assert_eq!(result.ops[0], Op::Insert);

        let row = result.rows.first().unwrap();
        assert_eq!(row.capacity(), 4);

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
        let data = r#"{"schema":{"type":"struct","fields":[{"type":"struct","fields":[{"type":"int32","optional":false,"field":"id"},{"type":"string","optional":false,"field":"name"},{"type":"string","optional":true,"field":"description"},{"type":"double","optional":true,"field":"weight"}],"optional":true,"name":"dbserver1.inventory.products.Value","field":"before"},{"type":"struct","fields":[{"type":"int32","optional":false,"field":"id"},{"type":"string","optional":false,"field":"name"},{"type":"string","optional":true,"field":"description"},{"type":"double","optional":true,"field":"weight"}],"optional":true,"name":"dbserver1.inventory.products.Value","field":"after"},{"type":"struct","fields":[{"type":"string","optional":false,"field":"version"},{"type":"string","optional":false,"field":"connector"},{"type":"string","optional":false,"field":"name"},{"type":"int64","optional":false,"field":"ts_ms"},{"type":"string","optional":true,"name":"io.debezium.data.Enum","version":1,"parameters":{"allowed":"true,last,false"},"default":"false","field":"snapshot"},{"type":"string","optional":false,"field":"db"},{"type":"string","optional":true,"field":"sequence"},{"type":"string","optional":true,"field":"table"},{"type":"int64","optional":false,"field":"server_id"},{"type":"string","optional":true,"field":"gtid"},{"type":"string","optional":false,"field":"file"},{"type":"int64","optional":false,"field":"pos"},{"type":"int32","optional":false,"field":"row"},{"type":"int64","optional":true,"field":"thread"},{"type":"string","optional":true,"field":"query"}],"optional":false,"name":"io.debezium.connector.mysql.Source","field":"source"},{"type":"string","optional":false,"field":"op"},{"type":"int64","optional":true,"field":"ts_ms"},{"type":"struct","fields":[{"type":"string","optional":false,"field":"id"},{"type":"int64","optional":false,"field":"total_order"},{"type":"int64","optional":false,"field":"data_collection_order"}],"optional":true,"field":"transaction"}],"optional":false,"name":"dbserver1.inventory.products.Envelope"},"payload":{"before":{"id":101,"name":"scooter","description":"Small 2-wheel scooter","weight":1.234},"after":null,"source":{"version":"1.7.1.Final","connector":"mysql","name":"dbserver1","ts_ms":1639551767000,"snapshot":"false","db":"inventory","sequence":null,"table":"products","server_id":223344,"gtid":null,"file":"mysql-bin.000003","pos":1045,"row":0,"thread":null,"query":null},"op":"d","ts_ms":1639551767775,"transaction":null}}"#;
        let parser = DebeziumJsonParser {};
        let columns = get_test_columns();
        let result = parser.parse(data.as_ref(), columns.as_ref()).unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.ops.len(), 1);
        assert_eq!(result.ops[0], Op::Delete);

        let row = result.rows.first().unwrap();
        assert_eq!(row.capacity(), 4);

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
        let data = r#"{"schema":{"type":"struct","fields":[{"type":"struct","fields":[{"type":"int32","optional":false,"field":"id"},{"type":"string","optional":false,"field":"name"},{"type":"string","optional":true,"field":"description"},{"type":"double","optional":true,"field":"weight"}],"optional":true,"name":"dbserver1.inventory.products.Value","field":"before"},{"type":"struct","fields":[{"type":"int32","optional":false,"field":"id"},{"type":"string","optional":false,"field":"name"},{"type":"string","optional":true,"field":"description"},{"type":"double","optional":true,"field":"weight"}],"optional":true,"name":"dbserver1.inventory.products.Value","field":"after"},{"type":"struct","fields":[{"type":"string","optional":false,"field":"version"},{"type":"string","optional":false,"field":"connector"},{"type":"string","optional":false,"field":"name"},{"type":"int64","optional":false,"field":"ts_ms"},{"type":"string","optional":true,"name":"io.debezium.data.Enum","version":1,"parameters":{"allowed":"true,last,false"},"default":"false","field":"snapshot"},{"type":"string","optional":false,"field":"db"},{"type":"string","optional":true,"field":"sequence"},{"type":"string","optional":true,"field":"table"},{"type":"int64","optional":false,"field":"server_id"},{"type":"string","optional":true,"field":"gtid"},{"type":"string","optional":false,"field":"file"},{"type":"int64","optional":false,"field":"pos"},{"type":"int32","optional":false,"field":"row"},{"type":"int64","optional":true,"field":"thread"},{"type":"string","optional":true,"field":"query"}],"optional":false,"name":"io.debezium.connector.mysql.Source","field":"source"},{"type":"string","optional":false,"field":"op"},{"type":"int64","optional":true,"field":"ts_ms"},{"type":"struct","fields":[{"type":"string","optional":false,"field":"id"},{"type":"int64","optional":false,"field":"total_order"},{"type":"int64","optional":false,"field":"data_collection_order"}],"optional":true,"field":"transaction"}],"optional":false,"name":"dbserver1.inventory.products.Envelope"},"payload":{"before":{"id":102,"name":"car battery","description":"12V car battery","weight":8.1},"after":{"id":102,"name":"car battery","description":"24V car battery","weight":9.1},"source":{"version":"1.7.1.Final","connector":"mysql","name":"dbserver1","ts_ms":1639551901000,"snapshot":"false","db":"inventory","sequence":null,"table":"products","server_id":223344,"gtid":null,"file":"mysql-bin.000003","pos":1382,"row":0,"thread":null,"query":null},"op":"u","ts_ms":1639551901165,"transaction":null}}"#;
        let parser = DebeziumJsonParser {};
        let columns = get_test_columns();
        let result = parser.parse(data.as_ref(), columns.as_ref()).unwrap();
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.ops.len(), 2);
        assert_eq!(result.ops[0], Op::UpdateDelete);
        assert_eq!(result.ops[1], Op::UpdateInsert);

        let row = result.rows.first().unwrap();
        assert_eq!(row.capacity(), 4);

        assert!(row[0].eq(&Some(ScalarImpl::Int32(102))));
        assert!(row[1].eq(&Some(ScalarImpl::Utf8("car battery".to_string()))));
        assert!(row[2].eq(&Some(ScalarImpl::Utf8("12V car battery".to_string()))));
        assert!(row[3].eq(&Some(ScalarImpl::Float64(8.1.into()))));

        let row = result.rows.last().unwrap();
        assert_eq!(row.capacity(), 4);

        assert!(row[0].eq(&Some(ScalarImpl::Int32(102))));
        assert!(row[1].eq(&Some(ScalarImpl::Utf8("car battery".to_string()))));
        assert!(row[2].eq(&Some(ScalarImpl::Utf8("24V car battery".to_string()))));
        assert!(row[3].eq(&Some(ScalarImpl::Float64(9.1.into()))));
    }

    #[test]
    fn test_debezium_json_parser_update_select() {
        //     "before": {
        //       "id": 102,
        //       "name": "car battery",
        //       "description": "12V car battery",
        //       "weight": 8.1
        //     },
        //     "after": {
        //       "id": 102,
        //       "name": "car battery",
        //       "description": "12V car battery",
        //       "weight": 9.1
        //     },
        let data = r#"{"schema":{"type":"struct","fields":[{"type":"struct","fields":[{"type":"int32","optional":false,"field":"id"},{"type":"string","optional":false,"field":"name"},{"type":"string","optional":true,"field":"description"},{"type":"double","optional":true,"field":"weight"}],"optional":true,"name":"dbserver1.inventory.products.Value","field":"before"},{"type":"struct","fields":[{"type":"int32","optional":false,"field":"id"},{"type":"string","optional":false,"field":"name"},{"type":"string","optional":true,"field":"description"},{"type":"double","optional":true,"field":"weight"}],"optional":true,"name":"dbserver1.inventory.products.Value","field":"after"},{"type":"struct","fields":[{"type":"string","optional":false,"field":"version"},{"type":"string","optional":false,"field":"connector"},{"type":"string","optional":false,"field":"name"},{"type":"int64","optional":false,"field":"ts_ms"},{"type":"string","optional":true,"name":"io.debezium.data.Enum","version":1,"parameters":{"allowed":"true,last,false"},"default":"false","field":"snapshot"},{"type":"string","optional":false,"field":"db"},{"type":"string","optional":true,"field":"sequence"},{"type":"string","optional":true,"field":"table"},{"type":"int64","optional":false,"field":"server_id"},{"type":"string","optional":true,"field":"gtid"},{"type":"string","optional":false,"field":"file"},{"type":"int64","optional":false,"field":"pos"},{"type":"int32","optional":false,"field":"row"},{"type":"int64","optional":true,"field":"thread"},{"type":"string","optional":true,"field":"query"}],"optional":false,"name":"io.debezium.connector.mysql.Source","field":"source"},{"type":"string","optional":false,"field":"op"},{"type":"int64","optional":true,"field":"ts_ms"},{"type":"struct","fields":[{"type":"string","optional":false,"field":"id"},{"type":"int64","optional":false,"field":"total_order"},{"type":"int64","optional":false,"field":"data_collection_order"}],"optional":true,"field":"transaction"}],"optional":false,"name":"dbserver1.inventory.products.Envelope"},"payload":{"before":{"id":102,"name":"car battery","description":"12V car battery","weight":8.1},"after":{"id":102,"name":"car battery","description":"12V car battery","weight":9.1},"source":{"version":"1.7.1.Final","connector":"mysql","name":"dbserver1","ts_ms":1639551901000,"snapshot":"false","db":"inventory","sequence":null,"table":"products","server_id":223344,"gtid":null,"file":"mysql-bin.000003","pos":1382,"row":0,"thread":null,"query":null},"op":"u","ts_ms":1639551901165,"transaction":null}}"#;
        let parser = DebeziumJsonParser {};
        let columns = get_test_columns();
        let columns = columns[..3].to_vec();
        let result = parser.parse(data.as_ref(), columns.as_ref()).unwrap();
        assert_eq!(result.rows.len(), 0);
        assert_eq!(result.ops.len(), 0);
    }
}
