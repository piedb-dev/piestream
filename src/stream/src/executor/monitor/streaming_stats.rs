// Copyright 2022 PieDb Data
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use prometheus::core::{AtomicF64, AtomicI64, AtomicU64, GenericCounterVec, GenericGaugeVec};
use prometheus::{
    exponential_buckets, histogram_opts, register_gauge_vec_with_registry,
    register_histogram_vec_with_registry, register_histogram_with_registry,
    register_int_counter_vec_with_registry, register_int_gauge_vec_with_registry, Histogram,
    HistogramVec, Registry,
};

pub struct StreamingMetrics {
    pub registry: Registry,
    pub executor_row_count: GenericCounterVec<AtomicU64>,
    pub actor_execution_time: GenericGaugeVec<AtomicF64>,
    pub actor_output_buffer_blocking_duration_ns: GenericCounterVec<AtomicU64>,
    pub actor_input_buffer_blocking_duration_ns: GenericCounterVec<AtomicU64>,
    pub actor_scheduled_duration: GenericGaugeVec<AtomicF64>,
    pub actor_scheduled_cnt: GenericGaugeVec<AtomicI64>,
    pub actor_fast_poll_duration: GenericGaugeVec<AtomicF64>,
    pub actor_fast_poll_cnt: GenericGaugeVec<AtomicI64>,
    pub actor_slow_poll_duration: GenericGaugeVec<AtomicF64>,
    pub actor_slow_poll_cnt: GenericGaugeVec<AtomicI64>,
    pub actor_poll_duration: GenericGaugeVec<AtomicF64>,
    pub actor_poll_cnt: GenericGaugeVec<AtomicI64>,
    pub actor_idle_duration: GenericGaugeVec<AtomicF64>,
    pub actor_idle_cnt: GenericGaugeVec<AtomicI64>,
    pub actor_in_record_cnt: GenericCounterVec<AtomicU64>,
    pub actor_out_record_cnt: GenericCounterVec<AtomicU64>,
    pub actor_sampled_deserialize_duration_ns: GenericCounterVec<AtomicU64>,
    pub source_output_row_count: GenericCounterVec<AtomicU64>,
    pub exchange_recv_size: GenericCounterVec<AtomicU64>,
    pub exchange_frag_recv_size: GenericCounterVec<AtomicU64>,

    // Streaming Join
    pub join_lookup_miss_count: GenericCounterVec<AtomicU64>,
    pub join_total_lookup_count: GenericCounterVec<AtomicU64>,
    pub join_actor_input_waiting_duration_ns: GenericCounterVec<AtomicU64>,
    pub join_barrier_align_duration: HistogramVec,
    pub join_cached_entries: GenericGaugeVec<AtomicI64>,
    pub join_cached_rows: GenericGaugeVec<AtomicI64>,
    pub join_cached_estimated_size: GenericGaugeVec<AtomicI64>,

    // Streaming Aggregation
    pub agg_lookup_miss_count: GenericCounterVec<AtomicU64>,
    pub agg_total_lookup_count: GenericCounterVec<AtomicU64>,
    pub agg_cached_keys: GenericGaugeVec<AtomicI64>,

    /// The duration from receipt of barrier to all actors collection.
    /// And the max of all node `barrier_inflight_latency` is the latency for a barrier
    /// to flow through the graph.
    pub barrier_inflight_latency: Histogram,
    /// The duration of sync to storage.
    pub barrier_sync_latency: Histogram,

    pub sink_commit_duration: HistogramVec,
}

