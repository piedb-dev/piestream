#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RowSeqScanNode {
    #[prost(message, optional, tag="1")]
    pub table_desc: ::core::option::Option<super::plan_common::CellBasedTableDesc>,
    #[prost(message, repeated, tag="2")]
    pub column_descs: ::prost::alloc::vec::Vec<super::plan_common::ColumnDesc>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SourceScanNode {
    #[prost(message, optional, tag="1")]
    pub table_ref_id: ::core::option::Option<super::plan_common::TableRefId>,
    /// timestamp_ms is used for offset synchronization of high level consumer groups, this field will be deprecated if a more elegant approach is available in the future
    #[prost(int64, tag="2")]
    pub timestamp_ms: i64,
    #[prost(int32, repeated, tag="3")]
    pub column_ids: ::prost::alloc::vec::Vec<i32>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProjectNode {
    #[prost(message, repeated, tag="1")]
    pub select_list: ::prost::alloc::vec::Vec<super::expr::ExprNode>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FilterNode {
    #[prost(message, optional, tag="1")]
    pub search_condition: ::core::option::Option<super::expr::ExprNode>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FilterScanNode {
    #[prost(message, optional, tag="1")]
    pub table_ref_id: ::core::option::Option<super::plan_common::TableRefId>,
    #[prost(int32, repeated, tag="2")]
    pub column_ids: ::prost::alloc::vec::Vec<i32>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InsertNode {
    #[prost(message, optional, tag="1")]
    pub table_source_ref_id: ::core::option::Option<super::plan_common::TableRefId>,
    #[prost(int32, repeated, tag="2")]
    pub column_ids: ::prost::alloc::vec::Vec<i32>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteNode {
    #[prost(message, optional, tag="1")]
    pub table_source_ref_id: ::core::option::Option<super::plan_common::TableRefId>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateNode {
    #[prost(message, optional, tag="1")]
    pub table_source_ref_id: ::core::option::Option<super::plan_common::TableRefId>,
    #[prost(message, repeated, tag="2")]
    pub exprs: ::prost::alloc::vec::Vec<super::expr::ExprNode>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValuesNode {
    #[prost(message, repeated, tag="1")]
    pub tuples: ::prost::alloc::vec::Vec<values_node::ExprTuple>,
    #[prost(message, repeated, tag="2")]
    pub fields: ::prost::alloc::vec::Vec<super::plan_common::Field>,
}
/// Nested message and enum types in `ValuesNode`.
pub mod values_node {
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ExprTuple {
        #[prost(message, repeated, tag="1")]
        pub cells: ::prost::alloc::vec::Vec<super::super::expr::ExprNode>,
    }
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrderByNode {
    #[prost(message, repeated, tag="1")]
    pub column_orders: ::prost::alloc::vec::Vec<super::plan_common::ColumnOrder>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TopNNode {
    #[prost(message, repeated, tag="1")]
    pub column_orders: ::prost::alloc::vec::Vec<super::plan_common::ColumnOrder>,
    #[prost(uint32, tag="2")]
    pub limit: u32,
    #[prost(uint32, tag="3")]
    pub offset: u32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LimitNode {
    #[prost(uint32, tag="1")]
    pub limit: u32,
    #[prost(uint32, tag="2")]
    pub offset: u32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NestedLoopJoinNode {
    #[prost(enumeration="super::plan_common::JoinType", tag="1")]
    pub join_type: i32,
    #[prost(message, optional, tag="2")]
    pub join_cond: ::core::option::Option<super::expr::ExprNode>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HashAggNode {
    #[prost(uint32, repeated, tag="1")]
    pub group_keys: ::prost::alloc::vec::Vec<u32>,
    #[prost(message, repeated, tag="2")]
    pub agg_calls: ::prost::alloc::vec::Vec<super::expr::AggCall>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SortAggNode {
    #[prost(message, repeated, tag="1")]
    pub group_keys: ::prost::alloc::vec::Vec<super::expr::ExprNode>,
    #[prost(message, repeated, tag="2")]
    pub agg_calls: ::prost::alloc::vec::Vec<super::expr::AggCall>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HashJoinNode {
    #[prost(enumeration="super::plan_common::JoinType", tag="1")]
    pub join_type: i32,
    #[prost(int32, repeated, tag="2")]
    pub left_key: ::prost::alloc::vec::Vec<i32>,
    #[prost(int32, repeated, tag="3")]
    pub right_key: ::prost::alloc::vec::Vec<i32>,
    #[prost(message, optional, tag="4")]
    pub condition: ::core::option::Option<super::expr::ExprNode>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SortMergeJoinNode {
    #[prost(enumeration="super::plan_common::JoinType", tag="1")]
    pub join_type: i32,
    #[prost(int32, repeated, tag="2")]
    pub left_keys: ::prost::alloc::vec::Vec<i32>,
    #[prost(int32, repeated, tag="3")]
    pub right_keys: ::prost::alloc::vec::Vec<i32>,
    #[prost(enumeration="super::plan_common::OrderType", tag="4")]
    pub direction: i32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HopWindowNode {
    #[prost(message, optional, tag="1")]
    pub time_col: ::core::option::Option<super::expr::InputRefExpr>,
    #[prost(message, optional, tag="2")]
    pub window_slide: ::core::option::Option<super::data::IntervalUnit>,
    #[prost(message, optional, tag="3")]
    pub window_size: ::core::option::Option<super::data::IntervalUnit>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenerateSeriesNode {
    #[prost(message, optional, tag="1")]
    pub start: ::core::option::Option<super::expr::ExprNode>,
    #[prost(message, optional, tag="2")]
    pub stop: ::core::option::Option<super::expr::ExprNode>,
    #[prost(message, optional, tag="3")]
    pub step: ::core::option::Option<super::expr::ExprNode>,
}
/// Task is a running instance of Stage.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TaskId {
    #[prost(string, tag="1")]
    pub query_id: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub stage_id: u32,
    #[prost(uint32, tag="3")]
    pub task_id: u32,
}
/// Every task will create N buffers (channels) for parent operators to fetch results from,
/// where N is the parallelism of parent stage.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TaskOutputId {
    #[prost(message, optional, tag="1")]
    pub task_id: ::core::option::Option<TaskId>,
    /// The id of output channel to fetch from
    #[prost(uint32, tag="2")]
    pub output_id: u32,
}
/// ExchangeSource describes where to read results from children operators
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExchangeSource {
    #[prost(message, optional, tag="1")]
    pub task_output_id: ::core::option::Option<TaskOutputId>,
    #[prost(message, optional, tag="2")]
    pub host: ::core::option::Option<super::common::HostAddress>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExchangeNode {
    #[prost(message, repeated, tag="1")]
    pub sources: ::prost::alloc::vec::Vec<ExchangeSource>,
    #[prost(message, repeated, tag="3")]
    pub input_schema: ::prost::alloc::vec::Vec<super::plan_common::Field>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MergeSortExchangeNode {
    #[prost(message, optional, tag="1")]
    pub exchange: ::core::option::Option<ExchangeNode>,
    #[prost(message, repeated, tag="2")]
    pub column_orders: ::prost::alloc::vec::Vec<super::plan_common::ColumnOrder>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlanNode {
    #[prost(message, repeated, tag="1")]
    pub children: ::prost::alloc::vec::Vec<PlanNode>,
    #[prost(string, tag="24")]
    pub identity: ::prost::alloc::string::String,
    #[prost(oneof="plan_node::NodeBody", tags="2, 3, 4, 5, 7, 8, 9, 10, 11, 14, 15, 16, 17, 18, 19, 21, 22, 25, 26")]
    pub node_body: ::core::option::Option<plan_node::NodeBody>,
}
/// Nested message and enum types in `PlanNode`.
pub mod plan_node {
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum NodeBody {
        #[prost(message, tag="2")]
        Insert(super::InsertNode),
        #[prost(message, tag="3")]
        Delete(super::DeleteNode),
        #[prost(message, tag="4")]
        Update(super::UpdateNode),
        #[prost(message, tag="5")]
        Project(super::ProjectNode),
        #[prost(message, tag="7")]
        HashAgg(super::HashAggNode),
        #[prost(message, tag="8")]
        Filter(super::FilterNode),
        #[prost(message, tag="9")]
        Exchange(super::ExchangeNode),
        #[prost(message, tag="10")]
        OrderBy(super::OrderByNode),
        #[prost(message, tag="11")]
        NestedLoopJoin(super::NestedLoopJoinNode),
        #[prost(message, tag="14")]
        TopN(super::TopNNode),
        #[prost(message, tag="15")]
        SortAgg(super::SortAggNode),
        #[prost(message, tag="16")]
        RowSeqScan(super::RowSeqScanNode),
        #[prost(message, tag="17")]
        Limit(super::LimitNode),
        #[prost(message, tag="18")]
        Values(super::ValuesNode),
        #[prost(message, tag="19")]
        HashJoin(super::HashJoinNode),
        #[prost(message, tag="21")]
        MergeSortExchange(super::MergeSortExchangeNode),
        #[prost(message, tag="22")]
        SortMergeJoin(super::SortMergeJoinNode),
        #[prost(message, tag="25")]
        HopWindow(super::HopWindowNode),
        #[prost(message, tag="26")]
        GenerateSeries(super::GenerateSeriesNode),
    }
}
/// ExchangeInfo determines how to distribute results to tasks of next stage.
///
/// Note that the fragment itself does not know the where are the receivers. Instead, it prepares results in
/// N buffers and wait for parent operators (`Exchange` nodes) to pull data from a specified buffer
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExchangeInfo {
    #[prost(enumeration="exchange_info::DistributionMode", tag="1")]
    pub mode: i32,
    #[prost(oneof="exchange_info::Distribution", tags="2, 3")]
    pub distribution: ::core::option::Option<exchange_info::Distribution>,
}
/// Nested message and enum types in `ExchangeInfo`.
pub mod exchange_info {
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct BroadcastInfo {
        #[prost(uint32, tag="1")]
        pub count: u32,
    }
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct HashInfo {
        #[prost(uint32, tag="1")]
        pub output_count: u32,
        #[prost(uint32, repeated, tag="3")]
        pub keys: ::prost::alloc::vec::Vec<u32>,
    }
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum DistributionMode {
        /// No partitioning at all, used for root segment which aggregates query results
        Single = 0,
        Broadcast = 1,
        Hash = 2,
    }
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Distribution {
        #[prost(message, tag="2")]
        BroadcastInfo(BroadcastInfo),
        #[prost(message, tag="3")]
        HashInfo(HashInfo),
    }
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlanFragment {
    #[prost(message, optional, tag="1")]
    pub root: ::core::option::Option<PlanNode>,
    #[prost(message, optional, tag="2")]
    pub exchange_info: ::core::option::Option<ExchangeInfo>,
}
