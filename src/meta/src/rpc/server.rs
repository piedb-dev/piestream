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
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use etcd_client::{Client as EtcdClient, ConnectOptions};
use prost::Message;
use piestream_common::bail;
use piestream_common::monitor::process_linux::monitor_process;
use piestream_common_service::metrics_manager::MetricsManager;
use piestream_pb::ddl_service::ddl_service_server::DdlServiceServer;
use piestream_pb::health::health_server::HealthServer;
use piestream_pb::hummock::hummock_manager_service_server::HummockManagerServiceServer;
use piestream_pb::meta::cluster_service_server::ClusterServiceServer;
use piestream_pb::meta::heartbeat_service_server::HeartbeatServiceServer;
use piestream_pb::meta::notification_service_server::NotificationServiceServer;
use piestream_pb::meta::scale_service_server::ScaleServiceServer;
use piestream_pb::meta::stream_manager_service_server::StreamManagerServiceServer;
use piestream_pb::meta::{MetaLeaderInfo, MetaLeaseInfo};
use piestream_pb::user::user_service_server::UserServiceServer;
use tokio::sync::oneshot::Sender;
use tokio::task::JoinHandle;

use super::intercept::MetricsMiddlewareLayer;
use super::service::health_service::HealthServiceImpl;
use super::service::notification_service::NotificationServiceImpl;
use super::service::scale_service::ScaleServiceImpl;
use super::DdlServiceImpl;
use crate::barrier::{BarrierScheduler, GlobalBarrierManager};
use crate::hummock::compaction_group::manager::CompactionGroupManager;
use crate::hummock::{CompactionScheduler, HummockManager};
use crate::manager::{
    CatalogManager, ClusterManager, FragmentManager, IdleManager, MetaOpts, MetaSrvEnv,
    StreamingJobBackgroundDeleter,
};
use crate::rpc::metrics::MetaMetrics;
use crate::rpc::service::cluster_service::ClusterServiceImpl;
use crate::rpc::service::heartbeat_service::HeartbeatServiceImpl;
use crate::rpc::service::hummock_service::HummockServiceImpl;
use crate::rpc::service::stream_service::StreamServiceImpl;
use crate::rpc::service::user_service::UserServiceImpl;
use crate::rpc::{META_CF_NAME, META_LEADER_KEY, META_LEASE_KEY};
use crate::storage::{EtcdMetaStore, MemStore, MetaStore, MetaStoreError, Transaction};
use crate::stream::{GlobalStreamManager, SourceManager};
use crate::{hummock, MetaResult};

#[derive(Debug)]
pub enum MetaStoreBackend {
    Etcd {
        endpoints: Vec<String>,
        credentials: Option<(String, String)>,
    },
    Mem,
}

#[derive(Clone)]
pub struct AddressInfo {
    pub addr: String,
    pub listen_addr: SocketAddr,
    pub prometheus_addr: Option<SocketAddr>,
    pub dashboard_addr: Option<SocketAddr>,
    pub ui_path: Option<String>,
}

impl Default for AddressInfo {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:0000".to_string(),
            listen_addr: SocketAddr::V4("127.0.0.1:0000".parse().unwrap()),
            prometheus_addr: None,
            dashboard_addr: None,
            ui_path: None,
        }
    }
}

pub async fn rpc_serve(
    address_info: AddressInfo,
    meta_store_backend: MetaStoreBackend,
    max_heartbeat_interval: Duration,
    lease_interval_secs: u64,
    opts: MetaOpts,
) -> MetaResult<(JoinHandle<()>, Sender<()>)> {
    match meta_store_backend {
        MetaStoreBackend::Etcd {
            endpoints,
            credentials,
        } => {
            let mut options = ConnectOptions::default()
                .with_keep_alive(Duration::from_secs(3), Duration::from_secs(5));
            if let Some((username, password)) = credentials {
                options = options.with_user(username, password)
            }
            let client = EtcdClient::connect(endpoints, Some(options))
                .await
                .map_err(|e| anyhow::anyhow!("failed to connect etcd {}", e))?;
            let meta_store = Arc::new(EtcdMetaStore::new(client));
            rpc_serve_with_store(
                meta_store,
                address_info,
                max_heartbeat_interval,
                lease_interval_secs,
                opts,
            )
            .await
        }
        MetaStoreBackend::Mem => {
            let meta_store = Arc::new(MemStore::new());
            rpc_serve_with_store(
                meta_store,
                address_info,
                max_heartbeat_interval,
                lease_interval_secs,
                opts,
            )
            .await
        }
    }
}

