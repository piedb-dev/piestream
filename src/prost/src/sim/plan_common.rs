/// Field is a column in the streaming or batch plan.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Field {
    #[prost(message, optional, tag="1")]
    pub data_type: ::core::option::Option<super::data::DataType>,
    #[prost(string, tag="2")]
    pub name: ::prost::alloc::string::String,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DatabaseRefId {
    #[prost(int32, tag="1")]
    pub database_id: i32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaRefId {
    #[prost(message, optional, tag="1")]
    pub database_ref_id: ::core::option::Option<DatabaseRefId>,
    #[prost(int32, tag="2")]
    pub schema_id: i32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TableRefId {
    #[prost(message, optional, tag="1")]
    pub schema_ref_id: ::core::option::Option<SchemaRefId>,
    #[prost(int32, tag="2")]
    pub table_id: i32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ColumnDesc {
    #[prost(message, optional, tag="1")]
    pub column_type: ::core::option::Option<super::data::DataType>,
    #[prost(int32, tag="2")]
    pub column_id: i32,
    /// we store the column name in column desc now just for debug, but in future we should store it in ColumnCatalog but not here
    #[prost(string, tag="3")]
    pub name: ::prost::alloc::string::String,
    /// For STRUCT type.
    #[prost(message, repeated, tag="4")]
    pub field_descs: ::prost::alloc::vec::Vec<ColumnDesc>,
    /// The user-defined type's name. Empty if the column type is a builtin type.
    /// For example, when the type is created from a protobuf schema file,
    /// this field will store the message name.
    #[prost(string, tag="5")]
    pub type_name: ::prost::alloc::string::String,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrderedColumnDesc {
    #[prost(message, optional, tag="1")]
    pub column_desc: ::core::option::Option<ColumnDesc>,
    #[prost(enumeration="OrderType", tag="2")]
    pub order: i32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ColumnCatalog {
    #[prost(message, optional, tag="1")]
    pub column_desc: ::core::option::Option<ColumnDesc>,
    #[prost(bool, tag="2")]
    pub is_hidden: bool,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CellBasedTableDesc {
    #[prost(uint32, tag="1")]
    pub table_id: u32,
    #[prost(message, repeated, tag="2")]
    pub pk: ::prost::alloc::vec::Vec<OrderedColumnDesc>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ColumnOrder {
    /// maybe other name
    #[prost(enumeration="OrderType", tag="1")]
    pub order_type: i32,
    #[prost(message, optional, tag="2")]
    pub input_ref: ::core::option::Option<super::expr::InputRefExpr>,
    #[prost(message, optional, tag="3")]
    pub return_type: ::core::option::Option<super::data::DataType>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StreamSourceInfo {
    #[prost(bool, tag="1")]
    pub append_only: bool,
    #[prost(map="string, string", tag="2")]
    pub properties: ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    #[prost(enumeration="RowFormatType", tag="3")]
    pub row_format: i32,
    #[prost(string, tag="4")]
    pub row_schema_location: ::prost::alloc::string::String,
    #[prost(int32, tag="5")]
    pub row_id_index: i32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TableSourceInfo {
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MaterializedViewInfo {
    #[prost(message, optional, tag="1")]
    pub associated_table_ref_id: ::core::option::Option<TableRefId>,
    #[prost(message, repeated, tag="2")]
    pub column_orders: ::prost::alloc::vec::Vec<ColumnOrder>,
    #[prost(int32, repeated, tag="3")]
    pub pk_indices: ::prost::alloc::vec::Vec<i32>,
    #[prost(message, repeated, tag="4")]
    pub dependent_tables: ::prost::alloc::vec::Vec<TableRefId>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum JoinType {
    /// Note that it comes from Calcite's JoinRelType.
    /// DO NOT HAVE direction for SEMI and ANTI now.
    Inner = 0,
    LeftOuter = 1,
    RightOuter = 2,
    FullOuter = 3,
    LeftSemi = 4,
    LeftAnti = 5,
    RightSemi = 6,
    RightAnti = 7,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum OrderType {
    Invalid = 0,
    Ascending = 1,
    Descending = 2,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum RowFormatType {
    Json = 0,
    Protobuf = 1,
    DebeziumJson = 2,
    Avro = 3,
}
