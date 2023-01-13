// Copyright 2022 Piedb Data
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

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use piestream_common::config::load_config;
use piestream_common::monitor::process_linux::monitor_process;
use piestream_common::util::addr::HostAddr;
use piestream_common_service::metrics_manager::MetricsManager;
use piestream_common_service::observer_manager::ObserverManager;
use piestream_hummock_sdk::filter_key_extractor::FilterKeyExtractorManager;
use piestream_object_store::object::parse_remote_object_store;
use piestream_pb::common::WorkerType;
use piestream_pb::hummock::compactor_service_server::CompactorServiceServer;
use piestream_rpc_client::MetaClient;
use piestream_storage::hummock::compactor::{CompactionExecutor, CompactorContext, Context};
use piestream_storage::hummock::hummock_meta_client::MonitoredHummockMetaClient;
use piestream_storage::hummock::{
    CompactorMemoryCollector, CompactorSstableStore, MemoryLimiter, SstableIdManager, SstableStore,
};
use piestream_storage::monitor::{
    monitor_cache, HummockMetrics, ObjectStoreMetrics, StateStoreMetrics,
};
use tokio::sync::oneshot::Sender;
use tokio::task::JoinHandle;

use super::compactor_observer::observer_manager::CompactorObserverNode;
use crate::rpc::CompactorServiceImpl;
use crate::{CompactorConfig, CompactorOpts};

/// Fetches and runs compaction tasks.
pub async fn compactor_serve(
    listen_addr: SocketAddr,
    client_addr: HostAddr,
    opts: CompactorOpts,
) -> (JoinHandle<()>, JoinHandle<()>, Sender<()>) {
    let config: CompactorConfig = load_config(&opts.config_path).unwrap();
    tracing::info!(
        "Starting compactor with config {:?} and opts {:?}",
        config,
        opts
    );

    // Register to the cluster.
    let meta_client =
        MetaClient::register_new(&opts.meta_address, WorkerType::Compactor, &client_addr, 0)
            .await
            .unwrap();
    tracing::info!("Assigned compactor id {}", meta_client.worker_id());
    meta_client.activate(&client_addr).await.unwrap();

    

    // Boot compactor
    let registry = prometheus::Registry::new();
    monitor_process(&registry).unwrap();
    let hummock_metrics = Arc::new(HummockMetrics::new(registry.clone()));
    let object_metrics = Arc::new(ObjectStoreMetrics::new(registry.clone()));
    let hummock_meta_client = Arc::new(MonitoredHummockMetaClient::new(
        meta_client.clone(),
        hummock_metrics.clone(),
    ));

    // use half of limit because any memory which would hold in meta-cache will be allocate by
    // limited at first.
    let storage_config = Arc::new(config.storage);
    let state_store_stats = Arc::new(StateStoreMetrics::new(registry.clone()));
    let object_store = Arc::new(
        parse_remote_object_store(
            opts.state_store
                .strip_prefix("hummock+")
                .expect("object store must be hummock for compactor server"),
            object_metrics,
        )
        .await,
    );
    let sstable_store = Arc::new(SstableStore::for_compactor(
        object_store,
        storage_config.data_directory.to_string(),
        1 << 20, // set 1MB memory to avoid panic.
        storage_config.meta_cache_capacity_mb * (1 << 20),
    ));

    let filter_key_extractor_manager = Arc::new(FilterKeyExtractorManager::default());
    let compactor_observer_node = CompactorObserverNode::new(filter_key_extractor_manager.clone());
    let observer_manager =
        ObserverManager::new_with_meta_client(meta_client.clone(), compactor_observer_node).await;

    let observer_join_handle = observer_manager.start().await.unwrap();
    let output_limit_mb = storage_config.compactor_memory_limit_mb as u64 / 2;
    let memory_limiter = Arc::new(MemoryLimiter::new(output_limit_mb << 20));
    let input_limit_mb = storage_config.compactor_memory_limit_mb as u64 / 2;
    let compact_sstable_store = Arc::new(CompactorSstableStore::new(
        sstable_store.clone(),
        Arc::new(MemoryLimiter::new(input_limit_mb << 20)),
    ));
    let memory_collector = Arc::new(CompactorMemoryCollector::new(
        memory_limiter.clone(),
        compact_sstable_store.clone(),
    ));
    monitor_cache(memory_collector, &registry).unwrap();
    let sstable_id_manager = Arc::new(SstableIdManager::new(
        hummock_meta_client.clone(),
        storage_config.sstable_id_remote_fetch_number,
    ));
    let context = Arc::new(Context {
        options: storage_config,
        meta_client: Arc::new(meta_client.clone()),
        hummock_meta_client: hummock_meta_client.clone(),
        sstable_store: sstable_store.clone(),
        stats: state_store_stats,
        is_share_buffer_compact: false,
        compaction_executor: Arc::new(CompactionExecutor::new(
            opts.compaction_worker_threads_number,
        )),
        filter_key_extractor_manager: filter_key_extractor_manager.clone(),
        read_memory_limiter: memory_limiter,
        sstable_id_manager: sstable_id_manager.clone(),
        task_progress_manager: Default::default(),
    });
    let compactor_context = Arc::new(CompactorContext {
        context,
        sstable_store: compact_sstable_store,
    });
    let sub_tasks = vec![
        MetaClient::start_heartbeat_loop(
            meta_client.clone(),
            Duration::from_millis(config.server.heartbeat_interval_ms as u64),
            vec![sstable_id_manager],
        ),
        piestream_storage::hummock::compactor::Compactor::start_compactor(
            compactor_context,
            hummock_meta_client,
            opts.max_concurrent_task_number,
        ),
    ];

    let (shutdown_send, mut shutdown_recv) = tokio::sync::oneshot::channel();
    let join_handle = tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(CompactorServiceServer::new(CompactorServiceImpl {}))
            .serve_with_shutdown(listen_addr, async move {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {},
                    _ = &mut shutdown_recv => {
                        for (join_handle, shutdown_sender) in sub_tasks {
                            if let Err(err) = shutdown_sender.send(()) {
                                tracing::warn!("Failed to send shutdown: {:?}", err);
                                continue;
                            }
                            if let Err(err) = join_handle.await {
                                tracing::warn!("Failed to join shutdown: {:?}", err);
                            }
                        }
                    },
                }
            })
            .await
            .unwrap();
    });

    // Boot metrics service.
    if opts.metrics_level > 0 {
        MetricsManager::boot_metrics_service(
            opts.prometheus_listener_addr.clone(),
            registry.clone(),
        );
    }

    (join_handle, observer_join_handle, shutdown_send)
}