pub async fn register_leader_for_meta<S: MetaStore>(
    addr: String,
    meta_store: Arc<S>,
    lease_time: u64,
) -> MetaResult<(MetaLeaderInfo, JoinHandle<()>, Sender<()>)> {
    let mut tick_interval = tokio::time::interval(Duration::from_secs(lease_time / 2));
    loop {
        tick_interval.tick().await;
        let old_leader_info = match meta_store
            .get_cf(META_CF_NAME, META_LEADER_KEY.as_bytes())
            .await
        {
            Err(MetaStoreError::ItemNotFound(_)) => vec![],
            Ok(v) => v,
            _ => {
                continue;
            }
        };
        let old_leader_lease = match meta_store
            .get_cf(META_CF_NAME, META_LEASE_KEY.as_bytes())
            .await
        {
            Err(MetaStoreError::ItemNotFound(_)) => vec![],
            Ok(v) => v,
            _ => {
                continue;
            }
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        if !old_leader_lease.is_empty() {
            let lease_info = MetaLeaseInfo::decode(&mut old_leader_lease.as_slice()).unwrap();

            if lease_info.lease_expire_time > now.as_secs()
                && lease_info.leader.as_ref().unwrap().node_address != addr
            {
                let err_info = format!(
                    "the lease {:?} does not expire, now time: {}",
                    lease_info,
                    now.as_secs(),
                );
                tracing::error!("{}", err_info);
                bail!(err_info);
            }
        }
        let lease_id = if !old_leader_info.is_empty() {
            let leader_info = MetaLeaderInfo::decode(&mut old_leader_info.as_slice()).unwrap();
            leader_info.lease_id + 1
        } else {
            0
        };
        let mut txn = Transaction::default();
        let leader_info = MetaLeaderInfo {
            lease_id,
            node_address: addr.to_string(),
        };
        let lease_info = MetaLeaseInfo {
            leader: Some(leader_info.clone()),
            lease_register_time: now.as_secs(),
            lease_expire_time: now.as_secs() + lease_time,
        };

        if !old_leader_info.is_empty() {
            txn.check_equal(
                META_CF_NAME.to_string(),
                META_LEADER_KEY.as_bytes().to_vec(),
                old_leader_info,
            );
            txn.put(
                META_CF_NAME.to_string(),
                META_LEADER_KEY.as_bytes().to_vec(),
                leader_info.encode_to_vec(),
            );
        } else {
            if let Err(e) = meta_store
                .put_cf(
                    META_CF_NAME,
                    META_LEADER_KEY.as_bytes().to_vec(),
                    leader_info.encode_to_vec(),
                )
                .await
            {
                tracing::warn!(
                    "new cluster put leader info failed, MetaStoreError: {:?}",
                    e
                );
                continue;
            }
            txn.check_equal(
                META_CF_NAME.to_string(),
                META_LEADER_KEY.as_bytes().to_vec(),
                leader_info.encode_to_vec(),
            );
        }
        txn.put(
            META_CF_NAME.to_string(),
            META_LEASE_KEY.as_bytes().to_vec(),
            lease_info.encode_to_vec(),
        );
        if let Err(e) = meta_store.txn(txn).await {
            tracing::warn!(
                "add leader info failed, MetaStoreError: {:?}, try again later",
                e
            );
            continue;
        }
        let leader = leader_info.clone();
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(lease_time / 2));
            loop {
                let mut txn = Transaction::default();
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards");
                let lease_info = MetaLeaseInfo {
                    leader: Some(leader_info.clone()),
                    lease_register_time: now.as_secs(),
                    lease_expire_time: now.as_secs() + lease_time,
                };
                txn.check_equal(
                    META_CF_NAME.to_string(),
                    META_LEADER_KEY.as_bytes().to_vec(),
                    leader_info.encode_to_vec(),
                );
                txn.put(
                    META_CF_NAME.to_string(),
                    META_LEASE_KEY.as_bytes().to_vec(),
                    lease_info.encode_to_vec(),
                );
                if let Err(e) = meta_store.txn(txn).await {
                    match e {
                        MetaStoreError::TransactionAbort() => {
                            panic!("keep lease failed, another node has become new leader");
                        }
                        MetaStoreError::Internal(e) => {
                            tracing::warn!(
                                "keep lease failed, try again later, MetaStoreError: {:?}",
                                e
                            );
                        }
                        MetaStoreError::ItemNotFound(e) => {
                            tracing::warn!("keep lease failed, MetaStoreError: {:?}", e);
                        }
                    }
                }
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        tracing::info!("Register leader info is stopped");
                        return;
                    }
                    // Wait for the minimal interval,
                    _ = ticker.tick() => {},
                }
            }
        });
        return Ok((leader, handle, shutdown_tx));
    }
}

