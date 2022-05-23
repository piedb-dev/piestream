/// Hash mapping for compute node. Stores mapping from virtual node to actor id.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActorMapping {
    #[prost(uint64, repeated, tag="1")]
    pub original_indices: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint32, repeated, tag="2")]
    pub data: ::prost::alloc::vec::Vec<u32>,
}
/// todo: StreamSourceNode or TableSourceNode
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SourceNode {
    #[prost(message, optional, tag="1")]
    pub table_ref_id: ::core::option::Option<super::plan_common::TableRefId>,
    #[prost(int32, repeated, tag="2")]
    pub column_ids: ::prost::alloc::vec::Vec<i32>,
    #[prost(enumeration="source_node::SourceType", tag="3")]
    pub source_type: i32,
    /// split allocation information,
    /// and in the future will distinguish between `StreamSource` and `TableSource`
    /// so that there is no need to put many fields that are not common into the same SourceNode structure
    #[prost(message, optional, tag="4")]
    pub stream_source_state: ::core::option::Option<StreamSourceState>,
}
/// Nested message and enum types in `SourceNode`.
pub mod source_node {
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum SourceType {
        Table = 0,
        Source = 1,
    }
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StreamSourceState {
    #[prost(string, tag="1")]
    pub split_type: ::prost::alloc::string::String,
    #[prost(bytes="vec", repeated, tag="2")]
    pub stream_source_splits: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
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
/// A materialized view is regarded as a table,
/// hence we copy the CreateTableNode definition in OLAP PlanNode.
/// In addition, we also specify primary key to MV for efficient point lookup during update and deletion.
/// The node will be used for both create mv and create index. When creating mv,
/// `pk == distribution_keys == column_orders`. When creating index, `column_orders` will contain both
/// arrange columns and pk columns, while distribution keys will be arrange columns.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MaterializeNode {
    #[prost(message, optional, tag="1")]
    pub table_ref_id: ::core::option::Option<super::plan_common::TableRefId>,
    #[prost(message, optional, tag="2")]
    pub associated_table_ref_id: ::core::option::Option<super::plan_common::TableRefId>,
    /// Column indexes and orders of primary key
    #[prost(message, repeated, tag="3")]
    pub column_orders: ::prost::alloc::vec::Vec<super::plan_common::ColumnOrder>,
    /// Column IDs of input schema
    #[prost(int32, repeated, tag="4")]
    pub column_ids: ::prost::alloc::vec::Vec<i32>,
    /// Hash keys of the materialize node, which is a subset of pk.
    #[prost(int32, repeated, tag="5")]
    pub distribution_keys: ::prost::alloc::vec::Vec<i32>,
}
/// Remark by Yanghao: for both local and global we use the same node in the protobuf.
/// Local and global aggregator distinguish with each other in PlanNode definition.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SimpleAggNode {
    #[prost(message, repeated, tag="1")]
    pub agg_calls: ::prost::alloc::vec::Vec<super::expr::AggCall>,
    #[prost(int32, repeated, tag="2")]
    pub distribution_keys: ::prost::alloc::vec::Vec<i32>,
    #[prost(uint32, repeated, tag="3")]
    pub table_ids: ::prost::alloc::vec::Vec<u32>,
    #[prost(bool, tag="4")]
    pub append_only: bool,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HashAggNode {
    #[prost(int32, repeated, tag="1")]
    pub distribution_keys: ::prost::alloc::vec::Vec<i32>,
    #[prost(message, repeated, tag="2")]
    pub agg_calls: ::prost::alloc::vec::Vec<super::expr::AggCall>,
    #[prost(uint32, repeated, tag="3")]
    pub table_ids: ::prost::alloc::vec::Vec<u32>,
    #[prost(bool, tag="4")]
    pub append_only: bool,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TopNNode {
    #[prost(message, repeated, tag="1")]
    pub column_orders: ::prost::alloc::vec::Vec<super::plan_common::ColumnOrder>,
    /// 0 means no limit as limit of 0 means this node should be optimized away
    #[prost(uint64, tag="2")]
    pub limit: u64,
    #[prost(uint64, tag="3")]
    pub offset: u64,
    #[prost(int32, repeated, tag="4")]
    pub distribution_keys: ::prost::alloc::vec::Vec<i32>,
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
    #[prost(int32, repeated, tag="5")]
    pub distribution_keys: ::prost::alloc::vec::Vec<i32>,
    /// Whether to use delta join for this hash join node. When enabled, arrangement will be created
    /// on-the-fly within the plan.
    /// TODO: remove this in the future when we have a separate DeltaHashJoin node.
    #[prost(bool, tag="6")]
    pub is_delta_join: bool,
    /// Used for internal table states. Id of the left table.
    #[prost(uint32, tag="7")]
    pub left_table_id: u32,
    /// Used for internal table states. Id of the right table.
    #[prost(uint32, tag="8")]
    pub right_table_id: u32,
}
/// Delta join with two indexes. This is a pseudo plan node generated on frontend. On meta
/// service, it will be rewritten into lookup joins.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeltaIndexJoinNode {
    #[prost(enumeration="super::plan_common::JoinType", tag="1")]
    pub join_type: i32,
    #[prost(int32, repeated, tag="2")]
    pub left_key: ::prost::alloc::vec::Vec<i32>,
    #[prost(int32, repeated, tag="3")]
    pub right_key: ::prost::alloc::vec::Vec<i32>,
    #[prost(message, optional, tag="4")]
    pub condition: ::core::option::Option<super::expr::ExprNode>,
    /// Table id of the left index.
    #[prost(uint32, tag="7")]
    pub left_table_id: u32,
    /// Table id of the right index.
    #[prost(uint32, tag="8")]
    pub right_table_id: u32,
    /// Info about the left index
    #[prost(message, optional, tag="9")]
    pub left_info: ::core::option::Option<ArrangementInfo>,
    /// Info about the right index
    #[prost(message, optional, tag="10")]
    pub right_info: ::core::option::Option<ArrangementInfo>,
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
pub struct MergeNode {
    #[prost(uint32, repeated, tag="1")]
    pub upstream_actor_id: ::prost::alloc::vec::Vec<u32>,
    /// The schema of input columns. TODO: remove this field.
    #[prost(message, repeated, tag="2")]
    pub fields: ::prost::alloc::vec::Vec<super::plan_common::Field>,
}
/// passed from frontend to meta, used by fragmenter to generate `MergeNode`
/// and maybe `DispatcherNode` later.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExchangeNode {
    #[prost(message, optional, tag="2")]
    pub strategy: ::core::option::Option<DispatchStrategy>,
}
/// ChainNode is used for mv on mv.
/// ChainNode is like a "UNION" on mv snapshot and streaming. So it takes two inputs with fixed order:
///   1. MergeNode (as a placeholder) for streaming read.
///   2. BatchPlanNode for snapshot read.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChainNode {
    #[prost(message, optional, tag="1")]
    pub table_ref_id: ::core::option::Option<super::plan_common::TableRefId>,
    /// The schema of input stream, which will be used to build a MergeNode
    #[prost(message, repeated, tag="2")]
    pub upstream_fields: ::prost::alloc::vec::Vec<super::plan_common::Field>,
    #[prost(int32, repeated, tag="3")]
    pub column_ids: ::prost::alloc::vec::Vec<i32>,
    /// Generally, the barrier needs to be rearranged during the MV creation process, so that data can
    /// be flushed to shared buffer periodically, instead of making the first epoch from batch query extra
    /// large. However, in some cases, e.g., shared state, the barrier cannot be rearranged in ChainNode.
    /// This option is used to disable barrier rearrangement.
    #[prost(bool, tag="4")]
    pub disable_rearrange: bool,
    /// Whether to place this chain on the same worker node as upstream actors.
    #[prost(bool, tag="5")]
    pub same_worker_node: bool,
}
/// BatchPlanNode is used for mv on mv snapshot read.
/// BatchPlanNode is supposed to carry a batch plan that can be optimized with the streaming plan_common.
/// Currently, streaming to batch push down is not yet supported, BatchPlanNode is simply a table scan.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BatchPlanNode {
    #[prost(message, optional, tag="1")]
    pub table_ref_id: ::core::option::Option<super::plan_common::TableRefId>,
    #[prost(message, repeated, tag="2")]
    pub column_descs: ::prost::alloc::vec::Vec<super::plan_common::ColumnDesc>,
    #[prost(int32, repeated, tag="3")]
    pub distribution_keys: ::prost::alloc::vec::Vec<i32>,
    #[prost(message, optional, tag="4")]
    pub hash_mapping: ::core::option::Option<super::common::ParallelUnitMapping>,
    #[prost(uint32, tag="5")]
    pub parallel_unit_id: u32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ArrangementInfo {
    /// Order keys of the arrangement, including order by keys and pks from the materialize
    /// executor.
    #[prost(message, repeated, tag="1")]
    pub arrange_key_orders: ::prost::alloc::vec::Vec<super::plan_common::ColumnOrder>,
    /// Column descs of the arrangement
    #[prost(message, repeated, tag="2")]
    pub column_descs: ::prost::alloc::vec::Vec<super::plan_common::ColumnDesc>,
}
/// Special node for shared state, which will only be produced in fragmenter. ArrangeNode will
/// produce a special Materialize executor, which materializes data for downstream to query.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ArrangeNode {
    /// Table Id of the arrangement
    #[prost(uint32, tag="2")]
    pub table_id: u32,
    /// Info about the arrangement
    #[prost(message, optional, tag="3")]
    pub table_info: ::core::option::Option<ArrangementInfo>,
}
/// Special node for shared state. LookupNode will join an arrangement with a stream.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LookupNode {
    /// Join keys of the arrangement side
    #[prost(int32, repeated, tag="1")]
    pub arrange_key: ::prost::alloc::vec::Vec<i32>,
    /// Join keys of the stream side
    #[prost(int32, repeated, tag="2")]
    pub stream_key: ::prost::alloc::vec::Vec<i32>,
    /// Whether to join the current epoch of arrangement
    #[prost(bool, tag="3")]
    pub use_current_epoch: bool,
    /// Sometimes we need to re-order the output data to meet the requirement of schema.
    /// By default, lookup executor will produce `<arrangement side, stream side>`. We
    /// will then apply the column mapping to the combined result.
    #[prost(int32, repeated, tag="4")]
    pub column_mapping: ::prost::alloc::vec::Vec<i32>,
    /// Info about the arrangement
    #[prost(message, optional, tag="7")]
    pub arrangement_table_info: ::core::option::Option<ArrangementInfo>,
    #[prost(oneof="lookup_node::ArrangementTableId", tags="5, 6")]
    pub arrangement_table_id: ::core::option::Option<lookup_node::ArrangementTableId>,
}
/// Nested message and enum types in `LookupNode`.
pub mod lookup_node {
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum ArrangementTableId {
        /// Table Id of the arrangement (when created along with join plan)
        #[prost(uint32, tag="5")]
        TableId(u32),
        /// Table Id of the arrangement (when using index)
        #[prost(uint32, tag="6")]
        IndexId(u32),
    }
}
/// Acts like a merger, but on different inputs.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnionNode {
}
/// Special node for shared state. Merge and align barrier from upstreams. Pipe inputs in order.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LookupUnionNode {
    #[prost(uint32, repeated, tag="1")]
    pub order: ::prost::alloc::vec::Vec<u32>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StreamNode {
    /// The id for the operator.
    #[prost(uint64, tag="1")]
    pub operator_id: u64,
    /// Child node in plan aka. upstream nodes in the streaming DAG
    #[prost(message, repeated, tag="3")]
    pub input: ::prost::alloc::vec::Vec<StreamNode>,
    #[prost(uint32, repeated, tag="2")]
    pub pk_indices: ::prost::alloc::vec::Vec<u32>,
    #[prost(bool, tag="24")]
    pub append_only: bool,
    #[prost(string, tag="18")]
    pub identity: ::prost::alloc::string::String,
    /// The schema of the plan node
    #[prost(message, repeated, tag="19")]
    pub fields: ::prost::alloc::vec::Vec<super::plan_common::Field>,
    #[prost(oneof="stream_node::NodeBody", tags="100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119")]
    pub node_body: ::core::option::Option<stream_node::NodeBody>,
}
/// Nested message and enum types in `StreamNode`.
pub mod stream_node {
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum NodeBody {
        #[prost(message, tag="100")]
        Source(super::SourceNode),
        #[prost(message, tag="101")]
        Project(super::ProjectNode),
        #[prost(message, tag="102")]
        Filter(super::FilterNode),
        #[prost(message, tag="103")]
        Materialize(super::MaterializeNode),
        #[prost(message, tag="104")]
        LocalSimpleAgg(super::SimpleAggNode),
        #[prost(message, tag="105")]
        GlobalSimpleAgg(super::SimpleAggNode),
        #[prost(message, tag="106")]
        HashAgg(super::HashAggNode),
        #[prost(message, tag="107")]
        AppendOnlyTopN(super::TopNNode),
        #[prost(message, tag="108")]
        HashJoin(super::HashJoinNode),
        #[prost(message, tag="109")]
        TopN(super::TopNNode),
        #[prost(message, tag="110")]
        HopWindow(super::HopWindowNode),
        #[prost(message, tag="111")]
        Merge(super::MergeNode),
        #[prost(message, tag="112")]
        Exchange(super::ExchangeNode),
        #[prost(message, tag="113")]
        Chain(super::ChainNode),
        #[prost(message, tag="114")]
        BatchPlan(super::BatchPlanNode),
        #[prost(message, tag="115")]
        Lookup(super::LookupNode),
        #[prost(message, tag="116")]
        Arrange(super::ArrangeNode),
        #[prost(message, tag="117")]
        LookupUnion(super::LookupUnionNode),
        #[prost(message, tag="118")]
        Union(super::UnionNode),
        #[prost(message, tag="119")]
        DeltaIndexJoin(super::DeltaIndexJoinNode),
    }
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DispatchStrategy {
    #[prost(enumeration="DispatcherType", tag="1")]
    pub r#type: i32,
    #[prost(uint32, repeated, tag="2")]
    pub column_indices: ::prost::alloc::vec::Vec<u32>,
}
/// A dispatcher redistribute messages.
/// We encode both the type and other usage information in the proto.
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Dispatcher {
    #[prost(enumeration="DispatcherType", tag="1")]
    pub r#type: i32,
    #[prost(uint32, repeated, tag="2")]
    pub column_indices: ::prost::alloc::vec::Vec<u32>,
    /// The hash mapping for consistent hash.
    #[prost(message, optional, tag="3")]
    pub hash_mapping: ::core::option::Option<ActorMapping>,
    /// Dispatcher can be uniquely identified by a combination of actor id and dispatcher id.
    /// For dispatchers within actors, the id is the same as operator_id of the exchange plan node.
    /// For cross-MV dispatchers, there will only be one broadcast dispatcher of id 0.
    #[prost(uint64, tag="4")]
    pub dispatcher_id: u64,
    /// Number of downstreams decides how many endpoints a dispatcher should dispatch.
    #[prost(uint32, repeated, tag="5")]
    pub downstream_actor_id: ::prost::alloc::vec::Vec<u32>,
}
/// A StreamActor is a running fragment of the overall stream graph,
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StreamActor {
    #[prost(uint32, tag="1")]
    pub actor_id: u32,
    #[prost(uint32, tag="2")]
    pub fragment_id: u32,
    #[prost(message, optional, tag="3")]
    pub nodes: ::core::option::Option<StreamNode>,
    #[prost(message, repeated, tag="4")]
    pub dispatcher: ::prost::alloc::vec::Vec<Dispatcher>,
    /// The actors that send messages to this actor.
    /// Note that upstream actor ids are also stored in the proto of merge nodes.
    /// It is painstaking to traverse through the node tree and get upstream actor id from the root StreamNode.
    /// We duplicate the information here to ease the parsing logic in stream manager.
    #[prost(uint32, repeated, tag="6")]
    pub upstream_actor_id: ::prost::alloc::vec::Vec<u32>,
    /// Placement rule for actor, need to stay on the same node as upstream.
    #[prost(bool, tag="7")]
    pub same_worker_node_as_upstream: bool,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StreamFragmentGraph {
    /// all the fragments in the graph.
    #[prost(map="uint32, message", tag="1")]
    pub fragments: ::std::collections::HashMap<u32, stream_fragment_graph::StreamFragment>,
    /// edges between fragments.
    #[prost(message, repeated, tag="2")]
    pub edges: ::prost::alloc::vec::Vec<stream_fragment_graph::StreamFragmentEdge>,
    #[prost(uint32, repeated, tag="3")]
    pub dependent_table_ids: ::prost::alloc::vec::Vec<u32>,
    #[prost(uint32, tag="4")]
    pub table_ids_cnt: u32,
}
/// Nested message and enum types in `StreamFragmentGraph`.
pub mod stream_fragment_graph {
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct StreamFragment {
        /// 0-based on frontend, and will be rewritten to global id on meta.
        #[prost(uint32, tag="1")]
        pub fragment_id: u32,
        /// root stream node in this fragment.
        #[prost(message, optional, tag="2")]
        pub node: ::core::option::Option<super::StreamNode>,
        #[prost(enumeration="super::FragmentType", tag="3")]
        pub fragment_type: i32,
        /// mark whether this fragment should only have one actor.
        #[prost(bool, tag="4")]
        pub is_singleton: bool,
        /// Number of table ids (stateful states) for this fragment.
        #[prost(uint32, tag="5")]
        pub table_ids_cnt: u32,
    }
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct StreamFragmentEdge {
        /// Dispatch strategy for the fragment.
        #[prost(message, optional, tag="1")]
        pub dispatch_strategy: ::core::option::Option<super::DispatchStrategy>,
        /// Whether the two linked nodes should be placed on the same worker node
        #[prost(bool, tag="2")]
        pub same_worker_node: bool,
        /// A unique identifer of this edge. Generally it should be exchange node's operator id. When
        /// rewriting fragments into delta joins or when inserting 1-to-1 exchange, there will be
        /// virtual links generated.
        #[prost(uint64, tag="3")]
        pub link_id: u64,
        #[prost(uint32, tag="4")]
        pub upstream_id: u32,
        #[prost(uint32, tag="5")]
        pub downstream_id: u32,
    }
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum DispatcherType {
    Invalid = 0,
    /// Dispatch by hash key, hashed by consistent hash.
    Hash = 1,
    /// Broadcast to all downstreams.
    /// TODO: we don't need this as we now support multi-dispatcher per actor.
    Broadcast = 2,
    /// Only one downstream.
    /// TODO: seems that same as broadcast dispatch (with only one downstream actor).
    Simple = 3,
    /// A special kind of exchange that doesn't involve shuffle. The upstream actor will be directly
    /// piped into the downstream actor, if there are the same number of actors. If number of actors
    /// are not the same, should use hash instead. Should be only used when distribution is the same.
    NoShuffle = 4,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum FragmentType {
    Source = 0,
    Sink = 1,
    Others = 2,
}
