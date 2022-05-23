#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StreamSourceInfo {
    #[prost(map="string, string", tag="1")]
    pub properties: ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    #[prost(enumeration="super::plan_common::RowFormatType", tag="2")]
    pub row_format: i32,
    #[prost(string, tag="3")]
    pub row_schema_location: ::prost::alloc::string::String,
    #[prost(int32, tag="4")]
    pub row_id_index: i32,
    #[prost(message, repeated, tag="5")]
    pub columns: ::prost::alloc::vec::Vec<super::plan_common::ColumnCatalog>,
    #[prost(int32, repeated, tag="6")]
    pub pk_column_ids: ::prost::alloc::vec::Vec<i32>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TableSourceInfo {
    #[prost(message, repeated, tag="1")]
    pub columns: ::prost::alloc::vec::Vec<super::plan_common::ColumnCatalog>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Source {
    #[prost(uint32, tag="1")]
    pub id: u32,
    #[prost(uint32, tag="2")]
    pub schema_id: u32,
    #[prost(uint32, tag="3")]
    pub database_id: u32,
    #[prost(string, tag="4")]
    pub name: ::prost::alloc::string::String,
    #[prost(oneof="source::Info", tags="5, 6")]
    pub info: ::core::option::Option<source::Info>,
}
/// Nested message and enum types in `Source`.
pub mod source {
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Info {
        #[prost(message, tag="5")]
        StreamSource(super::StreamSourceInfo),
        #[prost(message, tag="6")]
        TableSource(super::TableSourceInfo),
    }
}
/// VirtualTable defines a view in system catalogs, it can only be queried and not be treated as a source.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VirtualTable {
    #[prost(uint32, tag="1")]
    pub id: u32,
    #[prost(string, tag="2")]
    pub name: ::prost::alloc::string::String,
    #[prost(message, repeated, tag="3")]
    pub columns: ::prost::alloc::vec::Vec<super::plan_common::ColumnCatalog>,
}
//// See `TableCatalog` struct in frontend crate for more information.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Table {
    #[prost(uint32, tag="1")]
    pub id: u32,
    #[prost(uint32, tag="2")]
    pub schema_id: u32,
    #[prost(uint32, tag="3")]
    pub database_id: u32,
    #[prost(string, tag="4")]
    pub name: ::prost::alloc::string::String,
    #[prost(message, repeated, tag="5")]
    pub columns: ::prost::alloc::vec::Vec<super::plan_common::ColumnCatalog>,
    #[prost(int32, repeated, tag="6")]
    pub order_column_ids: ::prost::alloc::vec::Vec<i32>,
    #[prost(enumeration="super::plan_common::OrderType", repeated, tag="7")]
    pub orders: ::prost::alloc::vec::Vec<i32>,
    #[prost(uint32, repeated, tag="8")]
    pub dependent_relations: ::prost::alloc::vec::Vec<u32>,
    #[prost(bool, tag="10")]
    pub is_index: bool,
    #[prost(uint32, tag="11")]
    pub index_on_id: u32,
    #[prost(int32, repeated, tag="12")]
    pub distribution_keys: ::prost::alloc::vec::Vec<i32>,
    #[prost(int32, repeated, tag="13")]
    pub pk: ::prost::alloc::vec::Vec<i32>,
    #[prost(oneof="table::OptionalAssociatedSourceId", tags="9")]
    pub optional_associated_source_id: ::core::option::Option<table::OptionalAssociatedSourceId>,
}
/// Nested message and enum types in `Table`.
pub mod table {
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum OptionalAssociatedSourceId {
        #[prost(uint32, tag="9")]
        AssociatedSourceId(u32),
    }
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Schema {
    #[prost(uint32, tag="1")]
    pub id: u32,
    #[prost(uint32, tag="2")]
    pub database_id: u32,
    #[prost(string, tag="3")]
    pub name: ::prost::alloc::string::String,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Database {
    #[prost(uint32, tag="1")]
    pub id: u32,
    #[prost(string, tag="2")]
    pub name: ::prost::alloc::string::String,
}
