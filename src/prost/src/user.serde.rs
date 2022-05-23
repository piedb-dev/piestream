use crate::user::*;
impl serde::Serialize for AuthInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.encryption_type != 0 {
            len += 1;
        }
        if !self.encrypted_value.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("user.AuthInfo", len)?;
        if self.encryption_type != 0 {
            let v = auth_info::EncryptionType::from_i32(self.encryption_type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.encryption_type)))?;
            struct_ser.serialize_field("encryptionType", &v)?;
        }
        if !self.encrypted_value.is_empty() {
            struct_ser.serialize_field("encryptedValue", pbjson::private::base64::encode(&self.encrypted_value).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuthInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "encryptionType",
            "encryptedValue",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EncryptionType,
            EncryptedValue,
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
                            "encryptionType" => Ok(GeneratedField::EncryptionType),
                            "encryptedValue" => Ok(GeneratedField::EncryptedValue),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct user.AuthInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AuthInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut encryption_type = None;
                let mut encrypted_value = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::EncryptionType => {
                            if encryption_type.is_some() {
                                return Err(serde::de::Error::duplicate_field("encryptionType"));
                            }
                            encryption_type = Some(map.next_value::<auth_info::EncryptionType>()? as i32);
                        }
                        GeneratedField::EncryptedValue => {
                            if encrypted_value.is_some() {
                                return Err(serde::de::Error::duplicate_field("encryptedValue"));
                            }
                            encrypted_value = Some(
                                map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0
                            );
                        }
                    }
                }
                Ok(AuthInfo {
                    encryption_type: encryption_type.unwrap_or_default(),
                    encrypted_value: encrypted_value.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("user.AuthInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for auth_info::EncryptionType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unknown => "UNKNOWN",
            Self::Plaintext => "PLAINTEXT",
            Self::Sha256 => "SHA256",
            Self::Md5 => "MD5",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for auth_info::EncryptionType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "UNKNOWN",
            "PLAINTEXT",
            "SHA256",
            "MD5",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = auth_info::EncryptionType;

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
                    .and_then(auth_info::EncryptionType::from_i32)
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
                    .and_then(auth_info::EncryptionType::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "UNKNOWN" => Ok(auth_info::EncryptionType::Unknown),
                    "PLAINTEXT" => Ok(auth_info::EncryptionType::Plaintext),
                    "SHA256" => Ok(auth_info::EncryptionType::Sha256),
                    "MD5" => Ok(auth_info::EncryptionType::Md5),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for CreateUserRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.user.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("user.CreateUserRequest", len)?;
        if let Some(v) = self.user.as_ref() {
            struct_ser.serialize_field("user", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateUserRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            User,
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
                            "user" => Ok(GeneratedField::User),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateUserRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct user.CreateUserRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CreateUserRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::User => {
                            if user.is_some() {
                                return Err(serde::de::Error::duplicate_field("user"));
                            }
                            user = Some(map.next_value()?);
                        }
                    }
                }
                Ok(CreateUserRequest {
                    user,
                })
            }
        }
        deserializer.deserialize_struct("user.CreateUserRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateUserResponse {
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
        if self.user_id != 0 {
            len += 1;
        }
        if self.version != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("user.CreateUserResponse", len)?;
        if let Some(v) = self.status.as_ref() {
            struct_ser.serialize_field("status", v)?;
        }
        if self.user_id != 0 {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        if self.version != 0 {
            struct_ser.serialize_field("version", ToString::to_string(&self.version).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateUserResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "status",
            "userId",
            "version",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Status,
            UserId,
            Version,
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
                            "userId" => Ok(GeneratedField::UserId),
                            "version" => Ok(GeneratedField::Version),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateUserResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct user.CreateUserResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CreateUserResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut status = None;
                let mut user_id = None;
                let mut version = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Status => {
                            if status.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status = Some(map.next_value()?);
                        }
                        GeneratedField::UserId => {
                            if user_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::Version => {
                            if version.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                    }
                }
                Ok(CreateUserResponse {
                    status,
                    user_id: user_id.unwrap_or_default(),
                    version: version.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("user.CreateUserResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DropUserRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.user_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("user.DropUserRequest", len)?;
        if self.user_id != 0 {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DropUserRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "userId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserId,
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
                            "userId" => Ok(GeneratedField::UserId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DropUserRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct user.DropUserRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DropUserRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_id = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::UserId => {
                            if user_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                    }
                }
                Ok(DropUserRequest {
                    user_id: user_id.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("user.DropUserRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DropUserResponse {
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
        if self.version != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("user.DropUserResponse", len)?;
        if let Some(v) = self.status.as_ref() {
            struct_ser.serialize_field("status", v)?;
        }
        if self.version != 0 {
            struct_ser.serialize_field("version", ToString::to_string(&self.version).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DropUserResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "status",
            "version",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Status,
            Version,
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
                            "version" => Ok(GeneratedField::Version),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DropUserResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct user.DropUserResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DropUserResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut status = None;
                let mut version = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Status => {
                            if status.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status = Some(map.next_value()?);
                        }
                        GeneratedField::Version => {
                            if version.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                    }
                }
                Ok(DropUserResponse {
                    status,
                    version: version.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("user.DropUserResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GrantPrivilege {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.privileges.is_empty() {
            len += 1;
        }
        if self.with_grant_option {
            len += 1;
        }
        if self.target.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("user.GrantPrivilege", len)?;
        if !self.privileges.is_empty() {
            let v = self.privileges.iter().cloned().map(|v| {
                grant_privilege::Privilege::from_i32(v)
                    .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", v)))
                }).collect::<Result<Vec<_>, _>>()?;
            struct_ser.serialize_field("privileges", &v)?;
        }
        if self.with_grant_option {
            struct_ser.serialize_field("withGrantOption", &self.with_grant_option)?;
        }
        if let Some(v) = self.target.as_ref() {
            match v {
                grant_privilege::Target::GrantDatabase(v) => {
                    struct_ser.serialize_field("grantDatabase", v)?;
                }
                grant_privilege::Target::GrantSchema(v) => {
                    struct_ser.serialize_field("grantSchema", v)?;
                }
                grant_privilege::Target::GrantTable(v) => {
                    struct_ser.serialize_field("grantTable", v)?;
                }
                grant_privilege::Target::GrantAllTables(v) => {
                    struct_ser.serialize_field("grantAllTables", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GrantPrivilege {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "privileges",
            "withGrantOption",
            "grantDatabase",
            "grantSchema",
            "grantTable",
            "grantAllTables",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Privileges,
            WithGrantOption,
            GrantDatabase,
            GrantSchema,
            GrantTable,
            GrantAllTables,
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
                            "privileges" => Ok(GeneratedField::Privileges),
                            "withGrantOption" => Ok(GeneratedField::WithGrantOption),
                            "grantDatabase" => Ok(GeneratedField::GrantDatabase),
                            "grantSchema" => Ok(GeneratedField::GrantSchema),
                            "grantTable" => Ok(GeneratedField::GrantTable),
                            "grantAllTables" => Ok(GeneratedField::GrantAllTables),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GrantPrivilege;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct user.GrantPrivilege")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GrantPrivilege, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut privileges = None;
                let mut with_grant_option = None;
                let mut target = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Privileges => {
                            if privileges.is_some() {
                                return Err(serde::de::Error::duplicate_field("privileges"));
                            }
                            privileges = Some(map.next_value::<Vec<grant_privilege::Privilege>>()?.into_iter().map(|x| x as i32).collect());
                        }
                        GeneratedField::WithGrantOption => {
                            if with_grant_option.is_some() {
                                return Err(serde::de::Error::duplicate_field("withGrantOption"));
                            }
                            with_grant_option = Some(map.next_value()?);
                        }
                        GeneratedField::GrantDatabase => {
                            if target.is_some() {
                                return Err(serde::de::Error::duplicate_field("grantDatabase"));
                            }
                            target = Some(grant_privilege::Target::GrantDatabase(map.next_value()?));
                        }
                        GeneratedField::GrantSchema => {
                            if target.is_some() {
                                return Err(serde::de::Error::duplicate_field("grantSchema"));
                            }
                            target = Some(grant_privilege::Target::GrantSchema(map.next_value()?));
                        }
                        GeneratedField::GrantTable => {
                            if target.is_some() {
                                return Err(serde::de::Error::duplicate_field("grantTable"));
                            }
                            target = Some(grant_privilege::Target::GrantTable(map.next_value()?));
                        }
                        GeneratedField::GrantAllTables => {
                            if target.is_some() {
                                return Err(serde::de::Error::duplicate_field("grantAllTables"));
                            }
                            target = Some(grant_privilege::Target::GrantAllTables(map.next_value()?));
                        }
                    }
                }
                Ok(GrantPrivilege {
                    privileges: privileges.unwrap_or_default(),
                    with_grant_option: with_grant_option.unwrap_or_default(),
                    target,
                })
            }
        }
        deserializer.deserialize_struct("user.GrantPrivilege", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for grant_privilege::GrantAllTables {
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
        if self.schema_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("user.GrantPrivilege.GrantAllTables", len)?;
        if self.database_id != 0 {
            struct_ser.serialize_field("databaseId", &self.database_id)?;
        }
        if self.schema_id != 0 {
            struct_ser.serialize_field("schemaId", &self.schema_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for grant_privilege::GrantAllTables {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "databaseId",
            "schemaId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DatabaseId,
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
                            "databaseId" => Ok(GeneratedField::DatabaseId),
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
            type Value = grant_privilege::GrantAllTables;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct user.GrantPrivilege.GrantAllTables")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<grant_privilege::GrantAllTables, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut database_id = None;
                let mut schema_id = None;
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
                Ok(grant_privilege::GrantAllTables {
                    database_id: database_id.unwrap_or_default(),
                    schema_id: schema_id.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("user.GrantPrivilege.GrantAllTables", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for grant_privilege::GrantDatabase {
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
        let mut struct_ser = serializer.serialize_struct("user.GrantPrivilege.GrantDatabase", len)?;
        if self.database_id != 0 {
            struct_ser.serialize_field("databaseId", &self.database_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for grant_privilege::GrantDatabase {
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
            type Value = grant_privilege::GrantDatabase;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct user.GrantPrivilege.GrantDatabase")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<grant_privilege::GrantDatabase, V::Error>
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
                Ok(grant_privilege::GrantDatabase {
                    database_id: database_id.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("user.GrantPrivilege.GrantDatabase", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for grant_privilege::GrantSchema {
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
        if self.schema_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("user.GrantPrivilege.GrantSchema", len)?;
        if self.database_id != 0 {
            struct_ser.serialize_field("databaseId", &self.database_id)?;
        }
        if self.schema_id != 0 {
            struct_ser.serialize_field("schemaId", &self.schema_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for grant_privilege::GrantSchema {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "databaseId",
            "schemaId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DatabaseId,
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
                            "databaseId" => Ok(GeneratedField::DatabaseId),
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
            type Value = grant_privilege::GrantSchema;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct user.GrantPrivilege.GrantSchema")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<grant_privilege::GrantSchema, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut database_id = None;
                let mut schema_id = None;
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
                Ok(grant_privilege::GrantSchema {
                    database_id: database_id.unwrap_or_default(),
                    schema_id: schema_id.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("user.GrantPrivilege.GrantSchema", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for grant_privilege::GrantTable {
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
        if self.schema_id != 0 {
            len += 1;
        }
        if self.table_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("user.GrantPrivilege.GrantTable", len)?;
        if self.database_id != 0 {
            struct_ser.serialize_field("databaseId", &self.database_id)?;
        }
        if self.schema_id != 0 {
            struct_ser.serialize_field("schemaId", &self.schema_id)?;
        }
        if self.table_id != 0 {
            struct_ser.serialize_field("tableId", &self.table_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for grant_privilege::GrantTable {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "databaseId",
            "schemaId",
            "tableId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DatabaseId,
            SchemaId,
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
                            "databaseId" => Ok(GeneratedField::DatabaseId),
                            "schemaId" => Ok(GeneratedField::SchemaId),
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
            type Value = grant_privilege::GrantTable;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct user.GrantPrivilege.GrantTable")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<grant_privilege::GrantTable, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut database_id = None;
                let mut schema_id = None;
                let mut table_id = None;
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
                        GeneratedField::SchemaId => {
                            if schema_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("schemaId"));
                            }
                            schema_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
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
                Ok(grant_privilege::GrantTable {
                    database_id: database_id.unwrap_or_default(),
                    schema_id: schema_id.unwrap_or_default(),
                    table_id: table_id.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("user.GrantPrivilege.GrantTable", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for grant_privilege::Privilege {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unknown => "UNKNOWN",
            Self::Select => "SELECT",
            Self::Insert => "INSERT",
            Self::Update => "UPDATE",
            Self::Delete => "DELETE",
            Self::Create => "CREATE",
            Self::Connect => "CONNECT",
            Self::All => "ALL",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for grant_privilege::Privilege {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "UNKNOWN",
            "SELECT",
            "INSERT",
            "UPDATE",
            "DELETE",
            "CREATE",
            "CONNECT",
            "ALL",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = grant_privilege::Privilege;

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
                    .and_then(grant_privilege::Privilege::from_i32)
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
                    .and_then(grant_privilege::Privilege::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "UNKNOWN" => Ok(grant_privilege::Privilege::Unknown),
                    "SELECT" => Ok(grant_privilege::Privilege::Select),
                    "INSERT" => Ok(grant_privilege::Privilege::Insert),
                    "UPDATE" => Ok(grant_privilege::Privilege::Update),
                    "DELETE" => Ok(grant_privilege::Privilege::Delete),
                    "CREATE" => Ok(grant_privilege::Privilege::Create),
                    "CONNECT" => Ok(grant_privilege::Privilege::Connect),
                    "ALL" => Ok(grant_privilege::Privilege::All),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for GrantPrivilegeRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.user_id != 0 {
            len += 1;
        }
        if self.privilege.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("user.GrantPrivilegeRequest", len)?;
        if self.user_id != 0 {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        if let Some(v) = self.privilege.as_ref() {
            struct_ser.serialize_field("privilege", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GrantPrivilegeRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "userId",
            "privilege",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserId,
            Privilege,
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
                            "userId" => Ok(GeneratedField::UserId),
                            "privilege" => Ok(GeneratedField::Privilege),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GrantPrivilegeRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct user.GrantPrivilegeRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GrantPrivilegeRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_id = None;
                let mut privilege = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::UserId => {
                            if user_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::Privilege => {
                            if privilege.is_some() {
                                return Err(serde::de::Error::duplicate_field("privilege"));
                            }
                            privilege = Some(map.next_value()?);
                        }
                    }
                }
                Ok(GrantPrivilegeRequest {
                    user_id: user_id.unwrap_or_default(),
                    privilege,
                })
            }
        }
        deserializer.deserialize_struct("user.GrantPrivilegeRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GrantPrivilegeResponse {
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
        if self.version != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("user.GrantPrivilegeResponse", len)?;
        if let Some(v) = self.status.as_ref() {
            struct_ser.serialize_field("status", v)?;
        }
        if self.version != 0 {
            struct_ser.serialize_field("version", ToString::to_string(&self.version).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GrantPrivilegeResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "status",
            "version",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Status,
            Version,
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
                            "version" => Ok(GeneratedField::Version),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GrantPrivilegeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct user.GrantPrivilegeResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GrantPrivilegeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut status = None;
                let mut version = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Status => {
                            if status.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status = Some(map.next_value()?);
                        }
                        GeneratedField::Version => {
                            if version.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                    }
                }
                Ok(GrantPrivilegeResponse {
                    status,
                    version: version.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("user.GrantPrivilegeResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RevokePrivilegeRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.user_id != 0 {
            len += 1;
        }
        if self.privilege.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("user.RevokePrivilegeRequest", len)?;
        if self.user_id != 0 {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        if let Some(v) = self.privilege.as_ref() {
            struct_ser.serialize_field("privilege", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RevokePrivilegeRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "userId",
            "privilege",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserId,
            Privilege,
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
                            "userId" => Ok(GeneratedField::UserId),
                            "privilege" => Ok(GeneratedField::Privilege),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RevokePrivilegeRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct user.RevokePrivilegeRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RevokePrivilegeRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_id = None;
                let mut privilege = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::UserId => {
                            if user_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::Privilege => {
                            if privilege.is_some() {
                                return Err(serde::de::Error::duplicate_field("privilege"));
                            }
                            privilege = Some(map.next_value()?);
                        }
                    }
                }
                Ok(RevokePrivilegeRequest {
                    user_id: user_id.unwrap_or_default(),
                    privilege,
                })
            }
        }
        deserializer.deserialize_struct("user.RevokePrivilegeRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RevokePrivilegeResponse {
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
        if self.version != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("user.RevokePrivilegeResponse", len)?;
        if let Some(v) = self.status.as_ref() {
            struct_ser.serialize_field("status", v)?;
        }
        if self.version != 0 {
            struct_ser.serialize_field("version", ToString::to_string(&self.version).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RevokePrivilegeResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "status",
            "version",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Status,
            Version,
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
                            "version" => Ok(GeneratedField::Version),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RevokePrivilegeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct user.RevokePrivilegeResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RevokePrivilegeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut status = None;
                let mut version = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Status => {
                            if status.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status = Some(map.next_value()?);
                        }
                        GeneratedField::Version => {
                            if version.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                    }
                }
                Ok(RevokePrivilegeResponse {
                    status,
                    version: version.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("user.RevokePrivilegeResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UserInfo {
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
        if self.is_supper {
            len += 1;
        }
        if self.can_create_db {
            len += 1;
        }
        if self.can_login {
            len += 1;
        }
        if self.auth_info.is_some() {
            len += 1;
        }
        if !self.privileges.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("user.UserInfo", len)?;
        if self.id != 0 {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if self.is_supper {
            struct_ser.serialize_field("isSupper", &self.is_supper)?;
        }
        if self.can_create_db {
            struct_ser.serialize_field("canCreateDb", &self.can_create_db)?;
        }
        if self.can_login {
            struct_ser.serialize_field("canLogin", &self.can_login)?;
        }
        if let Some(v) = self.auth_info.as_ref() {
            struct_ser.serialize_field("authInfo", v)?;
        }
        if !self.privileges.is_empty() {
            struct_ser.serialize_field("privileges", &self.privileges)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UserInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "name",
            "isSupper",
            "canCreateDb",
            "canLogin",
            "authInfo",
            "privileges",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            Name,
            IsSupper,
            CanCreateDb,
            CanLogin,
            AuthInfo,
            Privileges,
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
                            "isSupper" => Ok(GeneratedField::IsSupper),
                            "canCreateDb" => Ok(GeneratedField::CanCreateDb),
                            "canLogin" => Ok(GeneratedField::CanLogin),
                            "authInfo" => Ok(GeneratedField::AuthInfo),
                            "privileges" => Ok(GeneratedField::Privileges),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UserInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct user.UserInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<UserInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id = None;
                let mut name = None;
                let mut is_supper = None;
                let mut can_create_db = None;
                let mut can_login = None;
                let mut auth_info = None;
                let mut privileges = None;
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
                        GeneratedField::IsSupper => {
                            if is_supper.is_some() {
                                return Err(serde::de::Error::duplicate_field("isSupper"));
                            }
                            is_supper = Some(map.next_value()?);
                        }
                        GeneratedField::CanCreateDb => {
                            if can_create_db.is_some() {
                                return Err(serde::de::Error::duplicate_field("canCreateDb"));
                            }
                            can_create_db = Some(map.next_value()?);
                        }
                        GeneratedField::CanLogin => {
                            if can_login.is_some() {
                                return Err(serde::de::Error::duplicate_field("canLogin"));
                            }
                            can_login = Some(map.next_value()?);
                        }
                        GeneratedField::AuthInfo => {
                            if auth_info.is_some() {
                                return Err(serde::de::Error::duplicate_field("authInfo"));
                            }
                            auth_info = Some(map.next_value()?);
                        }
                        GeneratedField::Privileges => {
                            if privileges.is_some() {
                                return Err(serde::de::Error::duplicate_field("privileges"));
                            }
                            privileges = Some(map.next_value()?);
                        }
                    }
                }
                Ok(UserInfo {
                    id: id.unwrap_or_default(),
                    name: name.unwrap_or_default(),
                    is_supper: is_supper.unwrap_or_default(),
                    can_create_db: can_create_db.unwrap_or_default(),
                    can_login: can_login.unwrap_or_default(),
                    auth_info,
                    privileges: privileges.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("user.UserInfo", FIELDS, GeneratedVisitor)
    }
}
