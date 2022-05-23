use crate::batch_plan::*;
impl serde::Serialize for DeleteNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.table_source_ref_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.DeleteNode", len)?;
        if let Some(v) = self.table_source_ref_id.as_ref() {
            struct_ser.serialize_field("tableSourceRefId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tableSourceRefId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableSourceRefId,
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
                            "tableSourceRefId" => Ok(GeneratedField::TableSourceRefId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeleteNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.DeleteNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DeleteNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_source_ref_id = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TableSourceRefId => {
                            if table_source_ref_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableSourceRefId"));
                            }
                            table_source_ref_id = Some(map.next_value()?);
                        }
                    }
                }
                Ok(DeleteNode {
                    table_source_ref_id,
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.DeleteNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExchangeInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.mode != 0 {
            len += 1;
        }
        if self.distribution.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.ExchangeInfo", len)?;
        if self.mode != 0 {
            let v = exchange_info::DistributionMode::from_i32(self.mode)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.mode)))?;
            struct_ser.serialize_field("mode", &v)?;
        }
        if let Some(v) = self.distribution.as_ref() {
            match v {
                exchange_info::Distribution::BroadcastInfo(v) => {
                    struct_ser.serialize_field("broadcastInfo", v)?;
                }
                exchange_info::Distribution::HashInfo(v) => {
                    struct_ser.serialize_field("hashInfo", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExchangeInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "mode",
            "broadcastInfo",
            "hashInfo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Mode,
            BroadcastInfo,
            HashInfo,
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
                            "mode" => Ok(GeneratedField::Mode),
                            "broadcastInfo" => Ok(GeneratedField::BroadcastInfo),
                            "hashInfo" => Ok(GeneratedField::HashInfo),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExchangeInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.ExchangeInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ExchangeInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut mode = None;
                let mut distribution = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Mode => {
                            if mode.is_some() {
                                return Err(serde::de::Error::duplicate_field("mode"));
                            }
                            mode = Some(map.next_value::<exchange_info::DistributionMode>()? as i32);
                        }
                        GeneratedField::BroadcastInfo => {
                            if distribution.is_some() {
                                return Err(serde::de::Error::duplicate_field("broadcastInfo"));
                            }
                            distribution = Some(exchange_info::Distribution::BroadcastInfo(map.next_value()?));
                        }
                        GeneratedField::HashInfo => {
                            if distribution.is_some() {
                                return Err(serde::de::Error::duplicate_field("hashInfo"));
                            }
                            distribution = Some(exchange_info::Distribution::HashInfo(map.next_value()?));
                        }
                    }
                }
                Ok(ExchangeInfo {
                    mode: mode.unwrap_or_default(),
                    distribution,
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.ExchangeInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for exchange_info::BroadcastInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.count != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.ExchangeInfo.BroadcastInfo", len)?;
        if self.count != 0 {
            struct_ser.serialize_field("count", &self.count)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for exchange_info::BroadcastInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "count",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Count,
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
                            "count" => Ok(GeneratedField::Count),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = exchange_info::BroadcastInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.ExchangeInfo.BroadcastInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<exchange_info::BroadcastInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut count = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Count => {
                            if count.is_some() {
                                return Err(serde::de::Error::duplicate_field("count"));
                            }
                            count = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                    }
                }
                Ok(exchange_info::BroadcastInfo {
                    count: count.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.ExchangeInfo.BroadcastInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for exchange_info::DistributionMode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Single => "SINGLE",
            Self::Broadcast => "BROADCAST",
            Self::Hash => "HASH",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for exchange_info::DistributionMode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "SINGLE",
            "BROADCAST",
            "HASH",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = exchange_info::DistributionMode;

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
                    .and_then(exchange_info::DistributionMode::from_i32)
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
                    .and_then(exchange_info::DistributionMode::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "SINGLE" => Ok(exchange_info::DistributionMode::Single),
                    "BROADCAST" => Ok(exchange_info::DistributionMode::Broadcast),
                    "HASH" => Ok(exchange_info::DistributionMode::Hash),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for exchange_info::HashInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.output_count != 0 {
            len += 1;
        }
        if !self.keys.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.ExchangeInfo.HashInfo", len)?;
        if self.output_count != 0 {
            struct_ser.serialize_field("outputCount", &self.output_count)?;
        }
        if !self.keys.is_empty() {
            struct_ser.serialize_field("keys", &self.keys)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for exchange_info::HashInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "outputCount",
            "keys",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OutputCount,
            Keys,
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
                            "outputCount" => Ok(GeneratedField::OutputCount),
                            "keys" => Ok(GeneratedField::Keys),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = exchange_info::HashInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.ExchangeInfo.HashInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<exchange_info::HashInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut output_count = None;
                let mut keys = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::OutputCount => {
                            if output_count.is_some() {
                                return Err(serde::de::Error::duplicate_field("outputCount"));
                            }
                            output_count = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::Keys => {
                            if keys.is_some() {
                                return Err(serde::de::Error::duplicate_field("keys"));
                            }
                            keys = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                    }
                }
                Ok(exchange_info::HashInfo {
                    output_count: output_count.unwrap_or_default(),
                    keys: keys.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.ExchangeInfo.HashInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExchangeNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.sources.is_empty() {
            len += 1;
        }
        if !self.input_schema.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.ExchangeNode", len)?;
        if !self.sources.is_empty() {
            struct_ser.serialize_field("sources", &self.sources)?;
        }
        if !self.input_schema.is_empty() {
            struct_ser.serialize_field("inputSchema", &self.input_schema)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExchangeNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sources",
            "inputSchema",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Sources,
            InputSchema,
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
                            "sources" => Ok(GeneratedField::Sources),
                            "inputSchema" => Ok(GeneratedField::InputSchema),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExchangeNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.ExchangeNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ExchangeNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sources = None;
                let mut input_schema = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Sources => {
                            if sources.is_some() {
                                return Err(serde::de::Error::duplicate_field("sources"));
                            }
                            sources = Some(map.next_value()?);
                        }
                        GeneratedField::InputSchema => {
                            if input_schema.is_some() {
                                return Err(serde::de::Error::duplicate_field("inputSchema"));
                            }
                            input_schema = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ExchangeNode {
                    sources: sources.unwrap_or_default(),
                    input_schema: input_schema.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.ExchangeNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExchangeSource {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.task_output_id.is_some() {
            len += 1;
        }
        if self.host.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.ExchangeSource", len)?;
        if let Some(v) = self.task_output_id.as_ref() {
            struct_ser.serialize_field("taskOutputId", v)?;
        }
        if let Some(v) = self.host.as_ref() {
            struct_ser.serialize_field("host", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExchangeSource {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "taskOutputId",
            "host",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TaskOutputId,
            Host,
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
                            "taskOutputId" => Ok(GeneratedField::TaskOutputId),
                            "host" => Ok(GeneratedField::Host),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExchangeSource;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.ExchangeSource")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ExchangeSource, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut task_output_id = None;
                let mut host = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TaskOutputId => {
                            if task_output_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("taskOutputId"));
                            }
                            task_output_id = Some(map.next_value()?);
                        }
                        GeneratedField::Host => {
                            if host.is_some() {
                                return Err(serde::de::Error::duplicate_field("host"));
                            }
                            host = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ExchangeSource {
                    task_output_id,
                    host,
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.ExchangeSource", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FilterNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.search_condition.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.FilterNode", len)?;
        if let Some(v) = self.search_condition.as_ref() {
            struct_ser.serialize_field("searchCondition", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FilterNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "searchCondition",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SearchCondition,
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
                            "searchCondition" => Ok(GeneratedField::SearchCondition),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FilterNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.FilterNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FilterNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut search_condition = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SearchCondition => {
                            if search_condition.is_some() {
                                return Err(serde::de::Error::duplicate_field("searchCondition"));
                            }
                            search_condition = Some(map.next_value()?);
                        }
                    }
                }
                Ok(FilterNode {
                    search_condition,
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.FilterNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FilterScanNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.table_ref_id.is_some() {
            len += 1;
        }
        if !self.column_ids.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.FilterScanNode", len)?;
        if let Some(v) = self.table_ref_id.as_ref() {
            struct_ser.serialize_field("tableRefId", v)?;
        }
        if !self.column_ids.is_empty() {
            struct_ser.serialize_field("columnIds", &self.column_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FilterScanNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tableRefId",
            "columnIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableRefId,
            ColumnIds,
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
                            "tableRefId" => Ok(GeneratedField::TableRefId),
                            "columnIds" => Ok(GeneratedField::ColumnIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FilterScanNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.FilterScanNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FilterScanNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_ref_id = None;
                let mut column_ids = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TableRefId => {
                            if table_ref_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableRefId"));
                            }
                            table_ref_id = Some(map.next_value()?);
                        }
                        GeneratedField::ColumnIds => {
                            if column_ids.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnIds"));
                            }
                            column_ids = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                    }
                }
                Ok(FilterScanNode {
                    table_ref_id,
                    column_ids: column_ids.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.FilterScanNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GenerateSeriesNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.start.is_some() {
            len += 1;
        }
        if self.stop.is_some() {
            len += 1;
        }
        if self.step.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.GenerateSeriesNode", len)?;
        if let Some(v) = self.start.as_ref() {
            struct_ser.serialize_field("start", v)?;
        }
        if let Some(v) = self.stop.as_ref() {
            struct_ser.serialize_field("stop", v)?;
        }
        if let Some(v) = self.step.as_ref() {
            struct_ser.serialize_field("step", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenerateSeriesNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "start",
            "stop",
            "step",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Start,
            Stop,
            Step,
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
                            "start" => Ok(GeneratedField::Start),
                            "stop" => Ok(GeneratedField::Stop),
                            "step" => Ok(GeneratedField::Step),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenerateSeriesNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.GenerateSeriesNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenerateSeriesNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut start = None;
                let mut stop = None;
                let mut step = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Start => {
                            if start.is_some() {
                                return Err(serde::de::Error::duplicate_field("start"));
                            }
                            start = Some(map.next_value()?);
                        }
                        GeneratedField::Stop => {
                            if stop.is_some() {
                                return Err(serde::de::Error::duplicate_field("stop"));
                            }
                            stop = Some(map.next_value()?);
                        }
                        GeneratedField::Step => {
                            if step.is_some() {
                                return Err(serde::de::Error::duplicate_field("step"));
                            }
                            step = Some(map.next_value()?);
                        }
                    }
                }
                Ok(GenerateSeriesNode {
                    start,
                    stop,
                    step,
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.GenerateSeriesNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for HashAggNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.group_keys.is_empty() {
            len += 1;
        }
        if !self.agg_calls.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.HashAggNode", len)?;
        if !self.group_keys.is_empty() {
            struct_ser.serialize_field("groupKeys", &self.group_keys)?;
        }
        if !self.agg_calls.is_empty() {
            struct_ser.serialize_field("aggCalls", &self.agg_calls)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for HashAggNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "groupKeys",
            "aggCalls",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GroupKeys,
            AggCalls,
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
                            "groupKeys" => Ok(GeneratedField::GroupKeys),
                            "aggCalls" => Ok(GeneratedField::AggCalls),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = HashAggNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.HashAggNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<HashAggNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut group_keys = None;
                let mut agg_calls = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::GroupKeys => {
                            if group_keys.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupKeys"));
                            }
                            group_keys = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::AggCalls => {
                            if agg_calls.is_some() {
                                return Err(serde::de::Error::duplicate_field("aggCalls"));
                            }
                            agg_calls = Some(map.next_value()?);
                        }
                    }
                }
                Ok(HashAggNode {
                    group_keys: group_keys.unwrap_or_default(),
                    agg_calls: agg_calls.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.HashAggNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for HashJoinNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.join_type != 0 {
            len += 1;
        }
        if !self.left_key.is_empty() {
            len += 1;
        }
        if !self.right_key.is_empty() {
            len += 1;
        }
        if self.condition.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.HashJoinNode", len)?;
        if self.join_type != 0 {
            let v = super::plan_common::JoinType::from_i32(self.join_type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.join_type)))?;
            struct_ser.serialize_field("joinType", &v)?;
        }
        if !self.left_key.is_empty() {
            struct_ser.serialize_field("leftKey", &self.left_key)?;
        }
        if !self.right_key.is_empty() {
            struct_ser.serialize_field("rightKey", &self.right_key)?;
        }
        if let Some(v) = self.condition.as_ref() {
            struct_ser.serialize_field("condition", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for HashJoinNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "joinType",
            "leftKey",
            "rightKey",
            "condition",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            JoinType,
            LeftKey,
            RightKey,
            Condition,
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
                            "joinType" => Ok(GeneratedField::JoinType),
                            "leftKey" => Ok(GeneratedField::LeftKey),
                            "rightKey" => Ok(GeneratedField::RightKey),
                            "condition" => Ok(GeneratedField::Condition),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = HashJoinNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.HashJoinNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<HashJoinNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut join_type = None;
                let mut left_key = None;
                let mut right_key = None;
                let mut condition = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::JoinType => {
                            if join_type.is_some() {
                                return Err(serde::de::Error::duplicate_field("joinType"));
                            }
                            join_type = Some(map.next_value::<super::plan_common::JoinType>()? as i32);
                        }
                        GeneratedField::LeftKey => {
                            if left_key.is_some() {
                                return Err(serde::de::Error::duplicate_field("leftKey"));
                            }
                            left_key = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::RightKey => {
                            if right_key.is_some() {
                                return Err(serde::de::Error::duplicate_field("rightKey"));
                            }
                            right_key = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::Condition => {
                            if condition.is_some() {
                                return Err(serde::de::Error::duplicate_field("condition"));
                            }
                            condition = Some(map.next_value()?);
                        }
                    }
                }
                Ok(HashJoinNode {
                    join_type: join_type.unwrap_or_default(),
                    left_key: left_key.unwrap_or_default(),
                    right_key: right_key.unwrap_or_default(),
                    condition,
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.HashJoinNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for HopWindowNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.time_col.is_some() {
            len += 1;
        }
        if self.window_slide.is_some() {
            len += 1;
        }
        if self.window_size.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.HopWindowNode", len)?;
        if let Some(v) = self.time_col.as_ref() {
            struct_ser.serialize_field("timeCol", v)?;
        }
        if let Some(v) = self.window_slide.as_ref() {
            struct_ser.serialize_field("windowSlide", v)?;
        }
        if let Some(v) = self.window_size.as_ref() {
            struct_ser.serialize_field("windowSize", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for HopWindowNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "timeCol",
            "windowSlide",
            "windowSize",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TimeCol,
            WindowSlide,
            WindowSize,
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
                            "timeCol" => Ok(GeneratedField::TimeCol),
                            "windowSlide" => Ok(GeneratedField::WindowSlide),
                            "windowSize" => Ok(GeneratedField::WindowSize),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = HopWindowNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.HopWindowNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<HopWindowNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut time_col = None;
                let mut window_slide = None;
                let mut window_size = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TimeCol => {
                            if time_col.is_some() {
                                return Err(serde::de::Error::duplicate_field("timeCol"));
                            }
                            time_col = Some(map.next_value()?);
                        }
                        GeneratedField::WindowSlide => {
                            if window_slide.is_some() {
                                return Err(serde::de::Error::duplicate_field("windowSlide"));
                            }
                            window_slide = Some(map.next_value()?);
                        }
                        GeneratedField::WindowSize => {
                            if window_size.is_some() {
                                return Err(serde::de::Error::duplicate_field("windowSize"));
                            }
                            window_size = Some(map.next_value()?);
                        }
                    }
                }
                Ok(HopWindowNode {
                    time_col,
                    window_slide,
                    window_size,
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.HopWindowNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InsertNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.table_source_ref_id.is_some() {
            len += 1;
        }
        if !self.column_ids.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.InsertNode", len)?;
        if let Some(v) = self.table_source_ref_id.as_ref() {
            struct_ser.serialize_field("tableSourceRefId", v)?;
        }
        if !self.column_ids.is_empty() {
            struct_ser.serialize_field("columnIds", &self.column_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InsertNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tableSourceRefId",
            "columnIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableSourceRefId,
            ColumnIds,
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
                            "tableSourceRefId" => Ok(GeneratedField::TableSourceRefId),
                            "columnIds" => Ok(GeneratedField::ColumnIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InsertNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.InsertNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<InsertNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_source_ref_id = None;
                let mut column_ids = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TableSourceRefId => {
                            if table_source_ref_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableSourceRefId"));
                            }
                            table_source_ref_id = Some(map.next_value()?);
                        }
                        GeneratedField::ColumnIds => {
                            if column_ids.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnIds"));
                            }
                            column_ids = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                    }
                }
                Ok(InsertNode {
                    table_source_ref_id,
                    column_ids: column_ids.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.InsertNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LimitNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.limit != 0 {
            len += 1;
        }
        if self.offset != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.LimitNode", len)?;
        if self.limit != 0 {
            struct_ser.serialize_field("limit", &self.limit)?;
        }
        if self.offset != 0 {
            struct_ser.serialize_field("offset", &self.offset)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LimitNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "limit",
            "offset",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Limit,
            Offset,
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
                            "limit" => Ok(GeneratedField::Limit),
                            "offset" => Ok(GeneratedField::Offset),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LimitNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.LimitNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<LimitNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut limit = None;
                let mut offset = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Limit => {
                            if limit.is_some() {
                                return Err(serde::de::Error::duplicate_field("limit"));
                            }
                            limit = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::Offset => {
                            if offset.is_some() {
                                return Err(serde::de::Error::duplicate_field("offset"));
                            }
                            offset = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                    }
                }
                Ok(LimitNode {
                    limit: limit.unwrap_or_default(),
                    offset: offset.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.LimitNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MergeSortExchangeNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.exchange.is_some() {
            len += 1;
        }
        if !self.column_orders.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.MergeSortExchangeNode", len)?;
        if let Some(v) = self.exchange.as_ref() {
            struct_ser.serialize_field("exchange", v)?;
        }
        if !self.column_orders.is_empty() {
            struct_ser.serialize_field("columnOrders", &self.column_orders)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MergeSortExchangeNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "exchange",
            "columnOrders",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Exchange,
            ColumnOrders,
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
                            "exchange" => Ok(GeneratedField::Exchange),
                            "columnOrders" => Ok(GeneratedField::ColumnOrders),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MergeSortExchangeNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.MergeSortExchangeNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MergeSortExchangeNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut exchange = None;
                let mut column_orders = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Exchange => {
                            if exchange.is_some() {
                                return Err(serde::de::Error::duplicate_field("exchange"));
                            }
                            exchange = Some(map.next_value()?);
                        }
                        GeneratedField::ColumnOrders => {
                            if column_orders.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnOrders"));
                            }
                            column_orders = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MergeSortExchangeNode {
                    exchange,
                    column_orders: column_orders.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.MergeSortExchangeNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NestedLoopJoinNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.join_type != 0 {
            len += 1;
        }
        if self.join_cond.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.NestedLoopJoinNode", len)?;
        if self.join_type != 0 {
            let v = super::plan_common::JoinType::from_i32(self.join_type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.join_type)))?;
            struct_ser.serialize_field("joinType", &v)?;
        }
        if let Some(v) = self.join_cond.as_ref() {
            struct_ser.serialize_field("joinCond", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NestedLoopJoinNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "joinType",
            "joinCond",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            JoinType,
            JoinCond,
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
                            "joinType" => Ok(GeneratedField::JoinType),
                            "joinCond" => Ok(GeneratedField::JoinCond),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NestedLoopJoinNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.NestedLoopJoinNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<NestedLoopJoinNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut join_type = None;
                let mut join_cond = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::JoinType => {
                            if join_type.is_some() {
                                return Err(serde::de::Error::duplicate_field("joinType"));
                            }
                            join_type = Some(map.next_value::<super::plan_common::JoinType>()? as i32);
                        }
                        GeneratedField::JoinCond => {
                            if join_cond.is_some() {
                                return Err(serde::de::Error::duplicate_field("joinCond"));
                            }
                            join_cond = Some(map.next_value()?);
                        }
                    }
                }
                Ok(NestedLoopJoinNode {
                    join_type: join_type.unwrap_or_default(),
                    join_cond,
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.NestedLoopJoinNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OrderByNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.column_orders.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.OrderByNode", len)?;
        if !self.column_orders.is_empty() {
            struct_ser.serialize_field("columnOrders", &self.column_orders)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OrderByNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "columnOrders",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ColumnOrders,
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
                            "columnOrders" => Ok(GeneratedField::ColumnOrders),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OrderByNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.OrderByNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<OrderByNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut column_orders = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ColumnOrders => {
                            if column_orders.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnOrders"));
                            }
                            column_orders = Some(map.next_value()?);
                        }
                    }
                }
                Ok(OrderByNode {
                    column_orders: column_orders.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.OrderByNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PlanFragment {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.root.is_some() {
            len += 1;
        }
        if self.exchange_info.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.PlanFragment", len)?;
        if let Some(v) = self.root.as_ref() {
            struct_ser.serialize_field("root", v)?;
        }
        if let Some(v) = self.exchange_info.as_ref() {
            struct_ser.serialize_field("exchangeInfo", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PlanFragment {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "root",
            "exchangeInfo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Root,
            ExchangeInfo,
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
                            "root" => Ok(GeneratedField::Root),
                            "exchangeInfo" => Ok(GeneratedField::ExchangeInfo),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PlanFragment;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.PlanFragment")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PlanFragment, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut root = None;
                let mut exchange_info = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Root => {
                            if root.is_some() {
                                return Err(serde::de::Error::duplicate_field("root"));
                            }
                            root = Some(map.next_value()?);
                        }
                        GeneratedField::ExchangeInfo => {
                            if exchange_info.is_some() {
                                return Err(serde::de::Error::duplicate_field("exchangeInfo"));
                            }
                            exchange_info = Some(map.next_value()?);
                        }
                    }
                }
                Ok(PlanFragment {
                    root,
                    exchange_info,
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.PlanFragment", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PlanNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.children.is_empty() {
            len += 1;
        }
        if !self.identity.is_empty() {
            len += 1;
        }
        if self.node_body.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.PlanNode", len)?;
        if !self.children.is_empty() {
            struct_ser.serialize_field("children", &self.children)?;
        }
        if !self.identity.is_empty() {
            struct_ser.serialize_field("identity", &self.identity)?;
        }
        if let Some(v) = self.node_body.as_ref() {
            match v {
                plan_node::NodeBody::Insert(v) => {
                    struct_ser.serialize_field("insert", v)?;
                }
                plan_node::NodeBody::Delete(v) => {
                    struct_ser.serialize_field("delete", v)?;
                }
                plan_node::NodeBody::Update(v) => {
                    struct_ser.serialize_field("update", v)?;
                }
                plan_node::NodeBody::Project(v) => {
                    struct_ser.serialize_field("project", v)?;
                }
                plan_node::NodeBody::HashAgg(v) => {
                    struct_ser.serialize_field("hashAgg", v)?;
                }
                plan_node::NodeBody::Filter(v) => {
                    struct_ser.serialize_field("filter", v)?;
                }
                plan_node::NodeBody::Exchange(v) => {
                    struct_ser.serialize_field("exchange", v)?;
                }
                plan_node::NodeBody::OrderBy(v) => {
                    struct_ser.serialize_field("orderBy", v)?;
                }
                plan_node::NodeBody::NestedLoopJoin(v) => {
                    struct_ser.serialize_field("nestedLoopJoin", v)?;
                }
                plan_node::NodeBody::TopN(v) => {
                    struct_ser.serialize_field("topN", v)?;
                }
                plan_node::NodeBody::SortAgg(v) => {
                    struct_ser.serialize_field("sortAgg", v)?;
                }
                plan_node::NodeBody::RowSeqScan(v) => {
                    struct_ser.serialize_field("rowSeqScan", v)?;
                }
                plan_node::NodeBody::Limit(v) => {
                    struct_ser.serialize_field("limit", v)?;
                }
                plan_node::NodeBody::Values(v) => {
                    struct_ser.serialize_field("values", v)?;
                }
                plan_node::NodeBody::HashJoin(v) => {
                    struct_ser.serialize_field("hashJoin", v)?;
                }
                plan_node::NodeBody::MergeSortExchange(v) => {
                    struct_ser.serialize_field("mergeSortExchange", v)?;
                }
                plan_node::NodeBody::SortMergeJoin(v) => {
                    struct_ser.serialize_field("sortMergeJoin", v)?;
                }
                plan_node::NodeBody::HopWindow(v) => {
                    struct_ser.serialize_field("hopWindow", v)?;
                }
                plan_node::NodeBody::GenerateSeries(v) => {
                    struct_ser.serialize_field("generateSeries", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PlanNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "children",
            "identity",
            "insert",
            "delete",
            "update",
            "project",
            "hashAgg",
            "filter",
            "exchange",
            "orderBy",
            "nestedLoopJoin",
            "topN",
            "sortAgg",
            "rowSeqScan",
            "limit",
            "values",
            "hashJoin",
            "mergeSortExchange",
            "sortMergeJoin",
            "hopWindow",
            "generateSeries",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Children,
            Identity,
            Insert,
            Delete,
            Update,
            Project,
            HashAgg,
            Filter,
            Exchange,
            OrderBy,
            NestedLoopJoin,
            TopN,
            SortAgg,
            RowSeqScan,
            Limit,
            Values,
            HashJoin,
            MergeSortExchange,
            SortMergeJoin,
            HopWindow,
            GenerateSeries,
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
                            "children" => Ok(GeneratedField::Children),
                            "identity" => Ok(GeneratedField::Identity),
                            "insert" => Ok(GeneratedField::Insert),
                            "delete" => Ok(GeneratedField::Delete),
                            "update" => Ok(GeneratedField::Update),
                            "project" => Ok(GeneratedField::Project),
                            "hashAgg" => Ok(GeneratedField::HashAgg),
                            "filter" => Ok(GeneratedField::Filter),
                            "exchange" => Ok(GeneratedField::Exchange),
                            "orderBy" => Ok(GeneratedField::OrderBy),
                            "nestedLoopJoin" => Ok(GeneratedField::NestedLoopJoin),
                            "topN" => Ok(GeneratedField::TopN),
                            "sortAgg" => Ok(GeneratedField::SortAgg),
                            "rowSeqScan" => Ok(GeneratedField::RowSeqScan),
                            "limit" => Ok(GeneratedField::Limit),
                            "values" => Ok(GeneratedField::Values),
                            "hashJoin" => Ok(GeneratedField::HashJoin),
                            "mergeSortExchange" => Ok(GeneratedField::MergeSortExchange),
                            "sortMergeJoin" => Ok(GeneratedField::SortMergeJoin),
                            "hopWindow" => Ok(GeneratedField::HopWindow),
                            "generateSeries" => Ok(GeneratedField::GenerateSeries),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PlanNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.PlanNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PlanNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut children = None;
                let mut identity = None;
                let mut node_body = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Children => {
                            if children.is_some() {
                                return Err(serde::de::Error::duplicate_field("children"));
                            }
                            children = Some(map.next_value()?);
                        }
                        GeneratedField::Identity => {
                            if identity.is_some() {
                                return Err(serde::de::Error::duplicate_field("identity"));
                            }
                            identity = Some(map.next_value()?);
                        }
                        GeneratedField::Insert => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("insert"));
                            }
                            node_body = Some(plan_node::NodeBody::Insert(map.next_value()?));
                        }
                        GeneratedField::Delete => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("delete"));
                            }
                            node_body = Some(plan_node::NodeBody::Delete(map.next_value()?));
                        }
                        GeneratedField::Update => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("update"));
                            }
                            node_body = Some(plan_node::NodeBody::Update(map.next_value()?));
                        }
                        GeneratedField::Project => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("project"));
                            }
                            node_body = Some(plan_node::NodeBody::Project(map.next_value()?));
                        }
                        GeneratedField::HashAgg => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("hashAgg"));
                            }
                            node_body = Some(plan_node::NodeBody::HashAgg(map.next_value()?));
                        }
                        GeneratedField::Filter => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("filter"));
                            }
                            node_body = Some(plan_node::NodeBody::Filter(map.next_value()?));
                        }
                        GeneratedField::Exchange => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("exchange"));
                            }
                            node_body = Some(plan_node::NodeBody::Exchange(map.next_value()?));
                        }
                        GeneratedField::OrderBy => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("orderBy"));
                            }
                            node_body = Some(plan_node::NodeBody::OrderBy(map.next_value()?));
                        }
                        GeneratedField::NestedLoopJoin => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("nestedLoopJoin"));
                            }
                            node_body = Some(plan_node::NodeBody::NestedLoopJoin(map.next_value()?));
                        }
                        GeneratedField::TopN => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("topN"));
                            }
                            node_body = Some(plan_node::NodeBody::TopN(map.next_value()?));
                        }
                        GeneratedField::SortAgg => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("sortAgg"));
                            }
                            node_body = Some(plan_node::NodeBody::SortAgg(map.next_value()?));
                        }
                        GeneratedField::RowSeqScan => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowSeqScan"));
                            }
                            node_body = Some(plan_node::NodeBody::RowSeqScan(map.next_value()?));
                        }
                        GeneratedField::Limit => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("limit"));
                            }
                            node_body = Some(plan_node::NodeBody::Limit(map.next_value()?));
                        }
                        GeneratedField::Values => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("values"));
                            }
                            node_body = Some(plan_node::NodeBody::Values(map.next_value()?));
                        }
                        GeneratedField::HashJoin => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("hashJoin"));
                            }
                            node_body = Some(plan_node::NodeBody::HashJoin(map.next_value()?));
                        }
                        GeneratedField::MergeSortExchange => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("mergeSortExchange"));
                            }
                            node_body = Some(plan_node::NodeBody::MergeSortExchange(map.next_value()?));
                        }
                        GeneratedField::SortMergeJoin => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("sortMergeJoin"));
                            }
                            node_body = Some(plan_node::NodeBody::SortMergeJoin(map.next_value()?));
                        }
                        GeneratedField::HopWindow => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("hopWindow"));
                            }
                            node_body = Some(plan_node::NodeBody::HopWindow(map.next_value()?));
                        }
                        GeneratedField::GenerateSeries => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("generateSeries"));
                            }
                            node_body = Some(plan_node::NodeBody::GenerateSeries(map.next_value()?));
                        }
                    }
                }
                Ok(PlanNode {
                    children: children.unwrap_or_default(),
                    identity: identity.unwrap_or_default(),
                    node_body,
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.PlanNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProjectNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.select_list.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.ProjectNode", len)?;
        if !self.select_list.is_empty() {
            struct_ser.serialize_field("selectList", &self.select_list)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProjectNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "selectList",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SelectList,
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
                            "selectList" => Ok(GeneratedField::SelectList),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProjectNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.ProjectNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ProjectNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut select_list = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SelectList => {
                            if select_list.is_some() {
                                return Err(serde::de::Error::duplicate_field("selectList"));
                            }
                            select_list = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ProjectNode {
                    select_list: select_list.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.ProjectNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RowSeqScanNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.table_desc.is_some() {
            len += 1;
        }
        if !self.column_descs.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.RowSeqScanNode", len)?;
        if let Some(v) = self.table_desc.as_ref() {
            struct_ser.serialize_field("tableDesc", v)?;
        }
        if !self.column_descs.is_empty() {
            struct_ser.serialize_field("columnDescs", &self.column_descs)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RowSeqScanNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tableDesc",
            "columnDescs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableDesc,
            ColumnDescs,
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
                            "tableDesc" => Ok(GeneratedField::TableDesc),
                            "columnDescs" => Ok(GeneratedField::ColumnDescs),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RowSeqScanNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.RowSeqScanNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RowSeqScanNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_desc = None;
                let mut column_descs = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TableDesc => {
                            if table_desc.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableDesc"));
                            }
                            table_desc = Some(map.next_value()?);
                        }
                        GeneratedField::ColumnDescs => {
                            if column_descs.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnDescs"));
                            }
                            column_descs = Some(map.next_value()?);
                        }
                    }
                }
                Ok(RowSeqScanNode {
                    table_desc,
                    column_descs: column_descs.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.RowSeqScanNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SortAggNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.group_keys.is_empty() {
            len += 1;
        }
        if !self.agg_calls.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.SortAggNode", len)?;
        if !self.group_keys.is_empty() {
            struct_ser.serialize_field("groupKeys", &self.group_keys)?;
        }
        if !self.agg_calls.is_empty() {
            struct_ser.serialize_field("aggCalls", &self.agg_calls)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SortAggNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "groupKeys",
            "aggCalls",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GroupKeys,
            AggCalls,
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
                            "groupKeys" => Ok(GeneratedField::GroupKeys),
                            "aggCalls" => Ok(GeneratedField::AggCalls),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SortAggNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.SortAggNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SortAggNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut group_keys = None;
                let mut agg_calls = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::GroupKeys => {
                            if group_keys.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupKeys"));
                            }
                            group_keys = Some(map.next_value()?);
                        }
                        GeneratedField::AggCalls => {
                            if agg_calls.is_some() {
                                return Err(serde::de::Error::duplicate_field("aggCalls"));
                            }
                            agg_calls = Some(map.next_value()?);
                        }
                    }
                }
                Ok(SortAggNode {
                    group_keys: group_keys.unwrap_or_default(),
                    agg_calls: agg_calls.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.SortAggNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SortMergeJoinNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.join_type != 0 {
            len += 1;
        }
        if !self.left_keys.is_empty() {
            len += 1;
        }
        if !self.right_keys.is_empty() {
            len += 1;
        }
        if self.direction != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.SortMergeJoinNode", len)?;
        if self.join_type != 0 {
            let v = super::plan_common::JoinType::from_i32(self.join_type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.join_type)))?;
            struct_ser.serialize_field("joinType", &v)?;
        }
        if !self.left_keys.is_empty() {
            struct_ser.serialize_field("leftKeys", &self.left_keys)?;
        }
        if !self.right_keys.is_empty() {
            struct_ser.serialize_field("rightKeys", &self.right_keys)?;
        }
        if self.direction != 0 {
            let v = super::plan_common::OrderType::from_i32(self.direction)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.direction)))?;
            struct_ser.serialize_field("direction", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SortMergeJoinNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "joinType",
            "leftKeys",
            "rightKeys",
            "direction",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            JoinType,
            LeftKeys,
            RightKeys,
            Direction,
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
                            "joinType" => Ok(GeneratedField::JoinType),
                            "leftKeys" => Ok(GeneratedField::LeftKeys),
                            "rightKeys" => Ok(GeneratedField::RightKeys),
                            "direction" => Ok(GeneratedField::Direction),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SortMergeJoinNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.SortMergeJoinNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SortMergeJoinNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut join_type = None;
                let mut left_keys = None;
                let mut right_keys = None;
                let mut direction = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::JoinType => {
                            if join_type.is_some() {
                                return Err(serde::de::Error::duplicate_field("joinType"));
                            }
                            join_type = Some(map.next_value::<super::plan_common::JoinType>()? as i32);
                        }
                        GeneratedField::LeftKeys => {
                            if left_keys.is_some() {
                                return Err(serde::de::Error::duplicate_field("leftKeys"));
                            }
                            left_keys = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::RightKeys => {
                            if right_keys.is_some() {
                                return Err(serde::de::Error::duplicate_field("rightKeys"));
                            }
                            right_keys = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::Direction => {
                            if direction.is_some() {
                                return Err(serde::de::Error::duplicate_field("direction"));
                            }
                            direction = Some(map.next_value::<super::plan_common::OrderType>()? as i32);
                        }
                    }
                }
                Ok(SortMergeJoinNode {
                    join_type: join_type.unwrap_or_default(),
                    left_keys: left_keys.unwrap_or_default(),
                    right_keys: right_keys.unwrap_or_default(),
                    direction: direction.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.SortMergeJoinNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SourceScanNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.table_ref_id.is_some() {
            len += 1;
        }
        if self.timestamp_ms != 0 {
            len += 1;
        }
        if !self.column_ids.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.SourceScanNode", len)?;
        if let Some(v) = self.table_ref_id.as_ref() {
            struct_ser.serialize_field("tableRefId", v)?;
        }
        if self.timestamp_ms != 0 {
            struct_ser.serialize_field("timestampMs", ToString::to_string(&self.timestamp_ms).as_str())?;
        }
        if !self.column_ids.is_empty() {
            struct_ser.serialize_field("columnIds", &self.column_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SourceScanNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tableRefId",
            "timestampMs",
            "columnIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableRefId,
            TimestampMs,
            ColumnIds,
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
                            "tableRefId" => Ok(GeneratedField::TableRefId),
                            "timestampMs" => Ok(GeneratedField::TimestampMs),
                            "columnIds" => Ok(GeneratedField::ColumnIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SourceScanNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.SourceScanNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SourceScanNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_ref_id = None;
                let mut timestamp_ms = None;
                let mut column_ids = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TableRefId => {
                            if table_ref_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableRefId"));
                            }
                            table_ref_id = Some(map.next_value()?);
                        }
                        GeneratedField::TimestampMs => {
                            if timestamp_ms.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestampMs"));
                            }
                            timestamp_ms = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::ColumnIds => {
                            if column_ids.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnIds"));
                            }
                            column_ids = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                    }
                }
                Ok(SourceScanNode {
                    table_ref_id,
                    timestamp_ms: timestamp_ms.unwrap_or_default(),
                    column_ids: column_ids.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.SourceScanNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TaskId {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.query_id.is_empty() {
            len += 1;
        }
        if self.stage_id != 0 {
            len += 1;
        }
        if self.task_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.TaskId", len)?;
        if !self.query_id.is_empty() {
            struct_ser.serialize_field("queryId", &self.query_id)?;
        }
        if self.stage_id != 0 {
            struct_ser.serialize_field("stageId", &self.stage_id)?;
        }
        if self.task_id != 0 {
            struct_ser.serialize_field("taskId", &self.task_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TaskId {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "queryId",
            "stageId",
            "taskId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            QueryId,
            StageId,
            TaskId,
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
                            "queryId" => Ok(GeneratedField::QueryId),
                            "stageId" => Ok(GeneratedField::StageId),
                            "taskId" => Ok(GeneratedField::TaskId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TaskId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.TaskId")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TaskId, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut query_id = None;
                let mut stage_id = None;
                let mut task_id = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::QueryId => {
                            if query_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("queryId"));
                            }
                            query_id = Some(map.next_value()?);
                        }
                        GeneratedField::StageId => {
                            if stage_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("stageId"));
                            }
                            stage_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::TaskId => {
                            if task_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("taskId"));
                            }
                            task_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                    }
                }
                Ok(TaskId {
                    query_id: query_id.unwrap_or_default(),
                    stage_id: stage_id.unwrap_or_default(),
                    task_id: task_id.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.TaskId", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TaskOutputId {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.task_id.is_some() {
            len += 1;
        }
        if self.output_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.TaskOutputId", len)?;
        if let Some(v) = self.task_id.as_ref() {
            struct_ser.serialize_field("taskId", v)?;
        }
        if self.output_id != 0 {
            struct_ser.serialize_field("outputId", &self.output_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TaskOutputId {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "taskId",
            "outputId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TaskId,
            OutputId,
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
                            "taskId" => Ok(GeneratedField::TaskId),
                            "outputId" => Ok(GeneratedField::OutputId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TaskOutputId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.TaskOutputId")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TaskOutputId, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut task_id = None;
                let mut output_id = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TaskId => {
                            if task_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("taskId"));
                            }
                            task_id = Some(map.next_value()?);
                        }
                        GeneratedField::OutputId => {
                            if output_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("outputId"));
                            }
                            output_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                    }
                }
                Ok(TaskOutputId {
                    task_id,
                    output_id: output_id.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.TaskOutputId", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TopNNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.column_orders.is_empty() {
            len += 1;
        }
        if self.limit != 0 {
            len += 1;
        }
        if self.offset != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.TopNNode", len)?;
        if !self.column_orders.is_empty() {
            struct_ser.serialize_field("columnOrders", &self.column_orders)?;
        }
        if self.limit != 0 {
            struct_ser.serialize_field("limit", &self.limit)?;
        }
        if self.offset != 0 {
            struct_ser.serialize_field("offset", &self.offset)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TopNNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "columnOrders",
            "limit",
            "offset",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ColumnOrders,
            Limit,
            Offset,
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
                            "columnOrders" => Ok(GeneratedField::ColumnOrders),
                            "limit" => Ok(GeneratedField::Limit),
                            "offset" => Ok(GeneratedField::Offset),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TopNNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.TopNNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TopNNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut column_orders = None;
                let mut limit = None;
                let mut offset = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ColumnOrders => {
                            if column_orders.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnOrders"));
                            }
                            column_orders = Some(map.next_value()?);
                        }
                        GeneratedField::Limit => {
                            if limit.is_some() {
                                return Err(serde::de::Error::duplicate_field("limit"));
                            }
                            limit = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::Offset => {
                            if offset.is_some() {
                                return Err(serde::de::Error::duplicate_field("offset"));
                            }
                            offset = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                    }
                }
                Ok(TopNNode {
                    column_orders: column_orders.unwrap_or_default(),
                    limit: limit.unwrap_or_default(),
                    offset: offset.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.TopNNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdateNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.table_source_ref_id.is_some() {
            len += 1;
        }
        if !self.exprs.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.UpdateNode", len)?;
        if let Some(v) = self.table_source_ref_id.as_ref() {
            struct_ser.serialize_field("tableSourceRefId", v)?;
        }
        if !self.exprs.is_empty() {
            struct_ser.serialize_field("exprs", &self.exprs)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdateNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tableSourceRefId",
            "exprs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableSourceRefId,
            Exprs,
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
                            "tableSourceRefId" => Ok(GeneratedField::TableSourceRefId),
                            "exprs" => Ok(GeneratedField::Exprs),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdateNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.UpdateNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<UpdateNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_source_ref_id = None;
                let mut exprs = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TableSourceRefId => {
                            if table_source_ref_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableSourceRefId"));
                            }
                            table_source_ref_id = Some(map.next_value()?);
                        }
                        GeneratedField::Exprs => {
                            if exprs.is_some() {
                                return Err(serde::de::Error::duplicate_field("exprs"));
                            }
                            exprs = Some(map.next_value()?);
                        }
                    }
                }
                Ok(UpdateNode {
                    table_source_ref_id,
                    exprs: exprs.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.UpdateNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValuesNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.tuples.is_empty() {
            len += 1;
        }
        if !self.fields.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.ValuesNode", len)?;
        if !self.tuples.is_empty() {
            struct_ser.serialize_field("tuples", &self.tuples)?;
        }
        if !self.fields.is_empty() {
            struct_ser.serialize_field("fields", &self.fields)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValuesNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tuples",
            "fields",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Tuples,
            Fields,
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
                            "tuples" => Ok(GeneratedField::Tuples),
                            "fields" => Ok(GeneratedField::Fields),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValuesNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.ValuesNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValuesNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tuples = None;
                let mut fields = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Tuples => {
                            if tuples.is_some() {
                                return Err(serde::de::Error::duplicate_field("tuples"));
                            }
                            tuples = Some(map.next_value()?);
                        }
                        GeneratedField::Fields => {
                            if fields.is_some() {
                                return Err(serde::de::Error::duplicate_field("fields"));
                            }
                            fields = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ValuesNode {
                    tuples: tuples.unwrap_or_default(),
                    fields: fields.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.ValuesNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for values_node::ExprTuple {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.cells.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("batch_plan.ValuesNode.ExprTuple", len)?;
        if !self.cells.is_empty() {
            struct_ser.serialize_field("cells", &self.cells)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for values_node::ExprTuple {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "cells",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Cells,
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
                            "cells" => Ok(GeneratedField::Cells),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = values_node::ExprTuple;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct batch_plan.ValuesNode.ExprTuple")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<values_node::ExprTuple, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut cells = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Cells => {
                            if cells.is_some() {
                                return Err(serde::de::Error::duplicate_field("cells"));
                            }
                            cells = Some(map.next_value()?);
                        }
                    }
                }
                Ok(values_node::ExprTuple {
                    cells: cells.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("batch_plan.ValuesNode.ExprTuple", FIELDS, GeneratedVisitor)
    }
}
