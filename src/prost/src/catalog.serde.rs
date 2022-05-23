use crate::catalog::*;
impl serde::Serialize for Database {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id != 0 {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("catalog.Database", len)?;
        if self.id != 0 {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Database {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "name",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            Name,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "id" => Ok(GeneratedField::Id),
                            "name" => Ok(GeneratedField::Name),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Database;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct catalog.Database")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Database, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id = None;
                let mut name = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::Name => {
                            if name.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Database {
                    id: id.unwrap_or_default(),
                    name: name.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("catalog.Database", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Schema {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id != 0 {
            len += 1;
        }
        if self.database_id != 0 {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("catalog.Schema", len)?;
        if self.id != 0 {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if self.database_id != 0 {
            struct_ser.serialize_field("databaseId", &self.database_id)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Schema {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "databaseId",
            "name",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            DatabaseId,
            Name,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "id" => Ok(GeneratedField::Id),
                            "databaseId" => Ok(GeneratedField::DatabaseId),
                            "name" => Ok(GeneratedField::Name),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Schema;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct catalog.Schema")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Schema, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id = None;
                let mut database_id = None;
                let mut name = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::DatabaseId => {
                            if database_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("databaseId"));
                            }
                            database_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::Name => {
                            if name.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Schema {
                    id: id.unwrap_or_default(),
                    database_id: database_id.unwrap_or_default(),
                    name: name.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("catalog.Schema", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Source {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id != 0 {
            len += 1;
        }
        if self.schema_id != 0 {
            len += 1;
        }
        if self.database_id != 0 {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        if self.info.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("catalog.Source", len)?;
        if self.id != 0 {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if self.schema_id != 0 {
            struct_ser.serialize_field("schemaId", &self.schema_id)?;
        }
        if self.database_id != 0 {
            struct_ser.serialize_field("databaseId", &self.database_id)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if let Some(v) = self.info.as_ref() {
            match v {
                source::Info::StreamSource(v) => {
                    struct_ser.serialize_field("streamSource", v)?;
                }
                source::Info::TableSource(v) => {
                    struct_ser.serialize_field("tableSource", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Source {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "schemaId",
            "databaseId",
            "name",
            "streamSource",
            "tableSource",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            SchemaId,
            DatabaseId,
            Name,
            StreamSource,
            TableSource,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "id" => Ok(GeneratedField::Id),
                            "schemaId" => Ok(GeneratedField::SchemaId),
                            "databaseId" => Ok(GeneratedField::DatabaseId),
                            "name" => Ok(GeneratedField::Name),
                            "streamSource" => Ok(GeneratedField::StreamSource),
                            "tableSource" => Ok(GeneratedField::TableSource),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Source;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct catalog.Source")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Source, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id = None;
                let mut schema_id = None;
                let mut database_id = None;
                let mut name = None;
                let mut info = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::SchemaId => {
                            if schema_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("schemaId"));
                            }
                            schema_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::DatabaseId => {
                            if database_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("databaseId"));
                            }
                            database_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::Name => {
                            if name.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        GeneratedField::StreamSource => {
                            if info.is_some() {
                                return Err(serde::de::Error::duplicate_field("streamSource"));
                            }
                            info = Some(source::Info::StreamSource(map.next_value()?));
                        }
                        GeneratedField::TableSource => {
                            if info.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableSource"));
                            }
                            info = Some(source::Info::TableSource(map.next_value()?));
                        }
                    }
                }
                Ok(Source {
                    id: id.unwrap_or_default(),
                    schema_id: schema_id.unwrap_or_default(),
                    database_id: database_id.unwrap_or_default(),
                    name: name.unwrap_or_default(),
                    info,
                })
            }
        }
        deserializer.deserialize_struct("catalog.Source", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StreamSourceInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.properties.is_empty() {
            len += 1;
        }
        if self.row_format != 0 {
            len += 1;
        }
        if !self.row_schema_location.is_empty() {
            len += 1;
        }
        if self.row_id_index != 0 {
            len += 1;
        }
        if !self.columns.is_empty() {
            len += 1;
        }
        if !self.pk_column_ids.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("catalog.StreamSourceInfo", len)?;
        if !self.properties.is_empty() {
            struct_ser.serialize_field("properties", &self.properties)?;
        }
        if self.row_format != 0 {
            let v = super::plan_common::RowFormatType::from_i32(self.row_format)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.row_format)))?;
            struct_ser.serialize_field("rowFormat", &v)?;
        }
        if !self.row_schema_location.is_empty() {
            struct_ser.serialize_field("rowSchemaLocation", &self.row_schema_location)?;
        }
        if self.row_id_index != 0 {
            struct_ser.serialize_field("rowIdIndex", &self.row_id_index)?;
        }
        if !self.columns.is_empty() {
            struct_ser.serialize_field("columns", &self.columns)?;
        }
        if !self.pk_column_ids.is_empty() {
            struct_ser.serialize_field("pkColumnIds", &self.pk_column_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StreamSourceInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "properties",
            "rowFormat",
            "rowSchemaLocation",
            "rowIdIndex",
            "columns",
            "pkColumnIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Properties,
            RowFormat,
            RowSchemaLocation,
            RowIdIndex,
            Columns,
            PkColumnIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "properties" => Ok(GeneratedField::Properties),
                            "rowFormat" => Ok(GeneratedField::RowFormat),
                            "rowSchemaLocation" => Ok(GeneratedField::RowSchemaLocation),
                            "rowIdIndex" => Ok(GeneratedField::RowIdIndex),
                            "columns" => Ok(GeneratedField::Columns),
                            "pkColumnIds" => Ok(GeneratedField::PkColumnIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StreamSourceInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct catalog.StreamSourceInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StreamSourceInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut properties = None;
                let mut row_format = None;
                let mut row_schema_location = None;
                let mut row_id_index = None;
                let mut columns = None;
                let mut pk_column_ids = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Properties => {
                            if properties.is_some() {
                                return Err(serde::de::Error::duplicate_field("properties"));
                            }
                            properties = Some(
                                map.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::RowFormat => {
                            if row_format.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowFormat"));
                            }
                            row_format = Some(map.next_value::<super::plan_common::RowFormatType>()? as i32);
                        }
                        GeneratedField::RowSchemaLocation => {
                            if row_schema_location.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowSchemaLocation"));
                            }
                            row_schema_location = Some(map.next_value()?);
                        }
                        GeneratedField::RowIdIndex => {
                            if row_id_index.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowIdIndex"));
                            }
                            row_id_index = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::Columns => {
                            if columns.is_some() {
                                return Err(serde::de::Error::duplicate_field("columns"));
                            }
                            columns = Some(map.next_value()?);
                        }
                        GeneratedField::PkColumnIds => {
                            if pk_column_ids.is_some() {
                                return Err(serde::de::Error::duplicate_field("pkColumnIds"));
                            }
                            pk_column_ids = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                    }
                }
                Ok(StreamSourceInfo {
                    properties: properties.unwrap_or_default(),
                    row_format: row_format.unwrap_or_default(),
                    row_schema_location: row_schema_location.unwrap_or_default(),
                    row_id_index: row_id_index.unwrap_or_default(),
                    columns: columns.unwrap_or_default(),
                    pk_column_ids: pk_column_ids.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("catalog.StreamSourceInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Table {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id != 0 {
            len += 1;
        }
        if self.schema_id != 0 {
            len += 1;
        }
        if self.database_id != 0 {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.columns.is_empty() {
            len += 1;
        }
        if !self.order_column_ids.is_empty() {
            len += 1;
        }
        if !self.orders.is_empty() {
            len += 1;
        }
        if !self.dependent_relations.is_empty() {
            len += 1;
        }
        if self.is_index {
            len += 1;
        }
        if self.index_on_id != 0 {
            len += 1;
        }
        if !self.distribution_keys.is_empty() {
            len += 1;
        }
        if !self.pk.is_empty() {
            len += 1;
        }
        if self.optional_associated_source_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("catalog.Table", len)?;
        if self.id != 0 {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if self.schema_id != 0 {
            struct_ser.serialize_field("schemaId", &self.schema_id)?;
        }
        if self.database_id != 0 {
            struct_ser.serialize_field("databaseId", &self.database_id)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.columns.is_empty() {
            struct_ser.serialize_field("columns", &self.columns)?;
        }
        if !self.order_column_ids.is_empty() {
            struct_ser.serialize_field("orderColumnIds", &self.order_column_ids)?;
        }
        if !self.orders.is_empty() {
            let v = self.orders.iter().cloned().map(|v| {
                super::plan_common::OrderType::from_i32(v)
                    .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", v)))
                }).collect::<Result<Vec<_>, _>>()?;
            struct_ser.serialize_field("orders", &v)?;
        }
        if !self.dependent_relations.is_empty() {
            struct_ser.serialize_field("dependentRelations", &self.dependent_relations)?;
        }
        if self.is_index {
            struct_ser.serialize_field("isIndex", &self.is_index)?;
        }
        if self.index_on_id != 0 {
            struct_ser.serialize_field("indexOnId", &self.index_on_id)?;
        }
        if !self.distribution_keys.is_empty() {
            struct_ser.serialize_field("distributionKeys", &self.distribution_keys)?;
        }
        if !self.pk.is_empty() {
            struct_ser.serialize_field("pk", &self.pk)?;
        }
        if let Some(v) = self.optional_associated_source_id.as_ref() {
            match v {
                table::OptionalAssociatedSourceId::AssociatedSourceId(v) => {
                    struct_ser.serialize_field("associatedSourceId", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Table {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "schemaId",
            "databaseId",
            "name",
            "columns",
            "orderColumnIds",
            "orders",
            "dependentRelations",
            "isIndex",
            "indexOnId",
            "distributionKeys",
            "pk",
            "associatedSourceId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            SchemaId,
            DatabaseId,
            Name,
            Columns,
            OrderColumnIds,
            Orders,
            DependentRelations,
            IsIndex,
            IndexOnId,
            DistributionKeys,
            Pk,
            AssociatedSourceId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "id" => Ok(GeneratedField::Id),
                            "schemaId" => Ok(GeneratedField::SchemaId),
                            "databaseId" => Ok(GeneratedField::DatabaseId),
                            "name" => Ok(GeneratedField::Name),
                            "columns" => Ok(GeneratedField::Columns),
                            "orderColumnIds" => Ok(GeneratedField::OrderColumnIds),
                            "orders" => Ok(GeneratedField::Orders),
                            "dependentRelations" => Ok(GeneratedField::DependentRelations),
                            "isIndex" => Ok(GeneratedField::IsIndex),
                            "indexOnId" => Ok(GeneratedField::IndexOnId),
                            "distributionKeys" => Ok(GeneratedField::DistributionKeys),
                            "pk" => Ok(GeneratedField::Pk),
                            "associatedSourceId" => Ok(GeneratedField::AssociatedSourceId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Table;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct catalog.Table")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Table, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id = None;
                let mut schema_id = None;
                let mut database_id = None;
                let mut name = None;
                let mut columns = None;
                let mut order_column_ids = None;
                let mut orders = None;
                let mut dependent_relations = None;
                let mut is_index = None;
                let mut index_on_id = None;
                let mut distribution_keys = None;
                let mut pk = None;
                let mut optional_associated_source_id = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::SchemaId => {
                            if schema_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("schemaId"));
                            }
                            schema_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::DatabaseId => {
                            if database_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("databaseId"));
                            }
                            database_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::Name => {
                            if name.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        GeneratedField::Columns => {
                            if columns.is_some() {
                                return Err(serde::de::Error::duplicate_field("columns"));
                            }
                            columns = Some(map.next_value()?);
                        }
                        GeneratedField::OrderColumnIds => {
                            if order_column_ids.is_some() {
                                return Err(serde::de::Error::duplicate_field("orderColumnIds"));
                            }
                            order_column_ids = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::Orders => {
                            if orders.is_some() {
                                return Err(serde::de::Error::duplicate_field("orders"));
                            }
                            orders = Some(map.next_value::<Vec<super::plan_common::OrderType>>()?.into_iter().map(|x| x as i32).collect());
                        }
                        GeneratedField::DependentRelations => {
                            if dependent_relations.is_some() {
                                return Err(serde::de::Error::duplicate_field("dependentRelations"));
                            }
                            dependent_relations = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::IsIndex => {
                            if is_index.is_some() {
                                return Err(serde::de::Error::duplicate_field("isIndex"));
                            }
                            is_index = Some(map.next_value()?);
                        }
                        GeneratedField::IndexOnId => {
                            if index_on_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("indexOnId"));
                            }
                            index_on_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::DistributionKeys => {
                            if distribution_keys.is_some() {
                                return Err(serde::de::Error::duplicate_field("distributionKeys"));
                            }
                            distribution_keys = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::Pk => {
                            if pk.is_some() {
                                return Err(serde::de::Error::duplicate_field("pk"));
                            }
                            pk = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::AssociatedSourceId => {
                            if optional_associated_source_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("associatedSourceId"));
                            }
                            optional_associated_source_id = Some(table::OptionalAssociatedSourceId::AssociatedSourceId(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            ));
                        }
                    }
                }
                Ok(Table {
                    id: id.unwrap_or_default(),
                    schema_id: schema_id.unwrap_or_default(),
                    database_id: database_id.unwrap_or_default(),
                    name: name.unwrap_or_default(),
                    columns: columns.unwrap_or_default(),
                    order_column_ids: order_column_ids.unwrap_or_default(),
                    orders: orders.unwrap_or_default(),
                    dependent_relations: dependent_relations.unwrap_or_default(),
                    is_index: is_index.unwrap_or_default(),
                    index_on_id: index_on_id.unwrap_or_default(),
                    distribution_keys: distribution_keys.unwrap_or_default(),
                    pk: pk.unwrap_or_default(),
                    optional_associated_source_id,
                })
            }
        }
        deserializer.deserialize_struct("catalog.Table", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TableSourceInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.columns.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("catalog.TableSourceInfo", len)?;
        if !self.columns.is_empty() {
            struct_ser.serialize_field("columns", &self.columns)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TableSourceInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "columns",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Columns,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "columns" => Ok(GeneratedField::Columns),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TableSourceInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct catalog.TableSourceInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TableSourceInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut columns = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Columns => {
                            if columns.is_some() {
                                return Err(serde::de::Error::duplicate_field("columns"));
                            }
                            columns = Some(map.next_value()?);
                        }
                    }
                }
                Ok(TableSourceInfo {
                    columns: columns.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("catalog.TableSourceInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for VirtualTable {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id != 0 {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.columns.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("catalog.VirtualTable", len)?;
        if self.id != 0 {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.columns.is_empty() {
            struct_ser.serialize_field("columns", &self.columns)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for VirtualTable {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "name",
            "columns",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            Name,
            Columns,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "id" => Ok(GeneratedField::Id),
                            "name" => Ok(GeneratedField::Name),
                            "columns" => Ok(GeneratedField::Columns),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = VirtualTable;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct catalog.VirtualTable")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<VirtualTable, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id = None;
                let mut name = None;
                let mut columns = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::Name => {
                            if name.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        GeneratedField::Columns => {
                            if columns.is_some() {
                                return Err(serde::de::Error::duplicate_field("columns"));
                            }
                            columns = Some(map.next_value()?);
                        }
                    }
                }
                Ok(VirtualTable {
                    id: id.unwrap_or_default(),
                    name: name.unwrap_or_default(),
                    columns: columns.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("catalog.VirtualTable", FIELDS, GeneratedVisitor)
    }
}
