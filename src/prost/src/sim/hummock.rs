#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SstableRefId {
    #[prost(uint64, tag="1")]
    pub id: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SstableIdInfo {
    #[prost(uint64, tag="1")]
    pub id: u64,
    /// Timestamp when the sstable id is created, in seconds.
    #[prost(uint64, tag="2")]
    pub id_create_timestamp: u64,
    /// Timestamp when the sstable is tracked in meta node, in seconds.
    #[prost(uint64, tag="3")]
    pub meta_create_timestamp: u64,
    /// Timestamp when the sstable is marked to delete, in seconds.
    #[prost(uint64, tag="4")]
    pub meta_delete_timestamp: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VNodeBitmap {
    #[prost(uint32, tag="1")]
    pub table_id: u32,
    #[prost(uint32, tag="2")]
    pub maplen: u32,
    #[prost(bytes="vec", tag="3")]
    pub bitmap: ::prost::alloc::vec::Vec<u8>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SstableInfo {
    #[prost(uint64, tag="1")]
    pub id: u64,
    #[prost(message, optional, tag="2")]
    pub key_range: ::core::option::Option<KeyRange>,
    #[prost(uint64, tag="3")]
    pub file_size: u64,
    #[prost(message, repeated, tag="4")]
    pub vnode_bitmaps: ::prost::alloc::vec::Vec<VNodeBitmap>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Level {
    #[prost(uint32, tag="1")]
    pub level_idx: u32,
    #[prost(enumeration="LevelType", tag="2")]
    pub level_type: i32,
    #[prost(message, repeated, tag="3")]
    pub table_infos: ::prost::alloc::vec::Vec<SstableInfo>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UncommittedEpoch {
    #[prost(uint64, tag="1")]
    pub epoch: u64,
    #[prost(message, repeated, tag="2")]
    pub tables: ::prost::alloc::vec::Vec<SstableInfo>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HummockVersionRefId {
    #[prost(uint64, tag="1")]
    pub id: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HummockVersion {
    #[prost(uint64, tag="1")]
    pub id: u64,
    #[prost(message, repeated, tag="2")]
    pub levels: ::prost::alloc::vec::Vec<Level>,
    #[prost(message, repeated, tag="3")]
    pub uncommitted_epochs: ::prost::alloc::vec::Vec<UncommittedEpoch>,
    #[prost(uint64, tag="4")]
    pub max_committed_epoch: u64,
    /// Snapshots with epoch less than the safe epoch have been GCed.
    /// Reads against such an epoch will fail.
    #[prost(uint64, tag="5")]
    pub safe_epoch: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HummockSnapshot {
    #[prost(uint64, tag="1")]
    pub epoch: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddTablesRequest {
    #[prost(uint32, tag="1")]
    pub context_id: u32,
    #[prost(message, repeated, tag="2")]
    pub tables: ::prost::alloc::vec::Vec<SstableInfo>,
    #[prost(uint64, tag="3")]
    pub epoch: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddTablesResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, optional, tag="2")]
    pub version: ::core::option::Option<HummockVersion>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PinVersionRequest {
    #[prost(uint32, tag="1")]
    pub context_id: u32,
    #[prost(uint64, tag="2")]
    pub last_pinned: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PinVersionResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, optional, tag="2")]
    pub pinned_version: ::core::option::Option<HummockVersion>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnpinVersionRequest {
    #[prost(uint32, tag="1")]
    pub context_id: u32,
    #[prost(uint64, repeated, tag="2")]
    pub pinned_version_ids: ::prost::alloc::vec::Vec<u64>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnpinVersionResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PinSnapshotRequest {
    #[prost(uint32, tag="1")]
    pub context_id: u32,
    #[prost(uint64, tag="2")]
    pub last_pinned: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PinSnapshotResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, optional, tag="2")]
    pub snapshot: ::core::option::Option<HummockSnapshot>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnpinSnapshotRequest {
    #[prost(uint32, tag="1")]
    pub context_id: u32,
    #[prost(message, repeated, tag="2")]
    pub snapshots: ::prost::alloc::vec::Vec<HummockSnapshot>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnpinSnapshotResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeyRange {
    #[prost(bytes="vec", tag="1")]
    pub left: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub right: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag="3")]
    pub inf: bool,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TableSetStatistics {
    #[prost(uint32, tag="1")]
    pub level_idx: u32,
    #[prost(double, tag="2")]
    pub size_gb: f64,
    #[prost(uint64, tag="3")]
    pub cnt: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompactMetrics {
    #[prost(message, optional, tag="1")]
    pub read_level_n: ::core::option::Option<TableSetStatistics>,
    #[prost(message, optional, tag="2")]
    pub read_level_nplus1: ::core::option::Option<TableSetStatistics>,
    #[prost(message, optional, tag="3")]
    pub write: ::core::option::Option<TableSetStatistics>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompactTask {
    /// SSTs to be compacted, which will be removed from LSM after compaction
    #[prost(message, repeated, tag="1")]
    pub input_ssts: ::prost::alloc::vec::Vec<Level>,
    /// In ideal case, the compaction will generate `splits.len()` tables which have key range
    /// corresponding to that in \[`splits`\], respectively
    #[prost(message, repeated, tag="2")]
    pub splits: ::prost::alloc::vec::Vec<KeyRange>,
    /// low watermark in 'ts-aware compaction'
    #[prost(uint64, tag="3")]
    pub watermark: u64,
    /// compacion output, which will be added to \[`target_level`\] of LSM after compaction
    #[prost(message, repeated, tag="4")]
    pub sorted_output_ssts: ::prost::alloc::vec::Vec<SstableInfo>,
    /// task id assigned by hummock storage service
    #[prost(uint64, tag="5")]
    pub task_id: u64,
    /// compaction output will be added to \[`target_level`\] of LSM after compaction
    #[prost(uint32, tag="6")]
    pub target_level: u32,
    #[prost(bool, tag="7")]
    pub is_target_ultimate_and_leveling: bool,
    #[prost(message, optional, tag="8")]
    pub metrics: ::core::option::Option<CompactMetrics>,
    #[prost(bool, tag="9")]
    pub task_status: bool,
    #[prost(message, repeated, tag="10")]
    pub prefix_pairs: ::prost::alloc::vec::Vec<compaction_group::PrefixPair>,
    /// Hash mapping from virtual node to parallel unit. Since one compactor might deal with SSTs
    /// with data for more than one relational state tables, here a vector is required.
    #[prost(message, repeated, tag="11")]
    pub vnode_mappings: ::prost::alloc::vec::Vec<super::common::ParallelUnitMapping>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompactionGroup {
}
/// Nested message and enum types in `CompactionGroup`.
pub mod compaction_group {
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct PrefixPair {
        /// key value with `prefix` belongs to compaction group `group_id`
        #[prost(uint64, tag="1")]
        pub group_id: u64,
        #[prost(bytes="vec", tag="2")]
        pub prefix: ::prost::alloc::vec::Vec<u8>,
    }
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LevelHandler {
    #[prost(uint32, tag="1")]
    pub level: u32,
    #[prost(message, repeated, tag="3")]
    pub tasks: ::prost::alloc::vec::Vec<level_handler::SstTask>,
}
/// Nested message and enum types in `LevelHandler`.
pub mod level_handler {
    #[derive(prost_helpers::AnyPB)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct SstTask {
        #[prost(uint64, tag="1")]
        pub task_id: u64,
        #[prost(uint64, repeated, tag="2")]
        pub ssts: ::prost::alloc::vec::Vec<u64>,
    }
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompactStatus {
    #[prost(message, repeated, tag="1")]
    pub level_handlers: ::prost::alloc::vec::Vec<LevelHandler>,
    #[prost(uint64, tag="2")]
    pub next_compact_task_id: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompactTaskAssignment {
    #[prost(message, optional, tag="1")]
    pub compact_task: ::core::option::Option<CompactTask>,
    #[prost(uint32, tag="2")]
    pub context_id: u32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompactTaskRefId {
    #[prost(uint64, tag="1")]
    pub id: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCompactionTasksRequest {
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCompactionTasksResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, optional, tag="2")]
    pub compact_task: ::core::option::Option<CompactTask>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportCompactionTasksRequest {
    #[prost(message, optional, tag="1")]
    pub compact_task: ::core::option::Option<CompactTask>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportCompactionTasksResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HummockContextRefId {
    #[prost(uint32, tag="1")]
    pub id: u32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HummockPinnedVersion {
    #[prost(uint32, tag="1")]
    pub context_id: u32,
    #[prost(uint64, repeated, tag="2")]
    pub version_id: ::prost::alloc::vec::Vec<u64>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HummockPinnedSnapshot {
    #[prost(uint32, tag="1")]
    pub context_id: u32,
    #[prost(uint64, repeated, tag="2")]
    pub snapshot_id: ::prost::alloc::vec::Vec<u64>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HummockStaleSstables {
    #[prost(uint64, tag="1")]
    pub version_id: u64,
    /// sstable ids
    #[prost(uint64, repeated, tag="2")]
    pub id: ::prost::alloc::vec::Vec<u64>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitEpochRequest {
    #[prost(uint64, tag="1")]
    pub epoch: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitEpochResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AbortEpochRequest {
    #[prost(uint64, tag="1")]
    pub epoch: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AbortEpochResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetNewTableIdRequest {
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetNewTableIdResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(uint64, tag="2")]
    pub table_id: u64,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeCompactTasksRequest {
    #[prost(uint32, tag="1")]
    pub context_id: u32,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeCompactTasksResponse {
    #[prost(message, optional, tag="1")]
    pub compact_task: ::core::option::Option<CompactTask>,
    #[prost(message, optional, tag="2")]
    pub vacuum_task: ::core::option::Option<VacuumTask>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VacuumTask {
    #[prost(uint64, repeated, tag="1")]
    pub sstable_ids: ::prost::alloc::vec::Vec<u64>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportVacuumTaskRequest {
    #[prost(message, optional, tag="1")]
    pub vacuum_task: ::core::option::Option<VacuumTask>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportVacuumTaskResponse {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[derive(prost_helpers::AnyPB)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum LevelType {
    Nonoverlapping = 0,
    Overlapping = 1,
}
/// Generated client implementations.
pub mod hummock_manager_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct HummockManagerServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl HummockManagerServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl HummockManagerServiceClient<tonic::transport::Channel> {
        pub fn new(inner: tonic::transport::Channel) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(self) -> Self {
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(self) -> Self {
            self
        }
        pub async fn pin_version(
            &mut self,
            request: impl tonic::IntoRequest<super::PinVersionRequest>,
        ) -> Result<tonic::Response<super::PinVersionResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e),
                    )
                })?;
            let codec = ();
            let path = http::uri::PathAndQuery::from_static(
                "/hummock.HummockManagerService/PinVersion",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn unpin_version(
            &mut self,
            request: impl tonic::IntoRequest<super::UnpinVersionRequest>,
        ) -> Result<tonic::Response<super::UnpinVersionResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e),
                    )
                })?;
            let codec = ();
            let path = http::uri::PathAndQuery::from_static(
                "/hummock.HummockManagerService/UnpinVersion",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn add_tables(
            &mut self,
            request: impl tonic::IntoRequest<super::AddTablesRequest>,
        ) -> Result<tonic::Response<super::AddTablesResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e),
                    )
                })?;
            let codec = ();
            let path = http::uri::PathAndQuery::from_static(
                "/hummock.HummockManagerService/AddTables",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn report_compaction_tasks(
            &mut self,
            request: impl tonic::IntoRequest<super::ReportCompactionTasksRequest>,
        ) -> Result<
                tonic::Response<super::ReportCompactionTasksResponse>,
                tonic::Status,
            > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e),
                    )
                })?;
            let codec = ();
            let path = http::uri::PathAndQuery::from_static(
                "/hummock.HummockManagerService/ReportCompactionTasks",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn pin_snapshot(
            &mut self,
            request: impl tonic::IntoRequest<super::PinSnapshotRequest>,
        ) -> Result<tonic::Response<super::PinSnapshotResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e),
                    )
                })?;
            let codec = ();
            let path = http::uri::PathAndQuery::from_static(
                "/hummock.HummockManagerService/PinSnapshot",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn unpin_snapshot(
            &mut self,
            request: impl tonic::IntoRequest<super::UnpinSnapshotRequest>,
        ) -> Result<tonic::Response<super::UnpinSnapshotResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e),
                    )
                })?;
            let codec = ();
            let path = http::uri::PathAndQuery::from_static(
                "/hummock.HummockManagerService/UnpinSnapshot",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn commit_epoch(
            &mut self,
            request: impl tonic::IntoRequest<super::CommitEpochRequest>,
        ) -> Result<tonic::Response<super::CommitEpochResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e),
                    )
                })?;
            let codec = ();
            let path = http::uri::PathAndQuery::from_static(
                "/hummock.HummockManagerService/CommitEpoch",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn abort_epoch(
            &mut self,
            request: impl tonic::IntoRequest<super::AbortEpochRequest>,
        ) -> Result<tonic::Response<super::AbortEpochResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e),
                    )
                })?;
            let codec = ();
            let path = http::uri::PathAndQuery::from_static(
                "/hummock.HummockManagerService/AbortEpoch",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_new_table_id(
            &mut self,
            request: impl tonic::IntoRequest<super::GetNewTableIdRequest>,
        ) -> Result<tonic::Response<super::GetNewTableIdResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e),
                    )
                })?;
            let codec = ();
            let path = http::uri::PathAndQuery::from_static(
                "/hummock.HummockManagerService/GetNewTableId",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn subscribe_compact_tasks(
            &mut self,
            request: impl tonic::IntoRequest<super::SubscribeCompactTasksRequest>,
        ) -> Result<
                tonic::Response<
                    tonic::codec::Streaming<super::SubscribeCompactTasksResponse>,
                >,
                tonic::Status,
            > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e),
                    )
                })?;
            let codec = ();
            let path = http::uri::PathAndQuery::from_static(
                "/hummock.HummockManagerService/SubscribeCompactTasks",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        pub async fn report_vacuum_task(
            &mut self,
            request: impl tonic::IntoRequest<super::ReportVacuumTaskRequest>,
        ) -> Result<tonic::Response<super::ReportVacuumTaskResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e),
                    )
                })?;
            let codec = ();
            let path = http::uri::PathAndQuery::from_static(
                "/hummock.HummockManagerService/ReportVacuumTask",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated client implementations.
pub mod compactor_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct CompactorServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl CompactorServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl CompactorServiceClient<tonic::transport::Channel> {
        pub fn new(inner: tonic::transport::Channel) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(self) -> Self {
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(self) -> Self {
            self
        }
    }
}
/// Generated server implementations.
pub mod hummock_manager_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        unused_mut,
        clippy::let_unit_value,
    )]
    use tonic::codegen::{
        http::uri::PathAndQuery, futures::stream::{self, StreamExt},
        *,
    };
    #[async_trait]
    pub trait HummockManagerService: Send + Sync + 'static {
        async fn pin_version(
            &self,
            request: tonic::Request<super::PinVersionRequest>,
        ) -> Result<tonic::Response<super::PinVersionResponse>, tonic::Status>;
        async fn unpin_version(
            &self,
            request: tonic::Request<super::UnpinVersionRequest>,
        ) -> Result<tonic::Response<super::UnpinVersionResponse>, tonic::Status>;
        async fn add_tables(
            &self,
            request: tonic::Request<super::AddTablesRequest>,
        ) -> Result<tonic::Response<super::AddTablesResponse>, tonic::Status>;
        async fn report_compaction_tasks(
            &self,
            request: tonic::Request<super::ReportCompactionTasksRequest>,
        ) -> Result<
                tonic::Response<super::ReportCompactionTasksResponse>,
                tonic::Status,
            >;
        async fn pin_snapshot(
            &self,
            request: tonic::Request<super::PinSnapshotRequest>,
        ) -> Result<tonic::Response<super::PinSnapshotResponse>, tonic::Status>;
        async fn unpin_snapshot(
            &self,
            request: tonic::Request<super::UnpinSnapshotRequest>,
        ) -> Result<tonic::Response<super::UnpinSnapshotResponse>, tonic::Status>;
        async fn commit_epoch(
            &self,
            request: tonic::Request<super::CommitEpochRequest>,
        ) -> Result<tonic::Response<super::CommitEpochResponse>, tonic::Status>;
        async fn abort_epoch(
            &self,
            request: tonic::Request<super::AbortEpochRequest>,
        ) -> Result<tonic::Response<super::AbortEpochResponse>, tonic::Status>;
        async fn get_new_table_id(
            &self,
            request: tonic::Request<super::GetNewTableIdRequest>,
        ) -> Result<tonic::Response<super::GetNewTableIdResponse>, tonic::Status>;
        type SubscribeCompactTasksStream: futures_core::Stream<
                Item = Result<super::SubscribeCompactTasksResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn subscribe_compact_tasks(
            &self,
            request: tonic::Request<super::SubscribeCompactTasksRequest>,
        ) -> Result<tonic::Response<Self::SubscribeCompactTasksStream>, tonic::Status>;
        async fn report_vacuum_task(
            &self,
            request: tonic::Request<super::ReportVacuumTaskRequest>,
        ) -> Result<tonic::Response<super::ReportVacuumTaskResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct HummockManagerServiceServer<T: HummockManagerService> {
        inner: Arc<T>,
    }
    impl<T: HummockManagerService> HummockManagerServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self { inner }
        }
    }
    impl<T> tonic::codegen::Service<(PathAndQuery, BoxMessageStream)>
    for HummockManagerServiceServer<T>
    where
        T: HummockManagerService,
    {
        type Response = BoxMessageStream;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(
            &mut self,
            (path, mut req): (PathAndQuery, BoxMessageStream),
        ) -> Self::Future {
            let inner = self.inner.clone();
            match path.path() {
                "/hummock.HummockManagerService/PinVersion" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::PinVersionRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .pin_version(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/hummock.HummockManagerService/UnpinVersion" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::UnpinVersionRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .unpin_version(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/hummock.HummockManagerService/AddTables" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::AddTablesRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .add_tables(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/hummock.HummockManagerService/ReportCompactionTasks" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<
                            tonic::Request<super::ReportCompactionTasksRequest>,
                        >()
                            .unwrap();
                        let res = (*inner)
                            .report_compaction_tasks(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/hummock.HummockManagerService/PinSnapshot" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::PinSnapshotRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .pin_snapshot(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/hummock.HummockManagerService/UnpinSnapshot" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::UnpinSnapshotRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .unpin_snapshot(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/hummock.HummockManagerService/CommitEpoch" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::CommitEpochRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .commit_epoch(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/hummock.HummockManagerService/AbortEpoch" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::AbortEpochRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .abort_epoch(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/hummock.HummockManagerService/GetNewTableId" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::GetNewTableIdRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .get_new_table_id(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                "/hummock.HummockManagerService/SubscribeCompactTasks" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<
                            tonic::Request<super::SubscribeCompactTasksRequest>,
                        >()
                            .unwrap();
                        let res = (*inner)
                            .subscribe_compact_tasks(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            res
                                .into_inner()
                                .map(|res| res.map(|rsp| Box::new(rsp) as BoxMessage))
                                .boxed(),
                        )
                    })
                }
                "/hummock.HummockManagerService/ReportVacuumTask" => {
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let request = *req
                            .next()
                            .await
                            .unwrap()
                            .unwrap()
                            .downcast::<tonic::Request<super::ReportVacuumTaskRequest>>()
                            .unwrap();
                        let res = (*inner)
                            .report_vacuum_task(request)
                            .await
                            .expect("rpc handler returns error");
                        Ok(
                            stream::once(async move { Ok(Box::new(res) as BoxMessage) })
                                .boxed(),
                        )
                    })
                }
                _ => Box::pin(async move { Ok(stream::empty().boxed()) }),
            }
        }
    }
    impl<T: HummockManagerService> Clone for HummockManagerServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: HummockManagerService> tonic::transport::NamedService
    for HummockManagerServiceServer<T> {
        const NAME: &'static str = "hummock.HummockManagerService";
    }
}
/// Generated server implementations.
pub mod compactor_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        unused_mut,
        clippy::let_unit_value,
    )]
    use tonic::codegen::{
        http::uri::PathAndQuery, futures::stream::{self, StreamExt},
        *,
    };
    #[async_trait]
    pub trait CompactorService: Send + Sync + 'static {}
    #[derive(Debug)]
    pub struct CompactorServiceServer<T: CompactorService> {
        inner: Arc<T>,
    }
    impl<T: CompactorService> CompactorServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self { inner }
        }
    }
    impl<T> tonic::codegen::Service<(PathAndQuery, BoxMessageStream)>
    for CompactorServiceServer<T>
    where
        T: CompactorService,
    {
        type Response = BoxMessageStream;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(
            &mut self,
            (path, mut req): (PathAndQuery, BoxMessageStream),
        ) -> Self::Future {
            let inner = self.inner.clone();
            match path.path() {
                _ => Box::pin(async move { Ok(stream::empty().boxed()) }),
            }
        }
    }
    impl<T: CompactorService> Clone for CompactorServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: CompactorService> tonic::transport::NamedService
    for CompactorServiceServer<T> {
        const NAME: &'static str = "hummock.CompactorService";
    }
}
