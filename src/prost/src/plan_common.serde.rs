use crate::plan_common::*;
impl serde::Serialize for CellBasedTableDesc {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.table_id != 0 {
            len += 1;
        }
        if !self.pk.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("plan_common.CellBasedTableDesc", len)?;
        if self.table_id != 0 {
            struct_ser.serialize_field("tableId", &self.table_id)?;
        }
        if !self.pk.is_empty() {
            struct_ser.serialize_field("pk", &self.pk)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CellBasedTableDesc {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tableId",
            "pk",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableId,
            Pk,
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
                            "tableId" => Ok(GeneratedField::TableId),
                            "pk" => Ok(GeneratedField::Pk),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CellBasedTableDesc;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct plan_common.CellBasedTableDesc")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CellBasedTableDesc, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_id = None;
                let mut pk = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TableId => {
                            if table_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableId"));
                            }
                            table_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::Pk => {
                            if pk.is_some() {
                                return Err(serde::de::Error::duplicate_field("pk"));
                            }
                            pk = Some(map.next_value()?);
                        }
                    }
                }
                Ok(CellBasedTableDesc {
                    table_id: table_id.unwrap_or_default(),
                    pk: pk.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("plan_common.CellBasedTableDesc", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ColumnCatalog {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.column_desc.is_some() {
            len += 1;
        }
        if self.is_hidden {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("plan_common.ColumnCatalog", len)?;
        if let Some(v) = self.column_desc.as_ref() {
            struct_ser.serialize_field("columnDesc", v)?;
        }
        if self.is_hidden {
            struct_ser.serialize_field("isHidden", &self.is_hidden)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ColumnCatalog {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "columnDesc",
            "isHidden",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ColumnDesc,
            IsHidden,
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
                            "columnDesc" => Ok(GeneratedField::ColumnDesc),
                            "isHidden" => Ok(GeneratedField::IsHidden),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ColumnCatalog;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct plan_common.ColumnCatalog")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ColumnCatalog, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut column_desc = None;
                let mut is_hidden = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ColumnDesc => {
                            if column_desc.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnDesc"));
                            }
                            column_desc = Some(map.next_value()?);
                        }
                        GeneratedField::IsHidden => {
                            if is_hidden.is_some() {
                                return Err(serde::de::Error::duplicate_field("isHidden"));
                            }
                            is_hidden = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ColumnCatalog {
                    column_desc,
                    is_hidden: is_hidden.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("plan_common.ColumnCatalog", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ColumnDesc {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.column_type.is_some() {
            len += 1;
        }
        if self.column_id != 0 {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.field_descs.is_empty() {
            len += 1;
        }
        if !self.type_name.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("plan_common.ColumnDesc", len)?;
        if let Some(v) = self.column_type.as_ref() {
            struct_ser.serialize_field("columnType", v)?;
        }
        if self.column_id != 0 {
            struct_ser.serialize_field("columnId", &self.column_id)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.field_descs.is_empty() {
            struct_ser.serialize_field("fieldDescs", &self.field_descs)?;
        }
        if !self.type_name.is_empty() {
            struct_ser.serialize_field("typeName", &self.type_name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ColumnDesc {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "columnType",
            "columnId",
            "name",
            "fieldDescs",
            "typeName",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ColumnType,
            ColumnId,
            Name,
            FieldDescs,
            TypeName,
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
                            "columnType" => Ok(GeneratedField::ColumnType),
                            "columnId" => Ok(GeneratedField::ColumnId),
                            "name" => Ok(GeneratedField::Name),
                            "fieldDescs" => Ok(GeneratedField::FieldDescs),
                            "typeName" => Ok(GeneratedField::TypeName),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ColumnDesc;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct plan_common.ColumnDesc")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ColumnDesc, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut column_type = None;
                let mut column_id = None;
                let mut name = None;
                let mut field_descs = None;
                let mut type_name = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ColumnType => {
                            if column_type.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnType"));
                            }
                            column_type = Some(map.next_value()?);
                        }
                        GeneratedField::ColumnId => {
                            if column_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnId"));
                            }
                            column_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::Name => {
                            if name.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        GeneratedField::FieldDescs => {
                            if field_descs.is_some() {
                                return Err(serde::de::Error::duplicate_field("fieldDescs"));
                            }
                            field_descs = Some(map.next_value()?);
                        }
                        GeneratedField::TypeName => {
                            if type_name.is_some() {
                                return Err(serde::de::Error::duplicate_field("typeName"));
                            }
                            type_name = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ColumnDesc {
                    column_type,
                    column_id: column_id.unwrap_or_default(),
                    name: name.unwrap_or_default(),
                    field_descs: field_descs.unwrap_or_default(),
                    type_name: type_name.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("plan_common.ColumnDesc", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ColumnOrder {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.order_type != 0 {
            len += 1;
        }
        if self.input_ref.is_some() {
            len += 1;
        }
        if self.return_type.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("plan_common.ColumnOrder", len)?;
        if self.order_type != 0 {
            let v = OrderType::from_i32(self.order_type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.order_type)))?;
            struct_ser.serialize_field("orderType", &v)?;
        }
        if let Some(v) = self.input_ref.as_ref() {
            struct_ser.serialize_field("inputRef", v)?;
        }
        if let Some(v) = self.return_type.as_ref() {
            struct_ser.serialize_field("returnType", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ColumnOrder {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "orderType",
            "inputRef",
            "returnType",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OrderType,
            InputRef,
            ReturnType,
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
                            "orderType" => Ok(GeneratedField::OrderType),
                            "inputRef" => Ok(GeneratedField::InputRef),
                            "returnType" => Ok(GeneratedField::ReturnType),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ColumnOrder;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct plan_common.ColumnOrder")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ColumnOrder, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut order_type = None;
                let mut input_ref = None;
                let mut return_type = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::OrderType => {
                            if order_type.is_some() {
                                return Err(serde::de::Error::duplicate_field("orderType"));
                            }
                            order_type = Some(map.next_value::<OrderType>()? as i32);
                        }
                        GeneratedField::InputRef => {
                            if input_ref.is_some() {
                                return Err(serde::de::Error::duplicate_field("inputRef"));
                            }
                            input_ref = Some(map.next_value()?);
                        }
                        GeneratedField::ReturnType => {
                            if return_type.is_some() {
                                return Err(serde::de::Error::duplicate_field("returnType"));
                            }
                            return_type = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ColumnOrder {
                    order_type: order_type.unwrap_or_default(),
                    input_ref,
                    return_type,
                })
            }
        }
        deserializer.deserialize_struct("plan_common.ColumnOrder", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DatabaseRefId {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.database_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("plan_common.DatabaseRefId", len)?;
        if self.database_id != 0 {
            struct_ser.serialize_field("databaseId", &self.database_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DatabaseRefId {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "databaseId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DatabaseId,
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
                            "databaseId" => Ok(GeneratedField::DatabaseId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DatabaseRefId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct plan_common.DatabaseRefId")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DatabaseRefId, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut database_id = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DatabaseId => {
                            if database_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("databaseId"));
                            }
                            database_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                    }
                }
                Ok(DatabaseRefId {
                    database_id: database_id.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("plan_common.DatabaseRefId", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Field {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.data_type.is_some() {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("plan_common.Field", len)?;
        if let Some(v) = self.data_type.as_ref() {
            struct_ser.serialize_field("dataType", v)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Field {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "dataType",
            "name",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DataType,
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
                            "dataType" => Ok(GeneratedField::DataType),
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
            type Value = Field;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct plan_common.Field")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Field, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut data_type = None;
                let mut name = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DataType => {
                            if data_type.is_some() {
                                return Err(serde::de::Error::duplicate_field("dataType"));
                            }
                            data_type = Some(map.next_value()?);
                        }
                        GeneratedField::Name => {
                            if name.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Field {
                    data_type,
                    name: name.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("plan_common.Field", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for JoinType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Inner => "INNER",
            Self::LeftOuter => "LEFT_OUTER",
            Self::RightOuter => "RIGHT_OUTER",
            Self::FullOuter => "FULL_OUTER",
            Self::LeftSemi => "LEFT_SEMI",
            Self::LeftAnti => "LEFT_ANTI",
            Self::RightSemi => "RIGHT_SEMI",
            Self::RightAnti => "RIGHT_ANTI",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for JoinType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "INNER",
            "LEFT_OUTER",
            "RIGHT_OUTER",
            "FULL_OUTER",
            "LEFT_SEMI",
            "LEFT_ANTI",
            "RIGHT_SEMI",
            "RIGHT_ANTI",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = JoinType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(JoinType::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(JoinType::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "INNER" => Ok(JoinType::Inner),
                    "LEFT_OUTER" => Ok(JoinType::LeftOuter),
                    "RIGHT_OUTER" => Ok(JoinType::RightOuter),
                    "FULL_OUTER" => Ok(JoinType::FullOuter),
                    "LEFT_SEMI" => Ok(JoinType::LeftSemi),
                    "LEFT_ANTI" => Ok(JoinType::LeftAnti),
                    "RIGHT_SEMI" => Ok(JoinType::RightSemi),
                    "RIGHT_ANTI" => Ok(JoinType::RightAnti),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for MaterializedViewInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.associated_table_ref_id.is_some() {
            len += 1;
        }
        if !self.column_orders.is_empty() {
            len += 1;
        }
        if !self.pk_indices.is_empty() {
            len += 1;
        }
        if !self.dependent_tables.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("plan_common.MaterializedViewInfo", len)?;
        if let Some(v) = self.associated_table_ref_id.as_ref() {
            struct_ser.serialize_field("associatedTableRefId", v)?;
        }
        if !self.column_orders.is_empty() {
            struct_ser.serialize_field("columnOrders", &self.column_orders)?;
        }
        if !self.pk_indices.is_empty() {
            struct_ser.serialize_field("pkIndices", &self.pk_indices)?;
        }
        if !self.dependent_tables.is_empty() {
            struct_ser.serialize_field("dependentTables", &self.dependent_tables)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MaterializedViewInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "associatedTableRefId",
            "columnOrders",
            "pkIndices",
            "dependentTables",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AssociatedTableRefId,
            ColumnOrders,
            PkIndices,
            DependentTables,
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
                            "associatedTableRefId" => Ok(GeneratedField::AssociatedTableRefId),
                            "columnOrders" => Ok(GeneratedField::ColumnOrders),
                            "pkIndices" => Ok(GeneratedField::PkIndices),
                            "dependentTables" => Ok(GeneratedField::DependentTables),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MaterializedViewInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct plan_common.MaterializedViewInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MaterializedViewInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut associated_table_ref_id = None;
                let mut column_orders = None;
                let mut pk_indices = None;
                let mut dependent_tables = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::AssociatedTableRefId => {
                            if associated_table_ref_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("associatedTableRefId"));
                            }
                            associated_table_ref_id = Some(map.next_value()?);
                        }
                        GeneratedField::ColumnOrders => {
                            if column_orders.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnOrders"));
                            }
                            column_orders = Some(map.next_value()?);
                        }
                        GeneratedField::PkIndices => {
                            if pk_indices.is_some() {
                                return Err(serde::de::Error::duplicate_field("pkIndices"));
                            }
                            pk_indices = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::DependentTables => {
                            if dependent_tables.is_some() {
                                return Err(serde::de::Error::duplicate_field("dependentTables"));
                            }
                            dependent_tables = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MaterializedViewInfo {
                    associated_table_ref_id,
                    column_orders: column_orders.unwrap_or_default(),
                    pk_indices: pk_indices.unwrap_or_default(),
                    dependent_tables: dependent_tables.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("plan_common.MaterializedViewInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OrderType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Invalid => "INVALID",
            Self::Ascending => "ASCENDING",
            Self::Descending => "DESCENDING",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for OrderType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "INVALID",
            "ASCENDING",
            "DESCENDING",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OrderType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(OrderType::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(OrderType::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "INVALID" => Ok(OrderType::Invalid),
                    "ASCENDING" => Ok(OrderType::Ascending),
                    "DESCENDING" => Ok(OrderType::Descending),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for OrderedColumnDesc {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.column_desc.is_some() {
            len += 1;
        }
        if self.order != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("plan_common.OrderedColumnDesc", len)?;
        if let Some(v) = self.column_desc.as_ref() {
            struct_ser.serialize_field("columnDesc", v)?;
        }
        if self.order != 0 {
            let v = OrderType::from_i32(self.order)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.order)))?;
            struct_ser.serialize_field("order", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OrderedColumnDesc {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "columnDesc",
            "order",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ColumnDesc,
            Order,
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
                            "columnDesc" => Ok(GeneratedField::ColumnDesc),
                            "order" => Ok(GeneratedField::Order),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OrderedColumnDesc;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct plan_common.OrderedColumnDesc")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<OrderedColumnDesc, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut column_desc = None;
                let mut order = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ColumnDesc => {
                            if column_desc.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnDesc"));
                            }
                            column_desc = Some(map.next_value()?);
                        }
                        GeneratedField::Order => {
                            if order.is_some() {
                                return Err(serde::de::Error::duplicate_field("order"));
                            }
                            order = Some(map.next_value::<OrderType>()? as i32);
                        }
                    }
                }
                Ok(OrderedColumnDesc {
                    column_desc,
                    order: order.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("plan_common.OrderedColumnDesc", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RowFormatType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Json => "JSON",
            Self::Protobuf => "PROTOBUF",
            Self::DebeziumJson => "DEBEZIUM_JSON",
            Self::Avro => "AVRO",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for RowFormatType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "JSON",
            "PROTOBUF",
            "DEBEZIUM_JSON",
            "AVRO",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RowFormatType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(RowFormatType::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(RowFormatType::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "JSON" => Ok(RowFormatType::Json),
                    "PROTOBUF" => Ok(RowFormatType::Protobuf),
                    "DEBEZIUM_JSON" => Ok(RowFormatType::DebeziumJson),
                    "AVRO" => Ok(RowFormatType::Avro),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for SchemaRefId {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.database_ref_id.is_some() {
            len += 1;
        }
        if self.schema_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("plan_common.SchemaRefId", len)?;
        if let Some(v) = self.database_ref_id.as_ref() {
            struct_ser.serialize_field("databaseRefId", v)?;
        }
        if self.schema_id != 0 {
            struct_ser.serialize_field("schemaId", &self.schema_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SchemaRefId {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "databaseRefId",
            "schemaId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DatabaseRefId,
            SchemaId,
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
                            "databaseRefId" => Ok(GeneratedField::DatabaseRefId),
                            "schemaId" => Ok(GeneratedField::SchemaId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SchemaRefId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct plan_common.SchemaRefId")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SchemaRefId, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut database_ref_id = None;
                let mut schema_id = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DatabaseRefId => {
                            if database_ref_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("databaseRefId"));
                            }
                            database_ref_id = Some(map.next_value()?);
                        }
                        GeneratedField::SchemaId => {
                            if schema_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("schemaId"));
                            }
                            schema_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                    }
                }
                Ok(SchemaRefId {
                    database_ref_id,
                    schema_id: schema_id.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("plan_common.SchemaRefId", FIELDS, GeneratedVisitor)
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
        if self.append_only {
            len += 1;
        }
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
        let mut struct_ser = serializer.serialize_struct("plan_common.StreamSourceInfo", len)?;
        if self.append_only {
            struct_ser.serialize_field("appendOnly", &self.append_only)?;
        }
        if !self.properties.is_empty() {
            struct_ser.serialize_field("properties", &self.properties)?;
        }
        if self.row_format != 0 {
            let v = RowFormatType::from_i32(self.row_format)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.row_format)))?;
            struct_ser.serialize_field("rowFormat", &v)?;
        }
        if !self.row_schema_location.is_empty() {
            struct_ser.serialize_field("rowSchemaLocation", &self.row_schema_location)?;
        }
        if self.row_id_index != 0 {
            struct_ser.serialize_field("rowIdIndex", &self.row_id_index)?;
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
            "appendOnly",
            "properties",
            "rowFormat",
            "rowSchemaLocation",
            "rowIdIndex",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AppendOnly,
            Properties,
            RowFormat,
            RowSchemaLocation,
            RowIdIndex,
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
                            "appendOnly" => Ok(GeneratedField::AppendOnly),
                            "properties" => Ok(GeneratedField::Properties),
                            "rowFormat" => Ok(GeneratedField::RowFormat),
                            "rowSchemaLocation" => Ok(GeneratedField::RowSchemaLocation),
                            "rowIdIndex" => Ok(GeneratedField::RowIdIndex),
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
                formatter.write_str("struct plan_common.StreamSourceInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StreamSourceInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut append_only = None;
                let mut properties = None;
                let mut row_format = None;
                let mut row_schema_location = None;
                let mut row_id_index = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::AppendOnly => {
                            if append_only.is_some() {
                                return Err(serde::de::Error::duplicate_field("appendOnly"));
                            }
                            append_only = Some(map.next_value()?);
                        }
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
                            row_format = Some(map.next_value::<RowFormatType>()? as i32);
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
                    }
                }
                Ok(StreamSourceInfo {
                    append_only: append_only.unwrap_or_default(),
                    properties: properties.unwrap_or_default(),
                    row_format: row_format.unwrap_or_default(),
                    row_schema_location: row_schema_location.unwrap_or_default(),
                    row_id_index: row_id_index.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("plan_common.StreamSourceInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TableRefId {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.schema_ref_id.is_some() {
            len += 1;
        }
        if self.table_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("plan_common.TableRefId", len)?;
        if let Some(v) = self.schema_ref_id.as_ref() {
            struct_ser.serialize_field("schemaRefId", v)?;
        }
        if self.table_id != 0 {
            struct_ser.serialize_field("tableId", &self.table_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TableRefId {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "schemaRefId",
            "tableId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SchemaRefId,
            TableId,
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
                            "schemaRefId" => Ok(GeneratedField::SchemaRefId),
                            "tableId" => Ok(GeneratedField::TableId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TableRefId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct plan_common.TableRefId")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TableRefId, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut schema_ref_id = None;
                let mut table_id = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SchemaRefId => {
                            if schema_ref_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("schemaRefId"));
                            }
                            schema_ref_id = Some(map.next_value()?);
                        }
                        GeneratedField::TableId => {
                            if table_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableId"));
                            }
                            table_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                    }
                }
                Ok(TableRefId {
                    schema_ref_id,
                    table_id: table_id.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("plan_common.TableRefId", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TableSourceInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("plan_common.TableSourceInfo", len)?;
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
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TableSourceInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct plan_common.TableSourceInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TableSourceInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(TableSourceInfo {
                })
            }
        }
        deserializer.deserialize_struct("plan_common.TableSourceInfo", FIELDS, GeneratedVisitor)
    }
}
