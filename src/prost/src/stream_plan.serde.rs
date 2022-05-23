use crate::stream_plan::*;
impl serde::Serialize for ActorMapping {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.original_indices.is_empty() {
            len += 1;
        }
        if !self.data.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.ActorMapping", len)?;
        if !self.original_indices.is_empty() {
            struct_ser.serialize_field("originalIndices", &self.original_indices.iter().map(ToString::to_string).collect::<Vec<_>>())?;
        }
        if !self.data.is_empty() {
            struct_ser.serialize_field("data", &self.data)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActorMapping {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "originalIndices",
            "data",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OriginalIndices,
            Data,
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
                            "originalIndices" => Ok(GeneratedField::OriginalIndices),
                            "data" => Ok(GeneratedField::Data),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActorMapping;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.ActorMapping")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ActorMapping, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut original_indices = None;
                let mut data = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::OriginalIndices => {
                            if original_indices.is_some() {
                                return Err(serde::de::Error::duplicate_field("originalIndices"));
                            }
                            original_indices = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::Data => {
                            if data.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                    }
                }
                Ok(ActorMapping {
                    original_indices: original_indices.unwrap_or_default(),
                    data: data.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.ActorMapping", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ArrangeNode {
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
        if self.table_info.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.ArrangeNode", len)?;
        if self.table_id != 0 {
            struct_ser.serialize_field("tableId", &self.table_id)?;
        }
        if let Some(v) = self.table_info.as_ref() {
            struct_ser.serialize_field("tableInfo", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ArrangeNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tableId",
            "tableInfo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableId,
            TableInfo,
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
                            "tableInfo" => Ok(GeneratedField::TableInfo),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ArrangeNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.ArrangeNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ArrangeNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_id = None;
                let mut table_info = None;
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
                        GeneratedField::TableInfo => {
                            if table_info.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableInfo"));
                            }
                            table_info = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ArrangeNode {
                    table_id: table_id.unwrap_or_default(),
                    table_info,
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.ArrangeNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ArrangementInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.arrange_key_orders.is_empty() {
            len += 1;
        }
        if !self.column_descs.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.ArrangementInfo", len)?;
        if !self.arrange_key_orders.is_empty() {
            struct_ser.serialize_field("arrangeKeyOrders", &self.arrange_key_orders)?;
        }
        if !self.column_descs.is_empty() {
            struct_ser.serialize_field("columnDescs", &self.column_descs)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ArrangementInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "arrangeKeyOrders",
            "columnDescs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ArrangeKeyOrders,
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
                            "arrangeKeyOrders" => Ok(GeneratedField::ArrangeKeyOrders),
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
            type Value = ArrangementInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.ArrangementInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ArrangementInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut arrange_key_orders = None;
                let mut column_descs = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ArrangeKeyOrders => {
                            if arrange_key_orders.is_some() {
                                return Err(serde::de::Error::duplicate_field("arrangeKeyOrders"));
                            }
                            arrange_key_orders = Some(map.next_value()?);
                        }
                        GeneratedField::ColumnDescs => {
                            if column_descs.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnDescs"));
                            }
                            column_descs = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ArrangementInfo {
                    arrange_key_orders: arrange_key_orders.unwrap_or_default(),
                    column_descs: column_descs.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.ArrangementInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for BatchPlanNode {
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
        if !self.column_descs.is_empty() {
            len += 1;
        }
        if !self.distribution_keys.is_empty() {
            len += 1;
        }
        if self.hash_mapping.is_some() {
            len += 1;
        }
        if self.parallel_unit_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.BatchPlanNode", len)?;
        if let Some(v) = self.table_ref_id.as_ref() {
            struct_ser.serialize_field("tableRefId", v)?;
        }
        if !self.column_descs.is_empty() {
            struct_ser.serialize_field("columnDescs", &self.column_descs)?;
        }
        if !self.distribution_keys.is_empty() {
            struct_ser.serialize_field("distributionKeys", &self.distribution_keys)?;
        }
        if let Some(v) = self.hash_mapping.as_ref() {
            struct_ser.serialize_field("hashMapping", v)?;
        }
        if self.parallel_unit_id != 0 {
            struct_ser.serialize_field("parallelUnitId", &self.parallel_unit_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BatchPlanNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tableRefId",
            "columnDescs",
            "distributionKeys",
            "hashMapping",
            "parallelUnitId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableRefId,
            ColumnDescs,
            DistributionKeys,
            HashMapping,
            ParallelUnitId,
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
                            "columnDescs" => Ok(GeneratedField::ColumnDescs),
                            "distributionKeys" => Ok(GeneratedField::DistributionKeys),
                            "hashMapping" => Ok(GeneratedField::HashMapping),
                            "parallelUnitId" => Ok(GeneratedField::ParallelUnitId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BatchPlanNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.BatchPlanNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<BatchPlanNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_ref_id = None;
                let mut column_descs = None;
                let mut distribution_keys = None;
                let mut hash_mapping = None;
                let mut parallel_unit_id = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TableRefId => {
                            if table_ref_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableRefId"));
                            }
                            table_ref_id = Some(map.next_value()?);
                        }
                        GeneratedField::ColumnDescs => {
                            if column_descs.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnDescs"));
                            }
                            column_descs = Some(map.next_value()?);
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
                        GeneratedField::HashMapping => {
                            if hash_mapping.is_some() {
                                return Err(serde::de::Error::duplicate_field("hashMapping"));
                            }
                            hash_mapping = Some(map.next_value()?);
                        }
                        GeneratedField::ParallelUnitId => {
                            if parallel_unit_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("parallelUnitId"));
                            }
                            parallel_unit_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                    }
                }
                Ok(BatchPlanNode {
                    table_ref_id,
                    column_descs: column_descs.unwrap_or_default(),
                    distribution_keys: distribution_keys.unwrap_or_default(),
                    hash_mapping,
                    parallel_unit_id: parallel_unit_id.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.BatchPlanNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ChainNode {
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
        if !self.upstream_fields.is_empty() {
            len += 1;
        }
        if !self.column_ids.is_empty() {
            len += 1;
        }
        if self.disable_rearrange {
            len += 1;
        }
        if self.same_worker_node {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.ChainNode", len)?;
        if let Some(v) = self.table_ref_id.as_ref() {
            struct_ser.serialize_field("tableRefId", v)?;
        }
        if !self.upstream_fields.is_empty() {
            struct_ser.serialize_field("upstreamFields", &self.upstream_fields)?;
        }
        if !self.column_ids.is_empty() {
            struct_ser.serialize_field("columnIds", &self.column_ids)?;
        }
        if self.disable_rearrange {
            struct_ser.serialize_field("disableRearrange", &self.disable_rearrange)?;
        }
        if self.same_worker_node {
            struct_ser.serialize_field("sameWorkerNode", &self.same_worker_node)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ChainNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tableRefId",
            "upstreamFields",
            "columnIds",
            "disableRearrange",
            "sameWorkerNode",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableRefId,
            UpstreamFields,
            ColumnIds,
            DisableRearrange,
            SameWorkerNode,
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
                            "upstreamFields" => Ok(GeneratedField::UpstreamFields),
                            "columnIds" => Ok(GeneratedField::ColumnIds),
                            "disableRearrange" => Ok(GeneratedField::DisableRearrange),
                            "sameWorkerNode" => Ok(GeneratedField::SameWorkerNode),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ChainNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.ChainNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ChainNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_ref_id = None;
                let mut upstream_fields = None;
                let mut column_ids = None;
                let mut disable_rearrange = None;
                let mut same_worker_node = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TableRefId => {
                            if table_ref_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableRefId"));
                            }
                            table_ref_id = Some(map.next_value()?);
                        }
                        GeneratedField::UpstreamFields => {
                            if upstream_fields.is_some() {
                                return Err(serde::de::Error::duplicate_field("upstreamFields"));
                            }
                            upstream_fields = Some(map.next_value()?);
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
                        GeneratedField::DisableRearrange => {
                            if disable_rearrange.is_some() {
                                return Err(serde::de::Error::duplicate_field("disableRearrange"));
                            }
                            disable_rearrange = Some(map.next_value()?);
                        }
                        GeneratedField::SameWorkerNode => {
                            if same_worker_node.is_some() {
                                return Err(serde::de::Error::duplicate_field("sameWorkerNode"));
                            }
                            same_worker_node = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ChainNode {
                    table_ref_id,
                    upstream_fields: upstream_fields.unwrap_or_default(),
                    column_ids: column_ids.unwrap_or_default(),
                    disable_rearrange: disable_rearrange.unwrap_or_default(),
                    same_worker_node: same_worker_node.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.ChainNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeltaIndexJoinNode {
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
        if self.left_table_id != 0 {
            len += 1;
        }
        if self.right_table_id != 0 {
            len += 1;
        }
        if self.left_info.is_some() {
            len += 1;
        }
        if self.right_info.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.DeltaIndexJoinNode", len)?;
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
        if self.left_table_id != 0 {
            struct_ser.serialize_field("leftTableId", &self.left_table_id)?;
        }
        if self.right_table_id != 0 {
            struct_ser.serialize_field("rightTableId", &self.right_table_id)?;
        }
        if let Some(v) = self.left_info.as_ref() {
            struct_ser.serialize_field("leftInfo", v)?;
        }
        if let Some(v) = self.right_info.as_ref() {
            struct_ser.serialize_field("rightInfo", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeltaIndexJoinNode {
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
            "leftTableId",
            "rightTableId",
            "leftInfo",
            "rightInfo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            JoinType,
            LeftKey,
            RightKey,
            Condition,
            LeftTableId,
            RightTableId,
            LeftInfo,
            RightInfo,
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
                            "leftTableId" => Ok(GeneratedField::LeftTableId),
                            "rightTableId" => Ok(GeneratedField::RightTableId),
                            "leftInfo" => Ok(GeneratedField::LeftInfo),
                            "rightInfo" => Ok(GeneratedField::RightInfo),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeltaIndexJoinNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.DeltaIndexJoinNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DeltaIndexJoinNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut join_type = None;
                let mut left_key = None;
                let mut right_key = None;
                let mut condition = None;
                let mut left_table_id = None;
                let mut right_table_id = None;
                let mut left_info = None;
                let mut right_info = None;
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
                        GeneratedField::LeftTableId => {
                            if left_table_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("leftTableId"));
                            }
                            left_table_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::RightTableId => {
                            if right_table_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("rightTableId"));
                            }
                            right_table_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::LeftInfo => {
                            if left_info.is_some() {
                                return Err(serde::de::Error::duplicate_field("leftInfo"));
                            }
                            left_info = Some(map.next_value()?);
                        }
                        GeneratedField::RightInfo => {
                            if right_info.is_some() {
                                return Err(serde::de::Error::duplicate_field("rightInfo"));
                            }
                            right_info = Some(map.next_value()?);
                        }
                    }
                }
                Ok(DeltaIndexJoinNode {
                    join_type: join_type.unwrap_or_default(),
                    left_key: left_key.unwrap_or_default(),
                    right_key: right_key.unwrap_or_default(),
                    condition,
                    left_table_id: left_table_id.unwrap_or_default(),
                    right_table_id: right_table_id.unwrap_or_default(),
                    left_info,
                    right_info,
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.DeltaIndexJoinNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DispatchStrategy {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.r#type != 0 {
            len += 1;
        }
        if !self.column_indices.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.DispatchStrategy", len)?;
        if self.r#type != 0 {
            let v = DispatcherType::from_i32(self.r#type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.r#type)))?;
            struct_ser.serialize_field("type", &v)?;
        }
        if !self.column_indices.is_empty() {
            struct_ser.serialize_field("columnIndices", &self.column_indices)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DispatchStrategy {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "type",
            "columnIndices",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Type,
            ColumnIndices,
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
                            "type" => Ok(GeneratedField::Type),
                            "columnIndices" => Ok(GeneratedField::ColumnIndices),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DispatchStrategy;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.DispatchStrategy")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DispatchStrategy, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r#type = None;
                let mut column_indices = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Type => {
                            if r#type.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type = Some(map.next_value::<DispatcherType>()? as i32);
                        }
                        GeneratedField::ColumnIndices => {
                            if column_indices.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnIndices"));
                            }
                            column_indices = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                    }
                }
                Ok(DispatchStrategy {
                    r#type: r#type.unwrap_or_default(),
                    column_indices: column_indices.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.DispatchStrategy", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Dispatcher {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.r#type != 0 {
            len += 1;
        }
        if !self.column_indices.is_empty() {
            len += 1;
        }
        if self.hash_mapping.is_some() {
            len += 1;
        }
        if self.dispatcher_id != 0 {
            len += 1;
        }
        if !self.downstream_actor_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.Dispatcher", len)?;
        if self.r#type != 0 {
            let v = DispatcherType::from_i32(self.r#type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.r#type)))?;
            struct_ser.serialize_field("type", &v)?;
        }
        if !self.column_indices.is_empty() {
            struct_ser.serialize_field("columnIndices", &self.column_indices)?;
        }
        if let Some(v) = self.hash_mapping.as_ref() {
            struct_ser.serialize_field("hashMapping", v)?;
        }
        if self.dispatcher_id != 0 {
            struct_ser.serialize_field("dispatcherId", ToString::to_string(&self.dispatcher_id).as_str())?;
        }
        if !self.downstream_actor_id.is_empty() {
            struct_ser.serialize_field("downstreamActorId", &self.downstream_actor_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Dispatcher {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "type",
            "columnIndices",
            "hashMapping",
            "dispatcherId",
            "downstreamActorId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Type,
            ColumnIndices,
            HashMapping,
            DispatcherId,
            DownstreamActorId,
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
                            "type" => Ok(GeneratedField::Type),
                            "columnIndices" => Ok(GeneratedField::ColumnIndices),
                            "hashMapping" => Ok(GeneratedField::HashMapping),
                            "dispatcherId" => Ok(GeneratedField::DispatcherId),
                            "downstreamActorId" => Ok(GeneratedField::DownstreamActorId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Dispatcher;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.Dispatcher")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Dispatcher, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r#type = None;
                let mut column_indices = None;
                let mut hash_mapping = None;
                let mut dispatcher_id = None;
                let mut downstream_actor_id = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Type => {
                            if r#type.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type = Some(map.next_value::<DispatcherType>()? as i32);
                        }
                        GeneratedField::ColumnIndices => {
                            if column_indices.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnIndices"));
                            }
                            column_indices = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::HashMapping => {
                            if hash_mapping.is_some() {
                                return Err(serde::de::Error::duplicate_field("hashMapping"));
                            }
                            hash_mapping = Some(map.next_value()?);
                        }
                        GeneratedField::DispatcherId => {
                            if dispatcher_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("dispatcherId"));
                            }
                            dispatcher_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::DownstreamActorId => {
                            if downstream_actor_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("downstreamActorId"));
                            }
                            downstream_actor_id = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                    }
                }
                Ok(Dispatcher {
                    r#type: r#type.unwrap_or_default(),
                    column_indices: column_indices.unwrap_or_default(),
                    hash_mapping,
                    dispatcher_id: dispatcher_id.unwrap_or_default(),
                    downstream_actor_id: downstream_actor_id.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.Dispatcher", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DispatcherType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Invalid => "INVALID",
            Self::Hash => "HASH",
            Self::Broadcast => "BROADCAST",
            Self::Simple => "SIMPLE",
            Self::NoShuffle => "NO_SHUFFLE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for DispatcherType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "INVALID",
            "HASH",
            "BROADCAST",
            "SIMPLE",
            "NO_SHUFFLE",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DispatcherType;

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
                    .and_then(DispatcherType::from_i32)
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
                    .and_then(DispatcherType::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "INVALID" => Ok(DispatcherType::Invalid),
                    "HASH" => Ok(DispatcherType::Hash),
                    "BROADCAST" => Ok(DispatcherType::Broadcast),
                    "SIMPLE" => Ok(DispatcherType::Simple),
                    "NO_SHUFFLE" => Ok(DispatcherType::NoShuffle),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
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
        if self.strategy.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.ExchangeNode", len)?;
        if let Some(v) = self.strategy.as_ref() {
            struct_ser.serialize_field("strategy", v)?;
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
            "strategy",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Strategy,
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
                            "strategy" => Ok(GeneratedField::Strategy),
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
                formatter.write_str("struct stream_plan.ExchangeNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ExchangeNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut strategy = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Strategy => {
                            if strategy.is_some() {
                                return Err(serde::de::Error::duplicate_field("strategy"));
                            }
                            strategy = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ExchangeNode {
                    strategy,
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.ExchangeNode", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("stream_plan.FilterNode", len)?;
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
                formatter.write_str("struct stream_plan.FilterNode")
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
        deserializer.deserialize_struct("stream_plan.FilterNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FragmentType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Source => "SOURCE",
            Self::Sink => "SINK",
            Self::Others => "OTHERS",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for FragmentType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "SOURCE",
            "SINK",
            "OTHERS",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FragmentType;

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
                    .and_then(FragmentType::from_i32)
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
                    .and_then(FragmentType::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "SOURCE" => Ok(FragmentType::Source),
                    "SINK" => Ok(FragmentType::Sink),
                    "OTHERS" => Ok(FragmentType::Others),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
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
        if !self.distribution_keys.is_empty() {
            len += 1;
        }
        if !self.agg_calls.is_empty() {
            len += 1;
        }
        if !self.table_ids.is_empty() {
            len += 1;
        }
        if self.append_only {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.HashAggNode", len)?;
        if !self.distribution_keys.is_empty() {
            struct_ser.serialize_field("distributionKeys", &self.distribution_keys)?;
        }
        if !self.agg_calls.is_empty() {
            struct_ser.serialize_field("aggCalls", &self.agg_calls)?;
        }
        if !self.table_ids.is_empty() {
            struct_ser.serialize_field("tableIds", &self.table_ids)?;
        }
        if self.append_only {
            struct_ser.serialize_field("appendOnly", &self.append_only)?;
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
            "distributionKeys",
            "aggCalls",
            "tableIds",
            "appendOnly",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DistributionKeys,
            AggCalls,
            TableIds,
            AppendOnly,
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
                            "distributionKeys" => Ok(GeneratedField::DistributionKeys),
                            "aggCalls" => Ok(GeneratedField::AggCalls),
                            "tableIds" => Ok(GeneratedField::TableIds),
                            "appendOnly" => Ok(GeneratedField::AppendOnly),
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
                formatter.write_str("struct stream_plan.HashAggNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<HashAggNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut distribution_keys = None;
                let mut agg_calls = None;
                let mut table_ids = None;
                let mut append_only = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DistributionKeys => {
                            if distribution_keys.is_some() {
                                return Err(serde::de::Error::duplicate_field("distributionKeys"));
                            }
                            distribution_keys = Some(
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
                        GeneratedField::TableIds => {
                            if table_ids.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableIds"));
                            }
                            table_ids = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::AppendOnly => {
                            if append_only.is_some() {
                                return Err(serde::de::Error::duplicate_field("appendOnly"));
                            }
                            append_only = Some(map.next_value()?);
                        }
                    }
                }
                Ok(HashAggNode {
                    distribution_keys: distribution_keys.unwrap_or_default(),
                    agg_calls: agg_calls.unwrap_or_default(),
                    table_ids: table_ids.unwrap_or_default(),
                    append_only: append_only.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.HashAggNode", FIELDS, GeneratedVisitor)
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
        if !self.distribution_keys.is_empty() {
            len += 1;
        }
        if self.is_delta_join {
            len += 1;
        }
        if self.left_table_id != 0 {
            len += 1;
        }
        if self.right_table_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.HashJoinNode", len)?;
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
        if !self.distribution_keys.is_empty() {
            struct_ser.serialize_field("distributionKeys", &self.distribution_keys)?;
        }
        if self.is_delta_join {
            struct_ser.serialize_field("isDeltaJoin", &self.is_delta_join)?;
        }
        if self.left_table_id != 0 {
            struct_ser.serialize_field("leftTableId", &self.left_table_id)?;
        }
        if self.right_table_id != 0 {
            struct_ser.serialize_field("rightTableId", &self.right_table_id)?;
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
            "distributionKeys",
            "isDeltaJoin",
            "leftTableId",
            "rightTableId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            JoinType,
            LeftKey,
            RightKey,
            Condition,
            DistributionKeys,
            IsDeltaJoin,
            LeftTableId,
            RightTableId,
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
                            "distributionKeys" => Ok(GeneratedField::DistributionKeys),
                            "isDeltaJoin" => Ok(GeneratedField::IsDeltaJoin),
                            "leftTableId" => Ok(GeneratedField::LeftTableId),
                            "rightTableId" => Ok(GeneratedField::RightTableId),
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
                formatter.write_str("struct stream_plan.HashJoinNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<HashJoinNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut join_type = None;
                let mut left_key = None;
                let mut right_key = None;
                let mut condition = None;
                let mut distribution_keys = None;
                let mut is_delta_join = None;
                let mut left_table_id = None;
                let mut right_table_id = None;
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
                        GeneratedField::DistributionKeys => {
                            if distribution_keys.is_some() {
                                return Err(serde::de::Error::duplicate_field("distributionKeys"));
                            }
                            distribution_keys = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::IsDeltaJoin => {
                            if is_delta_join.is_some() {
                                return Err(serde::de::Error::duplicate_field("isDeltaJoin"));
                            }
                            is_delta_join = Some(map.next_value()?);
                        }
                        GeneratedField::LeftTableId => {
                            if left_table_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("leftTableId"));
                            }
                            left_table_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::RightTableId => {
                            if right_table_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("rightTableId"));
                            }
                            right_table_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                    }
                }
                Ok(HashJoinNode {
                    join_type: join_type.unwrap_or_default(),
                    left_key: left_key.unwrap_or_default(),
                    right_key: right_key.unwrap_or_default(),
                    condition,
                    distribution_keys: distribution_keys.unwrap_or_default(),
                    is_delta_join: is_delta_join.unwrap_or_default(),
                    left_table_id: left_table_id.unwrap_or_default(),
                    right_table_id: right_table_id.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.HashJoinNode", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("stream_plan.HopWindowNode", len)?;
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
                formatter.write_str("struct stream_plan.HopWindowNode")
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
        deserializer.deserialize_struct("stream_plan.HopWindowNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LookupNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.arrange_key.is_empty() {
            len += 1;
        }
        if !self.stream_key.is_empty() {
            len += 1;
        }
        if self.use_current_epoch {
            len += 1;
        }
        if !self.column_mapping.is_empty() {
            len += 1;
        }
        if self.arrangement_table_info.is_some() {
            len += 1;
        }
        if self.arrangement_table_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.LookupNode", len)?;
        if !self.arrange_key.is_empty() {
            struct_ser.serialize_field("arrangeKey", &self.arrange_key)?;
        }
        if !self.stream_key.is_empty() {
            struct_ser.serialize_field("streamKey", &self.stream_key)?;
        }
        if self.use_current_epoch {
            struct_ser.serialize_field("useCurrentEpoch", &self.use_current_epoch)?;
        }
        if !self.column_mapping.is_empty() {
            struct_ser.serialize_field("columnMapping", &self.column_mapping)?;
        }
        if let Some(v) = self.arrangement_table_info.as_ref() {
            struct_ser.serialize_field("arrangementTableInfo", v)?;
        }
        if let Some(v) = self.arrangement_table_id.as_ref() {
            match v {
                lookup_node::ArrangementTableId::TableId(v) => {
                    struct_ser.serialize_field("tableId", v)?;
                }
                lookup_node::ArrangementTableId::IndexId(v) => {
                    struct_ser.serialize_field("indexId", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LookupNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "arrangeKey",
            "streamKey",
            "useCurrentEpoch",
            "columnMapping",
            "arrangementTableInfo",
            "tableId",
            "indexId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ArrangeKey,
            StreamKey,
            UseCurrentEpoch,
            ColumnMapping,
            ArrangementTableInfo,
            TableId,
            IndexId,
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
                            "arrangeKey" => Ok(GeneratedField::ArrangeKey),
                            "streamKey" => Ok(GeneratedField::StreamKey),
                            "useCurrentEpoch" => Ok(GeneratedField::UseCurrentEpoch),
                            "columnMapping" => Ok(GeneratedField::ColumnMapping),
                            "arrangementTableInfo" => Ok(GeneratedField::ArrangementTableInfo),
                            "tableId" => Ok(GeneratedField::TableId),
                            "indexId" => Ok(GeneratedField::IndexId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LookupNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.LookupNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<LookupNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut arrange_key = None;
                let mut stream_key = None;
                let mut use_current_epoch = None;
                let mut column_mapping = None;
                let mut arrangement_table_info = None;
                let mut arrangement_table_id = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ArrangeKey => {
                            if arrange_key.is_some() {
                                return Err(serde::de::Error::duplicate_field("arrangeKey"));
                            }
                            arrange_key = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::StreamKey => {
                            if stream_key.is_some() {
                                return Err(serde::de::Error::duplicate_field("streamKey"));
                            }
                            stream_key = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::UseCurrentEpoch => {
                            if use_current_epoch.is_some() {
                                return Err(serde::de::Error::duplicate_field("useCurrentEpoch"));
                            }
                            use_current_epoch = Some(map.next_value()?);
                        }
                        GeneratedField::ColumnMapping => {
                            if column_mapping.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnMapping"));
                            }
                            column_mapping = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::ArrangementTableInfo => {
                            if arrangement_table_info.is_some() {
                                return Err(serde::de::Error::duplicate_field("arrangementTableInfo"));
                            }
                            arrangement_table_info = Some(map.next_value()?);
                        }
                        GeneratedField::TableId => {
                            if arrangement_table_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableId"));
                            }
                            arrangement_table_id = Some(lookup_node::ArrangementTableId::TableId(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            ));
                        }
                        GeneratedField::IndexId => {
                            if arrangement_table_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("indexId"));
                            }
                            arrangement_table_id = Some(lookup_node::ArrangementTableId::IndexId(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            ));
                        }
                    }
                }
                Ok(LookupNode {
                    arrange_key: arrange_key.unwrap_or_default(),
                    stream_key: stream_key.unwrap_or_default(),
                    use_current_epoch: use_current_epoch.unwrap_or_default(),
                    column_mapping: column_mapping.unwrap_or_default(),
                    arrangement_table_info,
                    arrangement_table_id,
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.LookupNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LookupUnionNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.order.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.LookupUnionNode", len)?;
        if !self.order.is_empty() {
            struct_ser.serialize_field("order", &self.order)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LookupUnionNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "order",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = LookupUnionNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.LookupUnionNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<LookupUnionNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut order = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Order => {
                            if order.is_some() {
                                return Err(serde::de::Error::duplicate_field("order"));
                            }
                            order = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                    }
                }
                Ok(LookupUnionNode {
                    order: order.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.LookupUnionNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MaterializeNode {
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
        if self.associated_table_ref_id.is_some() {
            len += 1;
        }
        if !self.column_orders.is_empty() {
            len += 1;
        }
        if !self.column_ids.is_empty() {
            len += 1;
        }
        if !self.distribution_keys.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.MaterializeNode", len)?;
        if let Some(v) = self.table_ref_id.as_ref() {
            struct_ser.serialize_field("tableRefId", v)?;
        }
        if let Some(v) = self.associated_table_ref_id.as_ref() {
            struct_ser.serialize_field("associatedTableRefId", v)?;
        }
        if !self.column_orders.is_empty() {
            struct_ser.serialize_field("columnOrders", &self.column_orders)?;
        }
        if !self.column_ids.is_empty() {
            struct_ser.serialize_field("columnIds", &self.column_ids)?;
        }
        if !self.distribution_keys.is_empty() {
            struct_ser.serialize_field("distributionKeys", &self.distribution_keys)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MaterializeNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tableRefId",
            "associatedTableRefId",
            "columnOrders",
            "columnIds",
            "distributionKeys",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableRefId,
            AssociatedTableRefId,
            ColumnOrders,
            ColumnIds,
            DistributionKeys,
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
                            "associatedTableRefId" => Ok(GeneratedField::AssociatedTableRefId),
                            "columnOrders" => Ok(GeneratedField::ColumnOrders),
                            "columnIds" => Ok(GeneratedField::ColumnIds),
                            "distributionKeys" => Ok(GeneratedField::DistributionKeys),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MaterializeNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.MaterializeNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MaterializeNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_ref_id = None;
                let mut associated_table_ref_id = None;
                let mut column_orders = None;
                let mut column_ids = None;
                let mut distribution_keys = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TableRefId => {
                            if table_ref_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableRefId"));
                            }
                            table_ref_id = Some(map.next_value()?);
                        }
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
                        GeneratedField::ColumnIds => {
                            if column_ids.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnIds"));
                            }
                            column_ids = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
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
                    }
                }
                Ok(MaterializeNode {
                    table_ref_id,
                    associated_table_ref_id,
                    column_orders: column_orders.unwrap_or_default(),
                    column_ids: column_ids.unwrap_or_default(),
                    distribution_keys: distribution_keys.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.MaterializeNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MergeNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.upstream_actor_id.is_empty() {
            len += 1;
        }
        if !self.fields.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.MergeNode", len)?;
        if !self.upstream_actor_id.is_empty() {
            struct_ser.serialize_field("upstreamActorId", &self.upstream_actor_id)?;
        }
        if !self.fields.is_empty() {
            struct_ser.serialize_field("fields", &self.fields)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MergeNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "upstreamActorId",
            "fields",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UpstreamActorId,
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
                            "upstreamActorId" => Ok(GeneratedField::UpstreamActorId),
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
            type Value = MergeNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.MergeNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MergeNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut upstream_actor_id = None;
                let mut fields = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::UpstreamActorId => {
                            if upstream_actor_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("upstreamActorId"));
                            }
                            upstream_actor_id = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::Fields => {
                            if fields.is_some() {
                                return Err(serde::de::Error::duplicate_field("fields"));
                            }
                            fields = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MergeNode {
                    upstream_actor_id: upstream_actor_id.unwrap_or_default(),
                    fields: fields.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.MergeNode", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("stream_plan.ProjectNode", len)?;
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
                formatter.write_str("struct stream_plan.ProjectNode")
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
        deserializer.deserialize_struct("stream_plan.ProjectNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SimpleAggNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.agg_calls.is_empty() {
            len += 1;
        }
        if !self.distribution_keys.is_empty() {
            len += 1;
        }
        if !self.table_ids.is_empty() {
            len += 1;
        }
        if self.append_only {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.SimpleAggNode", len)?;
        if !self.agg_calls.is_empty() {
            struct_ser.serialize_field("aggCalls", &self.agg_calls)?;
        }
        if !self.distribution_keys.is_empty() {
            struct_ser.serialize_field("distributionKeys", &self.distribution_keys)?;
        }
        if !self.table_ids.is_empty() {
            struct_ser.serialize_field("tableIds", &self.table_ids)?;
        }
        if self.append_only {
            struct_ser.serialize_field("appendOnly", &self.append_only)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SimpleAggNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "aggCalls",
            "distributionKeys",
            "tableIds",
            "appendOnly",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AggCalls,
            DistributionKeys,
            TableIds,
            AppendOnly,
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
                            "aggCalls" => Ok(GeneratedField::AggCalls),
                            "distributionKeys" => Ok(GeneratedField::DistributionKeys),
                            "tableIds" => Ok(GeneratedField::TableIds),
                            "appendOnly" => Ok(GeneratedField::AppendOnly),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SimpleAggNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.SimpleAggNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SimpleAggNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut agg_calls = None;
                let mut distribution_keys = None;
                let mut table_ids = None;
                let mut append_only = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::AggCalls => {
                            if agg_calls.is_some() {
                                return Err(serde::de::Error::duplicate_field("aggCalls"));
                            }
                            agg_calls = Some(map.next_value()?);
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
                        GeneratedField::TableIds => {
                            if table_ids.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableIds"));
                            }
                            table_ids = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::AppendOnly => {
                            if append_only.is_some() {
                                return Err(serde::de::Error::duplicate_field("appendOnly"));
                            }
                            append_only = Some(map.next_value()?);
                        }
                    }
                }
                Ok(SimpleAggNode {
                    agg_calls: agg_calls.unwrap_or_default(),
                    distribution_keys: distribution_keys.unwrap_or_default(),
                    table_ids: table_ids.unwrap_or_default(),
                    append_only: append_only.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.SimpleAggNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SourceNode {
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
        if self.source_type != 0 {
            len += 1;
        }
        if self.stream_source_state.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.SourceNode", len)?;
        if let Some(v) = self.table_ref_id.as_ref() {
            struct_ser.serialize_field("tableRefId", v)?;
        }
        if !self.column_ids.is_empty() {
            struct_ser.serialize_field("columnIds", &self.column_ids)?;
        }
        if self.source_type != 0 {
            let v = source_node::SourceType::from_i32(self.source_type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.source_type)))?;
            struct_ser.serialize_field("sourceType", &v)?;
        }
        if let Some(v) = self.stream_source_state.as_ref() {
            struct_ser.serialize_field("streamSourceState", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SourceNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tableRefId",
            "columnIds",
            "sourceType",
            "streamSourceState",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableRefId,
            ColumnIds,
            SourceType,
            StreamSourceState,
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
                            "sourceType" => Ok(GeneratedField::SourceType),
                            "streamSourceState" => Ok(GeneratedField::StreamSourceState),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SourceNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.SourceNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SourceNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_ref_id = None;
                let mut column_ids = None;
                let mut source_type = None;
                let mut stream_source_state = None;
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
                        GeneratedField::SourceType => {
                            if source_type.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourceType"));
                            }
                            source_type = Some(map.next_value::<source_node::SourceType>()? as i32);
                        }
                        GeneratedField::StreamSourceState => {
                            if stream_source_state.is_some() {
                                return Err(serde::de::Error::duplicate_field("streamSourceState"));
                            }
                            stream_source_state = Some(map.next_value()?);
                        }
                    }
                }
                Ok(SourceNode {
                    table_ref_id,
                    column_ids: column_ids.unwrap_or_default(),
                    source_type: source_type.unwrap_or_default(),
                    stream_source_state,
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.SourceNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for source_node::SourceType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Table => "TABLE",
            Self::Source => "SOURCE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for source_node::SourceType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "TABLE",
            "SOURCE",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = source_node::SourceType;

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
                    .and_then(source_node::SourceType::from_i32)
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
                    .and_then(source_node::SourceType::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "TABLE" => Ok(source_node::SourceType::Table),
                    "SOURCE" => Ok(source_node::SourceType::Source),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for StreamActor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.actor_id != 0 {
            len += 1;
        }
        if self.fragment_id != 0 {
            len += 1;
        }
        if self.nodes.is_some() {
            len += 1;
        }
        if !self.dispatcher.is_empty() {
            len += 1;
        }
        if !self.upstream_actor_id.is_empty() {
            len += 1;
        }
        if self.same_worker_node_as_upstream {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.StreamActor", len)?;
        if self.actor_id != 0 {
            struct_ser.serialize_field("actorId", &self.actor_id)?;
        }
        if self.fragment_id != 0 {
            struct_ser.serialize_field("fragmentId", &self.fragment_id)?;
        }
        if let Some(v) = self.nodes.as_ref() {
            struct_ser.serialize_field("nodes", v)?;
        }
        if !self.dispatcher.is_empty() {
            struct_ser.serialize_field("dispatcher", &self.dispatcher)?;
        }
        if !self.upstream_actor_id.is_empty() {
            struct_ser.serialize_field("upstreamActorId", &self.upstream_actor_id)?;
        }
        if self.same_worker_node_as_upstream {
            struct_ser.serialize_field("sameWorkerNodeAsUpstream", &self.same_worker_node_as_upstream)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StreamActor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "actorId",
            "fragmentId",
            "nodes",
            "dispatcher",
            "upstreamActorId",
            "sameWorkerNodeAsUpstream",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ActorId,
            FragmentId,
            Nodes,
            Dispatcher,
            UpstreamActorId,
            SameWorkerNodeAsUpstream,
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
                            "actorId" => Ok(GeneratedField::ActorId),
                            "fragmentId" => Ok(GeneratedField::FragmentId),
                            "nodes" => Ok(GeneratedField::Nodes),
                            "dispatcher" => Ok(GeneratedField::Dispatcher),
                            "upstreamActorId" => Ok(GeneratedField::UpstreamActorId),
                            "sameWorkerNodeAsUpstream" => Ok(GeneratedField::SameWorkerNodeAsUpstream),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StreamActor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.StreamActor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StreamActor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut actor_id = None;
                let mut fragment_id = None;
                let mut nodes = None;
                let mut dispatcher = None;
                let mut upstream_actor_id = None;
                let mut same_worker_node_as_upstream = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ActorId => {
                            if actor_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("actorId"));
                            }
                            actor_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::FragmentId => {
                            if fragment_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("fragmentId"));
                            }
                            fragment_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::Nodes => {
                            if nodes.is_some() {
                                return Err(serde::de::Error::duplicate_field("nodes"));
                            }
                            nodes = Some(map.next_value()?);
                        }
                        GeneratedField::Dispatcher => {
                            if dispatcher.is_some() {
                                return Err(serde::de::Error::duplicate_field("dispatcher"));
                            }
                            dispatcher = Some(map.next_value()?);
                        }
                        GeneratedField::UpstreamActorId => {
                            if upstream_actor_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("upstreamActorId"));
                            }
                            upstream_actor_id = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::SameWorkerNodeAsUpstream => {
                            if same_worker_node_as_upstream.is_some() {
                                return Err(serde::de::Error::duplicate_field("sameWorkerNodeAsUpstream"));
                            }
                            same_worker_node_as_upstream = Some(map.next_value()?);
                        }
                    }
                }
                Ok(StreamActor {
                    actor_id: actor_id.unwrap_or_default(),
                    fragment_id: fragment_id.unwrap_or_default(),
                    nodes,
                    dispatcher: dispatcher.unwrap_or_default(),
                    upstream_actor_id: upstream_actor_id.unwrap_or_default(),
                    same_worker_node_as_upstream: same_worker_node_as_upstream.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.StreamActor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StreamFragmentGraph {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.fragments.is_empty() {
            len += 1;
        }
        if !self.edges.is_empty() {
            len += 1;
        }
        if !self.dependent_table_ids.is_empty() {
            len += 1;
        }
        if self.table_ids_cnt != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.StreamFragmentGraph", len)?;
        if !self.fragments.is_empty() {
            struct_ser.serialize_field("fragments", &self.fragments)?;
        }
        if !self.edges.is_empty() {
            struct_ser.serialize_field("edges", &self.edges)?;
        }
        if !self.dependent_table_ids.is_empty() {
            struct_ser.serialize_field("dependentTableIds", &self.dependent_table_ids)?;
        }
        if self.table_ids_cnt != 0 {
            struct_ser.serialize_field("tableIdsCnt", &self.table_ids_cnt)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StreamFragmentGraph {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fragments",
            "edges",
            "dependentTableIds",
            "tableIdsCnt",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Fragments,
            Edges,
            DependentTableIds,
            TableIdsCnt,
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
                            "fragments" => Ok(GeneratedField::Fragments),
                            "edges" => Ok(GeneratedField::Edges),
                            "dependentTableIds" => Ok(GeneratedField::DependentTableIds),
                            "tableIdsCnt" => Ok(GeneratedField::TableIdsCnt),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StreamFragmentGraph;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.StreamFragmentGraph")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StreamFragmentGraph, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fragments = None;
                let mut edges = None;
                let mut dependent_table_ids = None;
                let mut table_ids_cnt = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Fragments => {
                            if fragments.is_some() {
                                return Err(serde::de::Error::duplicate_field("fragments"));
                            }
                            fragments = Some(
                                map.next_value::<std::collections::HashMap<::pbjson::private::NumberDeserialize<u32>, _>>()?
                                    .into_iter().map(|(k,v)| (k.0, v)).collect()
                            );
                        }
                        GeneratedField::Edges => {
                            if edges.is_some() {
                                return Err(serde::de::Error::duplicate_field("edges"));
                            }
                            edges = Some(map.next_value()?);
                        }
                        GeneratedField::DependentTableIds => {
                            if dependent_table_ids.is_some() {
                                return Err(serde::de::Error::duplicate_field("dependentTableIds"));
                            }
                            dependent_table_ids = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                        GeneratedField::TableIdsCnt => {
                            if table_ids_cnt.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableIdsCnt"));
                            }
                            table_ids_cnt = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                    }
                }
                Ok(StreamFragmentGraph {
                    fragments: fragments.unwrap_or_default(),
                    edges: edges.unwrap_or_default(),
                    dependent_table_ids: dependent_table_ids.unwrap_or_default(),
                    table_ids_cnt: table_ids_cnt.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.StreamFragmentGraph", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for stream_fragment_graph::StreamFragment {
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
        if self.node.is_some() {
            len += 1;
        }
        if self.fragment_type != 0 {
            len += 1;
        }
        if self.is_singleton {
            len += 1;
        }
        if self.table_ids_cnt != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.StreamFragmentGraph.StreamFragment", len)?;
        if self.fragment_id != 0 {
            struct_ser.serialize_field("fragmentId", &self.fragment_id)?;
        }
        if let Some(v) = self.node.as_ref() {
            struct_ser.serialize_field("node", v)?;
        }
        if self.fragment_type != 0 {
            let v = FragmentType::from_i32(self.fragment_type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.fragment_type)))?;
            struct_ser.serialize_field("fragmentType", &v)?;
        }
        if self.is_singleton {
            struct_ser.serialize_field("isSingleton", &self.is_singleton)?;
        }
        if self.table_ids_cnt != 0 {
            struct_ser.serialize_field("tableIdsCnt", &self.table_ids_cnt)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for stream_fragment_graph::StreamFragment {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fragmentId",
            "node",
            "fragmentType",
            "isSingleton",
            "tableIdsCnt",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FragmentId,
            Node,
            FragmentType,
            IsSingleton,
            TableIdsCnt,
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
                            "node" => Ok(GeneratedField::Node),
                            "fragmentType" => Ok(GeneratedField::FragmentType),
                            "isSingleton" => Ok(GeneratedField::IsSingleton),
                            "tableIdsCnt" => Ok(GeneratedField::TableIdsCnt),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = stream_fragment_graph::StreamFragment;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.StreamFragmentGraph.StreamFragment")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<stream_fragment_graph::StreamFragment, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fragment_id = None;
                let mut node = None;
                let mut fragment_type = None;
                let mut is_singleton = None;
                let mut table_ids_cnt = None;
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
                        GeneratedField::Node => {
                            if node.is_some() {
                                return Err(serde::de::Error::duplicate_field("node"));
                            }
                            node = Some(map.next_value()?);
                        }
                        GeneratedField::FragmentType => {
                            if fragment_type.is_some() {
                                return Err(serde::de::Error::duplicate_field("fragmentType"));
                            }
                            fragment_type = Some(map.next_value::<FragmentType>()? as i32);
                        }
                        GeneratedField::IsSingleton => {
                            if is_singleton.is_some() {
                                return Err(serde::de::Error::duplicate_field("isSingleton"));
                            }
                            is_singleton = Some(map.next_value()?);
                        }
                        GeneratedField::TableIdsCnt => {
                            if table_ids_cnt.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableIdsCnt"));
                            }
                            table_ids_cnt = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                    }
                }
                Ok(stream_fragment_graph::StreamFragment {
                    fragment_id: fragment_id.unwrap_or_default(),
                    node,
                    fragment_type: fragment_type.unwrap_or_default(),
                    is_singleton: is_singleton.unwrap_or_default(),
                    table_ids_cnt: table_ids_cnt.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.StreamFragmentGraph.StreamFragment", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for stream_fragment_graph::StreamFragmentEdge {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.dispatch_strategy.is_some() {
            len += 1;
        }
        if self.same_worker_node {
            len += 1;
        }
        if self.link_id != 0 {
            len += 1;
        }
        if self.upstream_id != 0 {
            len += 1;
        }
        if self.downstream_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.StreamFragmentGraph.StreamFragmentEdge", len)?;
        if let Some(v) = self.dispatch_strategy.as_ref() {
            struct_ser.serialize_field("dispatchStrategy", v)?;
        }
        if self.same_worker_node {
            struct_ser.serialize_field("sameWorkerNode", &self.same_worker_node)?;
        }
        if self.link_id != 0 {
            struct_ser.serialize_field("linkId", ToString::to_string(&self.link_id).as_str())?;
        }
        if self.upstream_id != 0 {
            struct_ser.serialize_field("upstreamId", &self.upstream_id)?;
        }
        if self.downstream_id != 0 {
            struct_ser.serialize_field("downstreamId", &self.downstream_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for stream_fragment_graph::StreamFragmentEdge {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "dispatchStrategy",
            "sameWorkerNode",
            "linkId",
            "upstreamId",
            "downstreamId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DispatchStrategy,
            SameWorkerNode,
            LinkId,
            UpstreamId,
            DownstreamId,
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
                            "dispatchStrategy" => Ok(GeneratedField::DispatchStrategy),
                            "sameWorkerNode" => Ok(GeneratedField::SameWorkerNode),
                            "linkId" => Ok(GeneratedField::LinkId),
                            "upstreamId" => Ok(GeneratedField::UpstreamId),
                            "downstreamId" => Ok(GeneratedField::DownstreamId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = stream_fragment_graph::StreamFragmentEdge;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.StreamFragmentGraph.StreamFragmentEdge")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<stream_fragment_graph::StreamFragmentEdge, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut dispatch_strategy = None;
                let mut same_worker_node = None;
                let mut link_id = None;
                let mut upstream_id = None;
                let mut downstream_id = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DispatchStrategy => {
                            if dispatch_strategy.is_some() {
                                return Err(serde::de::Error::duplicate_field("dispatchStrategy"));
                            }
                            dispatch_strategy = Some(map.next_value()?);
                        }
                        GeneratedField::SameWorkerNode => {
                            if same_worker_node.is_some() {
                                return Err(serde::de::Error::duplicate_field("sameWorkerNode"));
                            }
                            same_worker_node = Some(map.next_value()?);
                        }
                        GeneratedField::LinkId => {
                            if link_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("linkId"));
                            }
                            link_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::UpstreamId => {
                            if upstream_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("upstreamId"));
                            }
                            upstream_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::DownstreamId => {
                            if downstream_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("downstreamId"));
                            }
                            downstream_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                    }
                }
                Ok(stream_fragment_graph::StreamFragmentEdge {
                    dispatch_strategy,
                    same_worker_node: same_worker_node.unwrap_or_default(),
                    link_id: link_id.unwrap_or_default(),
                    upstream_id: upstream_id.unwrap_or_default(),
                    downstream_id: downstream_id.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.StreamFragmentGraph.StreamFragmentEdge", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StreamNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.operator_id != 0 {
            len += 1;
        }
        if !self.input.is_empty() {
            len += 1;
        }
        if !self.pk_indices.is_empty() {
            len += 1;
        }
        if self.append_only {
            len += 1;
        }
        if !self.identity.is_empty() {
            len += 1;
        }
        if !self.fields.is_empty() {
            len += 1;
        }
        if self.node_body.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.StreamNode", len)?;
        if self.operator_id != 0 {
            struct_ser.serialize_field("operatorId", ToString::to_string(&self.operator_id).as_str())?;
        }
        if !self.input.is_empty() {
            struct_ser.serialize_field("input", &self.input)?;
        }
        if !self.pk_indices.is_empty() {
            struct_ser.serialize_field("pkIndices", &self.pk_indices)?;
        }
        if self.append_only {
            struct_ser.serialize_field("appendOnly", &self.append_only)?;
        }
        if !self.identity.is_empty() {
            struct_ser.serialize_field("identity", &self.identity)?;
        }
        if !self.fields.is_empty() {
            struct_ser.serialize_field("fields", &self.fields)?;
        }
        if let Some(v) = self.node_body.as_ref() {
            match v {
                stream_node::NodeBody::Source(v) => {
                    struct_ser.serialize_field("source", v)?;
                }
                stream_node::NodeBody::Project(v) => {
                    struct_ser.serialize_field("project", v)?;
                }
                stream_node::NodeBody::Filter(v) => {
                    struct_ser.serialize_field("filter", v)?;
                }
                stream_node::NodeBody::Materialize(v) => {
                    struct_ser.serialize_field("materialize", v)?;
                }
                stream_node::NodeBody::LocalSimpleAgg(v) => {
                    struct_ser.serialize_field("localSimpleAgg", v)?;
                }
                stream_node::NodeBody::GlobalSimpleAgg(v) => {
                    struct_ser.serialize_field("globalSimpleAgg", v)?;
                }
                stream_node::NodeBody::HashAgg(v) => {
                    struct_ser.serialize_field("hashAgg", v)?;
                }
                stream_node::NodeBody::AppendOnlyTopN(v) => {
                    struct_ser.serialize_field("appendOnlyTopN", v)?;
                }
                stream_node::NodeBody::HashJoin(v) => {
                    struct_ser.serialize_field("hashJoin", v)?;
                }
                stream_node::NodeBody::TopN(v) => {
                    struct_ser.serialize_field("topN", v)?;
                }
                stream_node::NodeBody::HopWindow(v) => {
                    struct_ser.serialize_field("hopWindow", v)?;
                }
                stream_node::NodeBody::Merge(v) => {
                    struct_ser.serialize_field("merge", v)?;
                }
                stream_node::NodeBody::Exchange(v) => {
                    struct_ser.serialize_field("exchange", v)?;
                }
                stream_node::NodeBody::Chain(v) => {
                    struct_ser.serialize_field("chain", v)?;
                }
                stream_node::NodeBody::BatchPlan(v) => {
                    struct_ser.serialize_field("batchPlan", v)?;
                }
                stream_node::NodeBody::Lookup(v) => {
                    struct_ser.serialize_field("lookup", v)?;
                }
                stream_node::NodeBody::Arrange(v) => {
                    struct_ser.serialize_field("arrange", v)?;
                }
                stream_node::NodeBody::LookupUnion(v) => {
                    struct_ser.serialize_field("lookupUnion", v)?;
                }
                stream_node::NodeBody::Union(v) => {
                    struct_ser.serialize_field("union", v)?;
                }
                stream_node::NodeBody::DeltaIndexJoin(v) => {
                    struct_ser.serialize_field("deltaIndexJoin", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StreamNode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "operatorId",
            "input",
            "pkIndices",
            "appendOnly",
            "identity",
            "fields",
            "source",
            "project",
            "filter",
            "materialize",
            "localSimpleAgg",
            "globalSimpleAgg",
            "hashAgg",
            "appendOnlyTopN",
            "hashJoin",
            "topN",
            "hopWindow",
            "merge",
            "exchange",
            "chain",
            "batchPlan",
            "lookup",
            "arrange",
            "lookupUnion",
            "union",
            "deltaIndexJoin",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OperatorId,
            Input,
            PkIndices,
            AppendOnly,
            Identity,
            Fields,
            Source,
            Project,
            Filter,
            Materialize,
            LocalSimpleAgg,
            GlobalSimpleAgg,
            HashAgg,
            AppendOnlyTopN,
            HashJoin,
            TopN,
            HopWindow,
            Merge,
            Exchange,
            Chain,
            BatchPlan,
            Lookup,
            Arrange,
            LookupUnion,
            Union,
            DeltaIndexJoin,
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
                            "operatorId" => Ok(GeneratedField::OperatorId),
                            "input" => Ok(GeneratedField::Input),
                            "pkIndices" => Ok(GeneratedField::PkIndices),
                            "appendOnly" => Ok(GeneratedField::AppendOnly),
                            "identity" => Ok(GeneratedField::Identity),
                            "fields" => Ok(GeneratedField::Fields),
                            "source" => Ok(GeneratedField::Source),
                            "project" => Ok(GeneratedField::Project),
                            "filter" => Ok(GeneratedField::Filter),
                            "materialize" => Ok(GeneratedField::Materialize),
                            "localSimpleAgg" => Ok(GeneratedField::LocalSimpleAgg),
                            "globalSimpleAgg" => Ok(GeneratedField::GlobalSimpleAgg),
                            "hashAgg" => Ok(GeneratedField::HashAgg),
                            "appendOnlyTopN" => Ok(GeneratedField::AppendOnlyTopN),
                            "hashJoin" => Ok(GeneratedField::HashJoin),
                            "topN" => Ok(GeneratedField::TopN),
                            "hopWindow" => Ok(GeneratedField::HopWindow),
                            "merge" => Ok(GeneratedField::Merge),
                            "exchange" => Ok(GeneratedField::Exchange),
                            "chain" => Ok(GeneratedField::Chain),
                            "batchPlan" => Ok(GeneratedField::BatchPlan),
                            "lookup" => Ok(GeneratedField::Lookup),
                            "arrange" => Ok(GeneratedField::Arrange),
                            "lookupUnion" => Ok(GeneratedField::LookupUnion),
                            "union" => Ok(GeneratedField::Union),
                            "deltaIndexJoin" => Ok(GeneratedField::DeltaIndexJoin),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StreamNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.StreamNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StreamNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut operator_id = None;
                let mut input = None;
                let mut pk_indices = None;
                let mut append_only = None;
                let mut identity = None;
                let mut fields = None;
                let mut node_body = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::OperatorId => {
                            if operator_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("operatorId"));
                            }
                            operator_id = Some(
                                map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0
                            );
                        }
                        GeneratedField::Input => {
                            if input.is_some() {
                                return Err(serde::de::Error::duplicate_field("input"));
                            }
                            input = Some(map.next_value()?);
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
                        GeneratedField::AppendOnly => {
                            if append_only.is_some() {
                                return Err(serde::de::Error::duplicate_field("appendOnly"));
                            }
                            append_only = Some(map.next_value()?);
                        }
                        GeneratedField::Identity => {
                            if identity.is_some() {
                                return Err(serde::de::Error::duplicate_field("identity"));
                            }
                            identity = Some(map.next_value()?);
                        }
                        GeneratedField::Fields => {
                            if fields.is_some() {
                                return Err(serde::de::Error::duplicate_field("fields"));
                            }
                            fields = Some(map.next_value()?);
                        }
                        GeneratedField::Source => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("source"));
                            }
                            node_body = Some(stream_node::NodeBody::Source(map.next_value()?));
                        }
                        GeneratedField::Project => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("project"));
                            }
                            node_body = Some(stream_node::NodeBody::Project(map.next_value()?));
                        }
                        GeneratedField::Filter => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("filter"));
                            }
                            node_body = Some(stream_node::NodeBody::Filter(map.next_value()?));
                        }
                        GeneratedField::Materialize => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("materialize"));
                            }
                            node_body = Some(stream_node::NodeBody::Materialize(map.next_value()?));
                        }
                        GeneratedField::LocalSimpleAgg => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("localSimpleAgg"));
                            }
                            node_body = Some(stream_node::NodeBody::LocalSimpleAgg(map.next_value()?));
                        }
                        GeneratedField::GlobalSimpleAgg => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("globalSimpleAgg"));
                            }
                            node_body = Some(stream_node::NodeBody::GlobalSimpleAgg(map.next_value()?));
                        }
                        GeneratedField::HashAgg => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("hashAgg"));
                            }
                            node_body = Some(stream_node::NodeBody::HashAgg(map.next_value()?));
                        }
                        GeneratedField::AppendOnlyTopN => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("appendOnlyTopN"));
                            }
                            node_body = Some(stream_node::NodeBody::AppendOnlyTopN(map.next_value()?));
                        }
                        GeneratedField::HashJoin => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("hashJoin"));
                            }
                            node_body = Some(stream_node::NodeBody::HashJoin(map.next_value()?));
                        }
                        GeneratedField::TopN => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("topN"));
                            }
                            node_body = Some(stream_node::NodeBody::TopN(map.next_value()?));
                        }
                        GeneratedField::HopWindow => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("hopWindow"));
                            }
                            node_body = Some(stream_node::NodeBody::HopWindow(map.next_value()?));
                        }
                        GeneratedField::Merge => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("merge"));
                            }
                            node_body = Some(stream_node::NodeBody::Merge(map.next_value()?));
                        }
                        GeneratedField::Exchange => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("exchange"));
                            }
                            node_body = Some(stream_node::NodeBody::Exchange(map.next_value()?));
                        }
                        GeneratedField::Chain => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("chain"));
                            }
                            node_body = Some(stream_node::NodeBody::Chain(map.next_value()?));
                        }
                        GeneratedField::BatchPlan => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("batchPlan"));
                            }
                            node_body = Some(stream_node::NodeBody::BatchPlan(map.next_value()?));
                        }
                        GeneratedField::Lookup => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("lookup"));
                            }
                            node_body = Some(stream_node::NodeBody::Lookup(map.next_value()?));
                        }
                        GeneratedField::Arrange => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("arrange"));
                            }
                            node_body = Some(stream_node::NodeBody::Arrange(map.next_value()?));
                        }
                        GeneratedField::LookupUnion => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("lookupUnion"));
                            }
                            node_body = Some(stream_node::NodeBody::LookupUnion(map.next_value()?));
                        }
                        GeneratedField::Union => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("union"));
                            }
                            node_body = Some(stream_node::NodeBody::Union(map.next_value()?));
                        }
                        GeneratedField::DeltaIndexJoin => {
                            if node_body.is_some() {
                                return Err(serde::de::Error::duplicate_field("deltaIndexJoin"));
                            }
                            node_body = Some(stream_node::NodeBody::DeltaIndexJoin(map.next_value()?));
                        }
                    }
                }
                Ok(StreamNode {
                    operator_id: operator_id.unwrap_or_default(),
                    input: input.unwrap_or_default(),
                    pk_indices: pk_indices.unwrap_or_default(),
                    append_only: append_only.unwrap_or_default(),
                    identity: identity.unwrap_or_default(),
                    fields: fields.unwrap_or_default(),
                    node_body,
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.StreamNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StreamSourceState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.split_type.is_empty() {
            len += 1;
        }
        if !self.stream_source_splits.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.StreamSourceState", len)?;
        if !self.split_type.is_empty() {
            struct_ser.serialize_field("splitType", &self.split_type)?;
        }
        if !self.stream_source_splits.is_empty() {
            struct_ser.serialize_field("streamSourceSplits", &self.stream_source_splits.iter().map(pbjson::private::base64::encode).collect::<Vec<_>>())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StreamSourceState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "splitType",
            "streamSourceSplits",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SplitType,
            StreamSourceSplits,
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
                            "splitType" => Ok(GeneratedField::SplitType),
                            "streamSourceSplits" => Ok(GeneratedField::StreamSourceSplits),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StreamSourceState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.StreamSourceState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StreamSourceState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut split_type = None;
                let mut stream_source_splits = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SplitType => {
                            if split_type.is_some() {
                                return Err(serde::de::Error::duplicate_field("splitType"));
                            }
                            split_type = Some(map.next_value()?);
                        }
                        GeneratedField::StreamSourceSplits => {
                            if stream_source_splits.is_some() {
                                return Err(serde::de::Error::duplicate_field("streamSourceSplits"));
                            }
                            stream_source_splits = Some(
                                map.next_value::<Vec<::pbjson::private::BytesDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                    }
                }
                Ok(StreamSourceState {
                    split_type: split_type.unwrap_or_default(),
                    stream_source_splits: stream_source_splits.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.StreamSourceState", FIELDS, GeneratedVisitor)
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
        if !self.distribution_keys.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("stream_plan.TopNNode", len)?;
        if !self.column_orders.is_empty() {
            struct_ser.serialize_field("columnOrders", &self.column_orders)?;
        }
        if self.limit != 0 {
            struct_ser.serialize_field("limit", ToString::to_string(&self.limit).as_str())?;
        }
        if self.offset != 0 {
            struct_ser.serialize_field("offset", ToString::to_string(&self.offset).as_str())?;
        }
        if !self.distribution_keys.is_empty() {
            struct_ser.serialize_field("distributionKeys", &self.distribution_keys)?;
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
            "distributionKeys",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ColumnOrders,
            Limit,
            Offset,
            DistributionKeys,
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
                            "distributionKeys" => Ok(GeneratedField::DistributionKeys),
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
                formatter.write_str("struct stream_plan.TopNNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TopNNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut column_orders = None;
                let mut limit = None;
                let mut offset = None;
                let mut distribution_keys = None;
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
                        GeneratedField::DistributionKeys => {
                            if distribution_keys.is_some() {
                                return Err(serde::de::Error::duplicate_field("distributionKeys"));
                            }
                            distribution_keys = Some(
                                map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect()
                            );
                        }
                    }
                }
                Ok(TopNNode {
                    column_orders: column_orders.unwrap_or_default(),
                    limit: limit.unwrap_or_default(),
                    offset: offset.unwrap_or_default(),
                    distribution_keys: distribution_keys.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.TopNNode", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UnionNode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("stream_plan.UnionNode", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UnionNode {
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
            type Value = UnionNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct stream_plan.UnionNode")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<UnionNode, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(UnionNode {
                })
            }
        }
        deserializer.deserialize_struct("stream_plan.UnionNode", FIELDS, GeneratedVisitor)
    }
}