pub async fn rpc_serve_with_store<S: MetaStore>(
    meta_store: Arc<S>,
    mut address_info: AddressInfo,
    max_heartbeat_interval: Duration,
    lease_interval_secs: u64,
    opts: MetaOpts,
) -> MetaResult<(JoinHandle<()>, Sender<()>)> {
    // Initialize managers.
    let (info, lease_handle, lease_shutdown) = register_leader_for_meta(
        address_info.addr.clone(),
        meta_store.clone(),
        lease_interval_secs,
    )
    .await?;
    let env = MetaSrvEnv::<S>::new(opts, meta_store.clone(), info).await;
    let compaction_group_manager =
        Arc::new(CompactionGroupManager::new(env.clone()).await.unwrap());
    let fragment_manager = Arc::new(FragmentManager::new(env.clone()).await.unwrap());
    let meta_metrics = Arc::new(MetaMetrics::new());
    let registry = meta_metrics.registry();
    monitor_process(registry).unwrap();
    let compactor_manager = Arc::new(
        hummock::CompactorManager::with_meta(env.clone(), max_heartbeat_interval.as_secs())
            .await
            .unwrap(),
    );

    let cluster_manager = Arc::new(
        ClusterManager::new(env.clone(), max_heartbeat_interval)
            .await
            .unwrap(),
    );
    let hummock_manager = Arc::new(
        hummock::HummockManager::new(
            env.clone(),
            cluster_manager.clone(),
            meta_metrics.clone(),
            compaction_group_manager.clone(),
            compactor_manager.clone(),
        )
        .await
        .unwrap(),
    );

    #[cfg(not(madsim))]
    if let Some(dashboard_addr) = address_info.dashboard_addr.take() {
        let dashboard_service = crate::dashboard::DashboardService {
            dashboard_addr,
            cluster_manager: cluster_manager.clone(),
            fragment_manager: fragment_manager.clone(),
            meta_store: env.meta_store_ref(),
        };
        // TODO: join dashboard service back to local thread.
        tokio::spawn(dashboard_service.serve(address_info.ui_path));
    }

    let catalog_manager = Arc::new(CatalogManager::new(env.clone()).await.unwrap());

    let (barrier_scheduler, scheduled_barriers) =
        BarrierScheduler::new_pair(hummock_manager.clone(), env.opts.checkpoint_frequency);

    let source_manager = Arc::new(
        SourceManager::new(
            env.clone(),
            barrier_scheduler.clone(),
            catalog_manager.clone(),
            fragment_manager.clone(),
        )
        .await
        .unwrap(),
    );

    let barrier_manager = Arc::new(GlobalBarrierManager::new(
        scheduled_barriers,
        env.clone(),
        cluster_manager.clone(),
        catalog_manager.clone(),
        fragment_manager.clone(),
        hummock_manager.clone(),
        source_manager.clone(),
        meta_metrics.clone(),
    ));

    {
        let source_manager = source_manager.clone();
        tokio::spawn(async move {
            source_manager.run().await.unwrap();
        });
    }

    let stream_manager = Arc::new(
        GlobalStreamManager::new(
            env.clone(),
            fragment_manager.clone(),
            barrier_scheduler.clone(),
            cluster_manager.clone(),
            source_manager.clone(),
            compaction_group_manager.clone(),
        )
        .unwrap(),
    );

    let (table_background_deleter, deleter_handle, deleter_shutdown) =
        StreamingJobBackgroundDeleter::new(stream_manager.clone(), source_manager.clone())
            .await
            .unwrap();
    let table_background_deleter = Arc::new(table_background_deleter);

    compaction_group_manager
        .purge_stale_members(
            &fragment_manager
                .list_table_fragments()
                .await
                .expect("list_table_fragments"),
        )
        .await
        .unwrap();

    // Initialize services.
    let vacuum_trigger = Arc::new(hummock::VacuumManager::new(
        env.clone(),
        hummock_manager.clone(),
        compactor_manager.clone(),
    ));

    let heartbeat_srv = HeartbeatServiceImpl::new(cluster_manager.clone());
    let ddl_srv = DdlServiceImpl::<S>::new(
        env.clone(),
        catalog_manager.clone(),
        stream_manager.clone(),
        source_manager.clone(),
        cluster_manager.clone(),
        fragment_manager.clone(),
        table_background_deleter,
    );

    let user_srv = UserServiceImpl::<S>::new(env.clone(), catalog_manager.clone());

    let scale_srv = ScaleServiceImpl::<S>::new(
        barrier_scheduler.clone(),
        fragment_manager.clone(),
        cluster_manager.clone(),
        source_manager,
        catalog_manager.clone(),
        stream_manager.clone(),
    );

    let cluster_srv = ClusterServiceImpl::<S>::new(cluster_manager.clone());
    let stream_srv = StreamServiceImpl::<S>::new(
        env.clone(),
        barrier_scheduler.clone(),
        fragment_manager.clone(),
    );
    let hummock_srv = HummockServiceImpl::new(
        hummock_manager.clone(),
        compactor_manager.clone(),
        vacuum_trigger.clone(),
        compaction_group_manager.clone(),
        fragment_manager.clone(),
    );
    let notification_manager = env.notification_manager_ref();
    let notification_srv = NotificationServiceImpl::new(
        env.clone(),
        catalog_manager,
        cluster_manager.clone(),
        hummock_manager.clone(),
        fragment_manager.clone(),
    );
    let health_srv = HealthServiceImpl::new();

    if let Some(prometheus_addr) = address_info.prometheus_addr {
        MetricsManager::boot_metrics_service(
            prometheus_addr.to_string(),
            meta_metrics.registry().clone(),
        )
    }

    // Initialize sub-tasks.
    let compaction_scheduler = Arc::new(CompactionScheduler::new(
        env.clone(),
        hummock_manager.clone(),
        compactor_manager.clone(),
    ));
    let mut sub_tasks = hummock::start_hummock_workers(
        hummock_manager.clone(),
        compactor_manager,
        vacuum_trigger,
        notification_manager,
        compaction_scheduler,
        &env.opts,
    )
    .await;
    sub_tasks.push(
        ClusterManager::start_worker_num_monitor(
            cluster_manager.clone(),
            Duration::from_secs(env.opts.node_num_monitor_interval_sec),
            meta_metrics.clone(),
        )
        .await,
    );
    sub_tasks.push(HummockManager::start_compaction_heartbeat(hummock_manager).await);
    sub_tasks.push((lease_handle, lease_shutdown));
    sub_tasks.push((deleter_handle, deleter_shutdown));
    if cfg!(not(test)) {
        sub_tasks.push(
            ClusterManager::start_heartbeat_checker(cluster_manager, Duration::from_secs(1)).await,
        );
        sub_tasks.push(GlobalBarrierManager::start(barrier_manager).await);
    }

    let (idle_send, mut idle_recv) = tokio::sync::oneshot::channel();
    sub_tasks.push(
        IdleManager::start_idle_checker(env.idle_manager_ref(), Duration::from_secs(30), idle_send)
            .await,
    );

    let shutdown_all = async move {
        for (join_handle, shutdown_sender) in sub_tasks {
            if let Err(_err) = shutdown_sender.send(()) {
                // Maybe it is already shut down
                continue;
            }
            if let Err(err) = join_handle.await {
                tracing::warn!("Failed to join shutdown: {:?}", err);
            }
        }
    };

    // Start services.
    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .layer(MetricsMiddlewareLayer::new(meta_metrics.clone()))
            .add_service(HeartbeatServiceServer::new(heartbeat_srv))
            .add_service(ClusterServiceServer::new(cluster_srv))
            .add_service(StreamManagerServiceServer::new(stream_srv))
            .add_service(HummockManagerServiceServer::new(hummock_srv))
            .add_service(NotificationServiceServer::new(notification_srv))
            .add_service(DdlServiceServer::new(ddl_srv))
            .add_service(UserServiceServer::new(user_srv))
            .add_service(ScaleServiceServer::new(scale_srv))
            .add_service(HealthServer::new(health_srv))
            .serve(address_info.listen_addr)
            .await
            .unwrap();
    });

    // TODO: Use tonic's serve_with_shutdown for a graceful shutdown. Now it does not work,
    // as the graceful shutdown waits all connections to disconnect in order to finish the stop.
    let (shutdown_send, mut shutdown_recv) = tokio::sync::oneshot::channel();
    let join_handle = tokio::spawn(async move {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = &mut shutdown_recv => {
                shutdown_all.await;
            },
            _ = &mut idle_recv => {
                shutdown_all.await;
            },
        }
    });

    Ok((join_handle, shutdown_send))
}

#[cfg(test)]
mod tests {
    use tokio::time::sleep;

    use super::*;

    #[tokio::test]
    async fn test_leader_lease() {
        let info = AddressInfo {
            addr: "node1".to_string(),
            ..Default::default()
        };
        let meta_store = Arc::new(MemStore::default());
        let (handle, closer) = rpc_serve_with_store(
            meta_store.clone(),
            info,
            Duration::from_secs(10),
            2,
            MetaOpts::default(),
        )
        .await
        .unwrap();
        sleep(Duration::from_secs(4)).await;
        let info2 = AddressInfo {
            addr: "node2".to_string(),
            ..Default::default()
        };
        let ret = rpc_serve_with_store(
            meta_store.clone(),
            info2.clone(),
            Duration::from_secs(10),
            2,
            MetaOpts::default(),
        )
        .await;
        assert!(ret.is_err());
        closer.send(()).unwrap();
        handle.await.unwrap();
        sleep(Duration::from_secs(3)).await;
        rpc_serve_with_store(
            meta_store.clone(),
            info2,
            Duration::from_secs(10),
            2,
            MetaOpts::default(),
        )
        .await
        .unwrap();
    }
}
