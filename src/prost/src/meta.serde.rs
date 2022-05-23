use crate::meta::*;
impl serde::Serialize for ActivateWorkerNodeRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.host.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("meta.ActivateWorkerNodeRequest", len)?;
        if let Some(v) = self.host.as_ref() {
            struct_ser.serialize_field("host", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActivateWorkerNodeRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "host",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = ActivateWorkerNodeRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct meta.ActivateWorkerNodeRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ActivateWorkerNodeRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut host = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Host => {
                            if host.is_some() {
                                return Err(serde::de::Error::duplicate_field("host"));
                            }
                            host = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ActivateWorkerNodeRequest {
                    host,
                })
            }
        }
        deserializer.deserialize_struct("meta.ActivateWorkerNodeRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActivateWorkerNodeResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.status.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("meta.ActivateWorkerNodeResponse", len)?;
        if let Some(v) = self.status.as_ref() {
            struct_ser.serialize_field("status", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActivateWorkerNodeResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "status",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Status,
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
                            "status" => Ok(GeneratedField::Status),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActivateWorkerNodeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct meta.ActivateWorkerNodeResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ActivateWorkerNodeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut status = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Status => {
                            if status.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ActivateWorkerNodeResponse {
                    status,
                })
            }
        }
        deserializer.deserialize_struct("meta.ActivateWorkerNodeResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActorLocation {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.node.is_some() {
            len += 1;
        }
        if !self.actors.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("meta.ActorLocation", len)?;
        if let Some(v) = self.node.as_ref() {
            struct_ser.serialize_field("node", v)?;
        }
        if !self.actors.is_empty() {
            struct_ser.serialize_field("actors", &self.actors)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActorLocation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "node",
            "actors",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Node,
            Actors,
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
                            "node" => Ok(GeneratedField::Node),
                            "actors" => Ok(GeneratedField::Actors),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActorLocation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct meta.ActorLocation")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ActorLocation, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut node = None;
                let mut actors = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Node => {
                            if node.is_some() {
                                return Err(serde::de::Error::duplicate_field("node"));
                            }
                            node = Some(map.next_value()?);
                        }
                        GeneratedField::Actors => {
                            if actors.is_some() {
                                return Err(serde::de::Error::duplicate_field("actors"));
                            }
                            actors = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ActorLocation {
                    node,
                    actors: actors.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("meta.ActorLocation", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AddWorkerNodeRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.worker_type != 0 {
            len += 1;
        }
        if self.host.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("meta.AddWorkerNodeRequest", len)?;
        if self.worker_type != 0 {
            let v = super::common::WorkerType::from_i32(self.worker_type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.worker_type)))?;
            struct_ser.serialize_field("workerType", &v)?;
        }
        if let Some(v) = self.host.as_ref() {
            struct_ser.serialize_field("host", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AddWorkerNodeRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "workerType",
            "host",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WorkerType,
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
                            "workerType" => Ok(GeneratedField::WorkerType),
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
            type Value = AddWorkerNodeRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct meta.AddWorkerNodeRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AddWorkerNodeRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut worker_type = None;
                let mut host = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::WorkerType => {
                            if worker_type.is_some() {
                                return Err(serde::de::Error::duplicate_field("workerType"));
                            }
                            worker_type = Some(map.next_value::<super::common::WorkerType>()? as i32);
                        }
                        GeneratedField::Host => {
                            if host.is_some() {
                                return Err(serde::de::Error::duplicate_field("host"));
                            }
                            host = Some(map.next_value()?);
                        }
                    }
                }
                Ok(AddWorkerNodeRequest {
                    worker_type: worker_type.unwrap_or_default(),
                    host,
                })
            }
        }
        deserializer.deserialize_struct("meta.AddWorkerNodeRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AddWorkerNodeResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.status.is_some() {
            len += 1;
        }
        if self.node.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("meta.AddWorkerNodeResponse", len)?;
        if let Some(v) = self.status.as_ref() {
            struct_ser.serialize_field("status", v)?;
        }
        if let Some(v) = self.node.as_ref() {
            struct_ser.serialize_field("node", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AddWorkerNodeResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "status",
            "node",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Status,
            Node,
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
                            "status" => Ok(GeneratedField::Status),
                            "node" => Ok(GeneratedField::Node),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AddWorkerNodeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct meta.AddWorkerNodeResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AddWorkerNodeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut status = None;
                let mut node = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Status => {
                            if status.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status = Some(map.next_value()?);
                        }
                        GeneratedField::Node => {
                            if node.is_some() {
                                return Err(serde::de::Error::duplicate_field("node"));
                            }
                            node = Some(map.next_value()?);
                        }
                    }
                }
                Ok(AddWorkerNodeResponse {
                    status,
                    node,
                })
            }
        }
        deserializer.deserialize_struct("meta.AddWorkerNodeResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeleteWorkerNodeRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.host.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("meta.DeleteWorkerNodeRequest", len)?;
        if let Some(v) = self.host.as_ref() {
            struct_ser.serialize_field("host", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteWorkerNodeRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "host",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = DeleteWorkerNodeRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct meta.DeleteWorkerNodeRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DeleteWorkerNodeRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut host = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Host => {
                            if host.is_some() {
                                return Err(serde::de::Error::duplicate_field("host"));
                            }
                            host = Some(map.next_value()?);
                        }
                    }
                }
                Ok(DeleteWorkerNodeRequest {
                    host,
                })
            }
        }
        deserializer.deserialize_struct("meta.DeleteWorkerNodeRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeleteWorkerNodeResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.status.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("meta.DeleteWorkerNodeResponse", len)?;
        if let Some(v) = self.status.as_ref() {
            struct_ser.serialize_field("status", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteWorkerNodeResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "status",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Status,
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
                            "status" => Ok(GeneratedField::Status),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeleteWorkerNodeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct meta.DeleteWorkerNodeResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DeleteWorkerNodeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut status = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Status => {
                            if status.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status = Some(map.next_value()?);
                        }
                    }
                }
                Ok(DeleteWorkerNodeResponse {
                    status,
                })
            }
        }
        deserializer.deserialize_struct("meta.DeleteWorkerNodeResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FlushRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("meta.FlushRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FlushRequest {
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
            type Value = FlushRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct meta.FlushRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FlushRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(FlushRequest {
                })
            }
        }
        deserializer.deserialize_struct("meta.FlushRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FlushResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.status.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("meta.FlushResponse", len)?;
        if let Some(v) = self.status.as_ref() {
            struct_ser.serialize_field("status", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FlushResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "status",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Status,
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
                            "status" => Ok(GeneratedField::Status),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FlushResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct meta.FlushResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FlushResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut status = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Status => {
                            if status.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status = Some(map.next_value()?);
                        }
                    }
                }
                Ok(FlushResponse {
                    status,
                })
            }
        }
        deserializer.deserialize_struct("meta.FlushResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for HeartbeatRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.node_id != 0 {
            len += 1;
        }
        if self.worker_type != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("meta.HeartbeatRequest", len)?;
        if self.node_id != 0 {
            struct_ser.serialize_field("nodeId", &self.node_id)?;
        }
        if self.worker_type != 0 {
            let v = super::common::WorkerType::from_i32(self.worker_type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.worker_type)))?;
            struct_ser.serialize_field("workerType", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for HeartbeatRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "nodeId",
            "workerType",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NodeId,
            WorkerType,
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
                            "nodeId" => Ok(GeneratedField::NodeId),
                            "workerType" => Ok(GeneratedField::WorkerType),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = HeartbeatRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct meta.HeartbeatRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<HeartbeatRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut node_id = None;
                let mut worker_type = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::NodeId => {
                            if node_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("nodeId"));
                            }
                            node_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::WorkerType => {
                            if worker_type.is_some() {
                                return Err(serde::de::Error::duplicate_field("workerType"));
                            }
                            worker_type = Some(map.next_value::<super::common::WorkerType>()? as i32);
                        }
                    }
                }
                Ok(HeartbeatRequest {
                    node_id: node_id.unwrap_or_default(),
                    worker_type: worker_type.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("meta.HeartbeatRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for HeartbeatResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.status.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("meta.HeartbeatResponse", len)?;
        if let Some(v) = self.status.as_ref() {
            struct_ser.serialize_field("status", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for HeartbeatResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "status",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Status,
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
                            "status" => Ok(GeneratedField::Status),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = HeartbeatResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct meta.HeartbeatResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<HeartbeatResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut status = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Status => {
                            if status.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status = Some(map.next_value()?);
                        }
                    }
                }
                Ok(HeartbeatResponse {
                    status,
                })
            }
        }
        deserializer.deserialize_struct("meta.HeartbeatResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListAllNodesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.worker_type != 0 {
            len += 1;
        }
        if self.include_starting_nodes {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("meta.ListAllNodesRequest", len)?;
        if self.worker_type != 0 {
            let v = super::common::WorkerType::from_i32(self.worker_type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.worker_type)))?;
            struct_ser.serialize_field("workerType", &v)?;
        }
        if self.include_starting_nodes {
            struct_ser.serialize_field("includeStartingNodes", &self.include_starting_nodes)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListAllNodesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "workerType",
            "includeStartingNodes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WorkerType,
            IncludeStartingNodes,
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
                            "workerType" => Ok(GeneratedField::WorkerType),
                            "includeStartingNodes" => Ok(GeneratedField::IncludeStartingNodes),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ListAllNodesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct meta.ListAllNodesRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ListAllNodesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut worker_type = None;
                let mut include_starting_nodes = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::WorkerType => {
                            if worker_type.is_some() {
                                return Err(serde::de::Error::duplicate_field("workerType"));
                            }
                            worker_type = Some(map.next_value::<super::common::WorkerType>()? as i32);
                        }
                        GeneratedField::IncludeStartingNodes => {
                            if include_starting_nodes.is_some() {
                                return Err(serde::de::Error::duplicate_field("includeStartingNodes"));
                            }
                            include_starting_nodes = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ListAllNodesRequest {
                    worker_type: worker_type.unwrap_or_default(),
                    include_starting_nodes: include_starting_nodes.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("meta.ListAllNodesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListAllNodesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.status.is_some() {
            len += 1;
        }
        if !self.nodes.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("meta.ListAllNodesResponse", len)?;
        if let Some(v) = self.status.as_ref() {
            struct_ser.serialize_field("status", v)?;
        }
        if !self.nodes.is_empty() {
            struct_ser.serialize_field("nodes", &self.nodes)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListAllNodesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "status",
            "nodes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Status,
            Nodes,
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
                            "status" => Ok(GeneratedField::Status),
                            "nodes" => Ok(GeneratedField::Nodes),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ListAllNodesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct meta.ListAllNodesResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ListAllNodesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut status = None;
                let mut nodes = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Status => {
                            if status.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status = Some(map.next_value()?);
                        }
                        GeneratedField::Nodes => {
                            if nodes.is_some() {
                                return Err(serde::de::Error::duplicate_field("nodes"));
                            }
                            nodes = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ListAllNodesResponse {
                    status,
                    nodes: nodes.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("meta.ListAllNodesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MetaSnapshot {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.nodes.is_empty() {
            len += 1;
        }
        if !self.database.is_empty() {
            len += 1;
        }
        if !self.schema.is_empty() {
            len += 1;
        }
        if !self.source.is_empty() {
            len += 1;
        }
        if !self.table.is_empty() {
            len += 1;
        }
        if !self.view.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("meta.MetaSnapshot", len)?;
        if !self.nodes.is_empty() {
            struct_ser.serialize_field("nodes", &self.nodes)?;
        }
        if !self.database.is_empty() {
            struct_ser.serialize_field("database", &self.database)?;
        }
        if !self.schema.is_empty() {
            struct_ser.serialize_field("schema", &self.schema)?;
        }
        if !self.source.is_empty() {
            struct_ser.serialize_field("source", &self.source)?;
        }
        if !self.table.is_empty() {
            struct_ser.serialize_field("table", &self.table)?;
        }
        if !self.view.is_empty() {
            struct_ser.serialize_field("view", &self.view)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MetaSnapshot {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "nodes",
            "database",
            "schema",
            "source",
            "table",
            "view",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Nodes,
            Database,
            Schema,
            Source,
            Table,
            View,
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
                            "nodes" => Ok(GeneratedField::Nodes),
                            "database" => Ok(GeneratedField::Database),
                            "schema" => Ok(GeneratedField::Schema),
                            "source" => Ok(GeneratedField::Source),
                            "table" => Ok(GeneratedField::Table),
                            "view" => Ok(GeneratedField::View),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MetaSnapshot;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct meta.MetaSnapshot")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MetaSnapshot, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut nodes = None;
                let mut database = None;
                let mut schema = None;
                let mut source = None;
                let mut table = None;
                let mut view = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Nodes => {
                            if nodes.is_some() {
                                return Err(serde::de::Error::duplicate_field("nodes"));
                            }
                            nodes = Some(map.next_value()?);
                        }
                        GeneratedField::Database => {
                            if database.is_some() {
                                return Err(serde::de::Error::duplicate_field("database"));
                            }
                            database = Some(map.next_value()?);
                        }
                        GeneratedField::Schema => {
                            if schema.is_some() {
                                return Err(serde::de::Error::duplicate_field("schema"));
                            }
                            schema = Some(map.next_value()?);
                        }
                        GeneratedField::Source => {
                            if source.is_some() {
                                return Err(serde::de::Error::duplicate_field("source"));
                            }
                            source = Some(map.next_value()?);
                        }
                        GeneratedField::Table => {
                            if table.is_some() {
                                return Err(serde::de::Error::duplicate_field("table"));
                            }
                            table = Some(map.next_value()?);
                        }
                        GeneratedField::View => {
                            if view.is_some() {
                                return Err(serde::de::Error::duplicate_field("view"));
                            }
                            view = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MetaSnapshot {
                    nodes: nodes.unwrap_or_default(),
                    database: database.unwrap_or_default(),
                    schema: schema.unwrap_or_default(),
                    source: source.unwrap_or_default(),
                    table: table.unwrap_or_default(),
                    view: view.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("meta.MetaSnapshot", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SubscribeRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.worker_type != 0 {
            len += 1;
        }
        if self.host.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("meta.SubscribeRequest", len)?;
        if self.worker_type != 0 {
            let v = super::common::WorkerType::from_i32(self.worker_type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.worker_type)))?;
            struct_ser.serialize_field("workerType", &v)?;
        }
        if let Some(v) = self.host.as_ref() {
            struct_ser.serialize_field("host", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SubscribeRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "workerType",
            "host",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WorkerType,
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
                            "workerType" => Ok(GeneratedField::WorkerType),
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
            type Value = SubscribeRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct meta.SubscribeRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SubscribeRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut worker_type = None;
                let mut host = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::WorkerType => {
                            if worker_type.is_some() {
                                return Err(serde::de::Error::duplicate_field("workerType"));
                            }
                            worker_type = Some(map.next_value::<super::common::WorkerType>()? as i32);
                        }
                        GeneratedField::Host => {
                            if host.is_some() {
                                return Err(serde::de::Error::duplicate_field("host"));
                            }
                            host = Some(map.next_value()?);
                        }
                    }
                }
                Ok(SubscribeRequest {
                    worker_type: worker_type.unwrap_or_default(),
                    host,
                })
            }
        }
        deserializer.deserialize_struct("meta.SubscribeRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SubscribeResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.status.is_some() {
            len += 1;
        }
        if self.operation != 0 {
            len += 1;
        }
        if self.version != 0 {
            len += 1;
        }
        if self.info.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("meta.SubscribeResponse", len)?;
        if let Some(v) = self.status.as_ref() {
            struct_ser.serialize_field("status", v)?;
        }
        if self.operation != 0 {
            let v = subscribe_response::Operation::from_i32(self.operation)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.operation)))?;
            struct_ser.serialize_field("operation", &v)?;
        }
        if self.version != 0 {
            struct_ser.serialize_field("version", ToString::to_string(&self.version).as_str())?;
        }
        if let Some(v) = self.info.as_ref() {
            match v {
                subscribe_response::Info::Node(v) => {
                    struct_ser.serialize_field("node", v)?;
                }
                subscribe_response::Info::Database(v) => {
                    struct_ser.serialize_field("database", v)?;
                }
                subscribe_response::Info::Schema(v) => {
                    struct_ser.serialize_field("schema", v)?;
                }
                subscribe_response::Info::Table(v) => {
                    struct_ser.serialize_field("table", v)?;
                }
                subscribe_response::Info::Source(v) => {
                    struct_ser.serialize_field("source", v)?;
                }
                subscribe_response::Info::Snapshot(v) => {
                    struct_ser.serialize_field("snapshot", v)?;
                }
                subscribe_response::Info::HummockSnapshot(v) => {
                    struct_ser.serialize_field("hummockSnapshot", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SubscribeResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "status",
            "operation",
            "version",
            "node",
            "database",
            "schema",
            "table",
            "source",
            "snapshot",
            "hummockSnapshot",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Status,
            Operation,
            Version,
            Node,
            Database,
            Schema,
            Table,
            Source,
            Snapshot,
            HummockSnapshot,
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
                            "status" => Ok(GeneratedField::Status),
                            "operation" => Ok(GeneratedField::Operation),
                            "version" => Ok(GeneratedField::Version),
                            "node" => Ok(GeneratedField::Node),
                            "database" => Ok(GeneratedField::Database),
                            "schema" => Ok(GeneratedField::Schema),
                            "table" => Ok(GeneratedField::Table),
                            "source" => Ok(GeneratedField::Source),
                            "snapshot" => Ok(GeneratedField::Snapshot),
                            "hummockSnapshot" => Ok(GeneratedField::HummockSnapshot),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SubscribeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct meta.SubscribeResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SubscribeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut status = None;
                let mut operation = None;
                let mut version = None;
                let mut info = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Status => {
                            if status.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status = Some(map.next_value()?);
                        }
                        GeneratedField::Operation => {
                            if operation.is_some() {
                                return Err(serde::de::Error::duplicate_field("operation"));
                            }
                            operation = Some(map.next_value::<subscribe_response::Operation>()? as i32);
                        }
                        GeneratedField::Version => {
                            if version.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::Node => {
                            if info.is_some() {
                                return Err(serde::de::Error::duplicate_field("node"));
                            }
                            info = Some(subscribe_response::Info::Node(map.next_value()?));
                        }
                        GeneratedField::Database => {
                            if info.is_some() {
                                return Err(serde::de::Error::duplicate_field("database"));
                            }
                            info = Some(subscribe_response::Info::Database(map.next_value()?));
                        }
                        GeneratedField::Schema => {
                            if info.is_some() {
                                return Err(serde::de::Error::duplicate_field("schema"));
                            }
                            info = Some(subscribe_response::Info::Schema(map.next_value()?));
                        }
                        GeneratedField::Table => {
                            if info.is_some() {
                                return Err(serde::de::Error::duplicate_field("table"));
                            }
                            info = Some(subscribe_response::Info::Table(map.next_value()?));
                        }
                        GeneratedField::Source => {
                            if info.is_some() {
                                return Err(serde::de::Error::duplicate_field("source"));
                            }
                            info = Some(subscribe_response::Info::Source(map.next_value()?));
                        }
                        GeneratedField::Snapshot => {
                            if info.is_some() {
                                return Err(serde::de::Error::duplicate_field("snapshot"));
                            }
                            info = Some(subscribe_response::Info::Snapshot(map.next_value()?));
                        }
                        GeneratedField::HummockSnapshot => {
                            if info.is_some() {
                                return Err(serde::de::Error::duplicate_field("hummockSnapshot"));
                            }
                            info = Some(subscribe_response::Info::HummockSnapshot(map.next_value()?));
                        }
                    }
                }
                Ok(SubscribeResponse {
                    status,
                    operation: operation.unwrap_or_default(),
                    version: version.unwrap_or_default(),
                    info,
                })
            }
        }
        deserializer.deserialize_struct("meta.SubscribeResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for subscribe_response::Operation {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Invalid => "INVALID",
            Self::Add => "ADD",
            Self::Delete => "DELETE",
            Self::Update => "UPDATE",
            Self::Snapshot => "SNAPSHOT",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for subscribe_response::Operation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "INVALID",
            "ADD",
            "DELETE",
            "UPDATE",
            "SNAPSHOT",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = subscribe_response::Operation;

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
                    .and_then(subscribe_response::Operation::from_i32)
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
                    .and_then(subscribe_response::Operation::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "INVALID" => Ok(subscribe_response::Operation::Invalid),
                    "ADD" => Ok(subscribe_response::Operation::Add),
                    "DELETE" => Ok(subscribe_response::Operation::Delete),
                    "UPDATE" => Ok(subscribe_response::Operation::Update),
                    "SNAPSHOT" => Ok(subscribe_response::Operation::Snapshot),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for TableFragments {
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
        if !self.fragments.is_empty() {
            len += 1;
        }
        if !self.actor_status.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("meta.TableFragments", len)?;
        if self.table_id != 0 {
            struct_ser.serialize_field("tableId", &self.table_id)?;
        }
        if !self.fragments.is_empty() {
            struct_ser.serialize_field("fragments", &self.fragments)?;
        }
        if !self.actor_status.is_empty() {
            struct_ser.serialize_field("actorStatus", &self.actor_status)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TableFragments {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tableId",
            "fragments",
            "actorStatus",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableId,
            Fragments,
            ActorStatus,
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
                            "fragments" => Ok(GeneratedField::Fragments),
                            "actorStatus" => Ok(GeneratedField::ActorStatus),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TableFragments;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct meta.TableFragments")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TableFragments, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_id = None;
                let mut fragments = None;
                let mut actor_status = None;
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
                        GeneratedField::Fragments => {
                            if fragments.is_some() {
                                return Err(serde::de::Error::duplicate_field("fragments"));
                            }
                            fragments = Some(
                                map.next_value::<std::collections::HashMap<::pbjson::private::NumberDeserialize<u32>, _>>()?
                                    .into_iter().map(|(k,v)| (k.0, v)).collect()
                            );
                        }
                        GeneratedField::ActorStatus => {
                            if actor_status.is_some() {
                                return Err(serde::de::Error::duplicate_field("actorStatus"));
                            }
                            actor_status = Some(
                                map.next_value::<std::collections::HashMap<::pbjson::private::NumberDeserialize<u32>, _>>()?
                                    .into_iter().map(|(k,v)| (k.0, v)).collect()
                            );
                        }
                    }
                }
                Ok(TableFragments {
                    table_id: table_id.unwrap_or_default(),
                    fragments: fragments.unwrap_or_default(),
                    actor_status: actor_status.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("meta.TableFragments", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for table_fragments::ActorState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Inactive => "INACTIVE",
            Self::Running => "RUNNING",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for table_fragments::ActorState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "INACTIVE",
            "RUNNING",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = table_fragments::ActorState;

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
                    .and_then(table_fragments::ActorState::from_i32)
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
                    .and_then(table_fragments::ActorState::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "INACTIVE" => Ok(table_fragments::ActorState::Inactive),
                    "RUNNING" => Ok(table_fragments::ActorState::Running),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for table_fragments::ActorStatus {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.parallel_unit.is_some() {
            len += 1;
        }
        if self.state != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("meta.TableFragments.ActorStatus", len)?;
        if let Some(v) = self.parallel_unit.as_ref() {
            struct_ser.serialize_field("parallelUnit", v)?;
        }
        if self.state != 0 {
            let v = table_fragments::ActorState::from_i32(self.state)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.state)))?;
            struct_ser.serialize_field("state", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for table_fragments::ActorStatus {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "parallelUnit",
            "state",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ParallelUnit,
            State,
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
                            "parallelUnit" => Ok(GeneratedField::ParallelUnit),
                            "state" => Ok(GeneratedField::State),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = table_fragments::ActorStatus;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct meta.TableFragments.ActorStatus")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<table_fragments::ActorStatus, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut parallel_unit = None;
                let mut state = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ParallelUnit => {
                            if parallel_unit.is_some() {
                                return Err(serde::de::Error::duplicate_field("parallelUnit"));
                            }
                            parallel_unit = Some(map.next_value()?);
                        }
                        GeneratedField::State => {
                            if state.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state = Some(map.next_value::<table_fragments::ActorState>()? as i32);
                        }
                    }
                }
                Ok(table_fragments::ActorStatus {
                    parallel_unit,
                    state: state.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("meta.TableFragments.ActorStatus", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for table_fragments::Fragment {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.fragment_id != 0 {
            len += 1;
        }
        if self.fragment_type != 0 {
            len += 1;
        }
        if self.distribution_type != 0 {
            len += 1;
        }
        if !self.actors.is_empty() {
            len += 1;
        }
        if self.vnode_mapping.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("meta.TableFragments.Fragment", len)?;
        if self.fragment_id != 0 {
            struct_ser.serialize_field("fragmentId", &self.fragment_id)?;
        }
        if self.fragment_type != 0 {
            let v = super::stream_plan::FragmentType::from_i32(self.fragment_type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.fragment_type)))?;
            struct_ser.serialize_field("fragmentType", &v)?;
        }
        if self.distribution_type != 0 {
            let v = table_fragments::fragment::FragmentDistributionType::from_i32(self.distribution_type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.distribution_type)))?;
            struct_ser.serialize_field("distributionType", &v)?;
        }
        if !self.actors.is_empty() {
            struct_ser.serialize_field("actors", &self.actors)?;
        }
        if let Some(v) = self.vnode_mapping.as_ref() {
            struct_ser.serialize_field("vnodeMapping", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for table_fragments::Fragment {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fragmentId",
            "fragmentType",
            "distributionType",
            "actors",
            "vnodeMapping",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FragmentId,
            FragmentType,
            DistributionType,
            Actors,
            VnodeMapping,
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
                            "fragmentId" => Ok(GeneratedField::FragmentId),
                            "fragmentType" => Ok(GeneratedField::FragmentType),
                            "distributionType" => Ok(GeneratedField::DistributionType),
                            "actors" => Ok(GeneratedField::Actors),
                            "vnodeMapping" => Ok(GeneratedField::VnodeMapping),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = table_fragments::Fragment;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct meta.TableFragments.Fragment")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<table_fragments::Fragment, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fragment_id = None;
                let mut fragment_type = None;
                let mut distribution_type = None;
                let mut actors = None;
                let mut vnode_mapping = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FragmentId => {
                            if fragment_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("fragmentId"));
                            }
                            fragment_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::FragmentType => {
                            if fragment_type.is_some() {
                                return Err(serde::de::Error::duplicate_field("fragmentType"));
                            }
                            fragment_type = Some(map.next_value::<super::stream_plan::FragmentType>()? as i32);
                        }
                        GeneratedField::DistributionType => {
                            if distribution_type.is_some() {
                                return Err(serde::de::Error::duplicate_field("distributionType"));
                            }
                            distribution_type = Some(map.next_value::<table_fragments::fragment::FragmentDistributionType>()? as i32);
                        }
                        GeneratedField::Actors => {
                            if actors.is_some() {
                                return Err(serde::de::Error::duplicate_field("actors"));
                            }
                            actors = Some(map.next_value()?);
                        }
                        GeneratedField::VnodeMapping => {
                            if vnode_mapping.is_some() {
                                return Err(serde::de::Error::duplicate_field("vnodeMapping"));
                            }
                            vnode_mapping = Some(map.next_value()?);
                        }
                    }
                }
                Ok(table_fragments::Fragment {
                    fragment_id: fragment_id.unwrap_or_default(),
                    fragment_type: fragment_type.unwrap_or_default(),
                    distribution_type: distribution_type.unwrap_or_default(),
                    actors: actors.unwrap_or_default(),
                    vnode_mapping,
                })
            }
        }
        deserializer.deserialize_struct("meta.TableFragments.Fragment", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for table_fragments::fragment::FragmentDistributionType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Single => "SINGLE",
            Self::Hash => "HASH",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for table_fragments::fragment::FragmentDistributionType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "SINGLE",
            "HASH",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = table_fragments::fragment::FragmentDistributionType;

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
                    .and_then(table_fragments::fragment::FragmentDistributionType::from_i32)
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
                    .and_then(table_fragments::fragment::FragmentDistributionType::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "SINGLE" => Ok(table_fragments::fragment::FragmentDistributionType::Single),
                    "HASH" => Ok(table_fragments::fragment::FragmentDistributionType::Hash),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