impl StreamingMetrics {
    pub fn new(registry: Registry) -> Self {
        let executor_row_count = register_int_counter_vec_with_registry!(
            "stream_executor_row_count",
            "Total number of rows that have been output from each executor",
            &["actor_id", "executor_id"],
            registry
        )
        .unwrap();

        let source_output_row_count = register_int_counter_vec_with_registry!(
            "stream_source_output_rows_counts",
            "Total number of rows that have been output from source",
            &["source_id"],
            registry
        )
        .unwrap();

        let actor_execution_time = register_gauge_vec_with_registry!(
            "stream_actor_actor_execution_time",
            "Total execution time (s) of an actor",
            &["actor_id"],
            registry
        )
        .unwrap();

        let actor_output_buffer_blocking_duration_ns = register_int_counter_vec_with_registry!(
            "stream_actor_output_buffer_blocking_duration_ns",
            "Total blocking duration (ns) of output buffer",
            &["actor_id"],
            registry
        )
        .unwrap();

        let actor_input_buffer_blocking_duration_ns = register_int_counter_vec_with_registry!(
            "stream_actor_input_buffer_blocking_duration_ns",
            "Total blocking duration (ns) of input buffer",
            &["actor_id", "upstream_fragment_id"],
            registry
        )
        .unwrap();

        let exchange_recv_size = register_int_counter_vec_with_registry!(
            "stream_exchange_recv_size",
            "Total size of messages that have been received from upstream Actor",
            &["up_actor_id", "down_actor_id"],
            registry
        )
        .unwrap();

        let exchange_frag_recv_size = register_int_counter_vec_with_registry!(
            "stream_exchange_frag_recv_size",
            "Total size of messages that have been received from upstream Fragment",
            &["up_fragment_id", "down_fragment_id"],
            registry
        )
        .unwrap();

        let actor_fast_poll_duration = register_gauge_vec_with_registry!(
            "stream_actor_fast_poll_duration",
            "tokio's metrics",
            &["actor_id"],
            registry
        )
        .unwrap();

        let actor_fast_poll_cnt = register_int_gauge_vec_with_registry!(
            "stream_actor_fast_poll_cnt",
            "tokio's metrics",
            &["actor_id"],
            registry
        )
        .unwrap();

        let actor_slow_poll_duration = register_gauge_vec_with_registry!(
            "stream_actor_slow_poll_duration",
            "tokio's metrics",
            &["actor_id"],
            registry
        )
        .unwrap();

        let actor_slow_poll_cnt = register_int_gauge_vec_with_registry!(
            "stream_actor_slow_poll_cnt",
            "tokio's metrics",
            &["actor_id"],
            registry
        )
        .unwrap();

        let actor_poll_duration = register_gauge_vec_with_registry!(
            "stream_actor_poll_duration",
            "tokio's metrics",
            &["actor_id"],
            registry
        )
        .unwrap();

        let actor_poll_cnt = register_int_gauge_vec_with_registry!(
            "stream_actor_poll_cnt",
            "tokio's metrics",
            &["actor_id"],
            registry
        )
        .unwrap();

        let actor_scheduled_duration = register_gauge_vec_with_registry!(
            "stream_actor_scheduled_duration",
            "tokio's metrics",
            &["actor_id"],
            registry
        )
        .unwrap();

        let actor_scheduled_cnt = register_int_gauge_vec_with_registry!(
            "stream_actor_scheduled_cnt",
            "tokio's metrics",
            &["actor_id"],
            registry
        )
        .unwrap();

        let actor_idle_duration = register_gauge_vec_with_registry!(
            "stream_actor_idle_duration",
            "tokio's metrics",
            &["actor_id"],
            registry
        )
        .unwrap();

        let actor_idle_cnt = register_int_gauge_vec_with_registry!(
            "stream_actor_idle_cnt",
            "tokio's metrics",
            &["actor_id"],
            registry
        )
        .unwrap();

        let actor_in_record_cnt = register_int_counter_vec_with_registry!(
            "stream_actor_in_record_cnt",
            "Total number of rows actor received",
            &["actor_id"],
            registry
        )
        .unwrap();

        let actor_out_record_cnt = register_int_counter_vec_with_registry!(
            "stream_actor_out_record_cnt",
            "Total number of rows actor sent",
            &["actor_id"],
            registry
        )
        .unwrap();

        let actor_sampled_deserialize_duration_ns = register_int_counter_vec_with_registry!(
            "actor_sampled_deserialize_duration_ns",
            "Duration (ns) of sampled chunk deserialization",
            &["actor_id"],
            registry
        )
        .unwrap();

        let join_lookup_miss_count = register_int_counter_vec_with_registry!(
            "stream_join_lookup_miss_count",
            "Join executor lookup miss duration",
            &["actor_id", "side"],
            registry
        )
        .unwrap();

        let join_total_lookup_count = register_int_counter_vec_with_registry!(
            "stream_join_lookup_total_count",
            "Join executor lookup total operation",
            &["actor_id", "side"],
            registry
        )
        .unwrap();

        let join_actor_input_waiting_duration_ns = register_int_counter_vec_with_registry!(
            "stream_join_actor_input_waiting_duration_ns",
            "Total waiting duration (ns) of input buffer of join actor",
            &["actor_id"],
            registry
        )
        .unwrap();

        let opts = histogram_opts!(
            "stream_join_barrier_align_duration",
            "Duration of join align barrier",
            exponential_buckets(0.0001, 2.0, 21).unwrap() // max 104s
        );
        let join_barrier_align_duration =
            register_histogram_vec_with_registry!(opts, &["actor_id", "wait_side"], registry)
                .unwrap();

        let join_cached_entries = register_int_gauge_vec_with_registry!(
            "stream_join_cached_entries",
            "Number of cached entries in streaming join operators",
            &["actor_id", "side"],
            registry
        )
        .unwrap();

        let join_cached_rows = register_int_gauge_vec_with_registry!(
            "stream_join_cached_rows",
            "Number of cached rows in streaming join operators",
            &["actor_id", "side"],
            registry
        )
        .unwrap();

        let join_cached_estimated_size = register_int_gauge_vec_with_registry!(
            "stream_join_cached_estimated_size",
            "Estimated size of all cached entries in streaming join operators",
            &["actor_id", "side"],
            registry
        )
        .unwrap();

        let agg_lookup_miss_count = register_int_counter_vec_with_registry!(
            "stream_agg_lookup_miss_count",
            "Aggregation executor lookup miss duration",
            &["actor_id"],
            registry
        )
        .unwrap();

        let agg_total_lookup_count = register_int_counter_vec_with_registry!(
            "stream_agg_lookup_total_count",
            "Aggregation executor lookup total operation",
            &["actor_id"],
            registry
        )
        .unwrap();

        let agg_cached_keys = register_int_gauge_vec_with_registry!(
            "stream_agg_cached_keys",
            "Number of cached keys in streaming aggregation operators",
            &["actor_id"],
            registry
        )
        .unwrap();

        let opts = histogram_opts!(
            "stream_barrier_inflight_duration_seconds",
            "barrier_inflight_latency",
            exponential_buckets(0.1, 1.5, 16).unwrap() // max 43s
        );
        let barrier_inflight_latency = register_histogram_with_registry!(opts, registry).unwrap();

        let opts = histogram_opts!(
            "stream_barrier_sync_storage_duration_seconds",
            "barrier_sync_latency",
            exponential_buckets(0.1, 1.5, 16).unwrap() // max 43s
        );
        let barrier_sync_latency = register_histogram_with_registry!(opts, registry).unwrap();
        let sink_commit_duration = register_histogram_vec_with_registry!(
            "sink_commit_duration",
            "Duration of commit op in sink",
            &["executor_id", "connector"],
            registry
        )
        .unwrap();

        Self {
            registry,
            executor_row_count,
            actor_execution_time,
            actor_output_buffer_blocking_duration_ns,
            actor_input_buffer_blocking_duration_ns,
            actor_scheduled_duration,
            actor_scheduled_cnt,
            actor_fast_poll_duration,
            actor_fast_poll_cnt,
            actor_slow_poll_duration,
            actor_slow_poll_cnt,
            actor_poll_duration,
            actor_poll_cnt,
            actor_idle_duration,
            actor_idle_cnt,
            actor_in_record_cnt,
            actor_out_record_cnt,
            actor_sampled_deserialize_duration_ns,
            source_output_row_count,
            exchange_recv_size,
            exchange_frag_recv_size,
            join_lookup_miss_count,
            join_total_lookup_count,
            join_actor_input_waiting_duration_ns,
            join_barrier_align_duration,
            join_cached_entries,
            join_cached_rows,
            join_cached_estimated_size,
            agg_lookup_miss_count,
            agg_total_lookup_count,
            agg_cached_keys,
            barrier_inflight_latency,
            barrier_sync_latency,
            sink_commit_duration,
        }
    }

    /// Create a new `StreamingMetrics` instance used in tests or other places.
    pub fn unused() -> Self {
        Self::new(prometheus::Registry::new())
    }
}
