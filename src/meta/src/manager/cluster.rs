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

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use itertools::Itertools;
use piestream_common::types::ParallelUnitId;
use piestream_pb::common::worker_node::State;
use piestream_pb::common::{HostAddress, ParallelUnit, WorkerNode, WorkerType};
use piestream_pb::meta::heartbeat_request;
use piestream_pb::meta::subscribe_response::{Info, Operation};
use tokio::sync::oneshot::Sender;
use tokio::sync::{RwLock, RwLockReadGuard};
use tokio::task::JoinHandle;

use crate::manager::{IdCategory, LocalNotification, MetaSrvEnv};
use crate::model::{MetadataModel, Worker, INVALID_EXPIRE_AT};
use crate::rpc::metrics::MetaMetrics;
use crate::storage::MetaStore;
use crate::{MetaError, MetaResult};

pub type WorkerId = u32;
pub type WorkerLocations = HashMap<WorkerId, WorkerNode>;
pub type ClusterManagerRef<S> = Arc<ClusterManager<S>>;

#[derive(Clone, Debug)]
pub struct WorkerKey(pub HostAddress);

impl PartialEq<Self> for WorkerKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl Eq for WorkerKey {}

impl Hash for WorkerKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.host.hash(state);
        self.0.port.hash(state);
    }
}

/// The id preserved for the meta node. Note that there's no such entry in cluster manager.
pub const META_NODE_ID: u32 = 0;

/// [`ClusterManager`] manager cluster/worker meta data in [`MetaStore`].
pub struct ClusterManager<S: MetaStore> {
    env: MetaSrvEnv<S>,

    max_heartbeat_interval: Duration,

    core: RwLock<ClusterManagerCore>,
}

impl<S> ClusterManager<S>
where
    S: MetaStore,
{
    pub async fn new(env: MetaSrvEnv<S>, max_heartbeat_interval: Duration) -> MetaResult<Self> {
        let meta_store = env.meta_store_ref();
        let core = ClusterManagerCore::new(meta_store.clone()).await?;

        Ok(Self {
            env,
            max_heartbeat_interval,
            core: RwLock::new(core),
        })
    }

    /// Used in `NotificationService::subscribe`.
    /// Need to pay attention to the order of acquiring locks to prevent deadlock problems.
    pub async fn get_cluster_core_guard(&self) -> RwLockReadGuard<'_, ClusterManagerCore> {
        self.core.read().await
    }

    pub async fn start_worker_num_monitor(
        cluster_manager: ClusterManagerRef<S>,
        interval: Duration,
        meta_metrics: Arc<MetaMetrics>,
    ) -> (JoinHandle<()>, Sender<()>) {
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
        let join_handle = tokio::spawn(async move {
            let mut monitor_interval = tokio::time::interval(interval);
            monitor_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                tokio::select! {
                    // Wait for interval
                    _ = monitor_interval.tick() => {},
                    // Shutdown monitor
                    _ = &mut shutdown_rx => {
                        return;
                    }
                }

                for (worker_type, worker_num) in
                    cluster_manager.core.read().await.count_worker_node()
                {
                    meta_metrics
                        .worker_num
                        .with_label_values(&[(worker_type.as_str_name())])
                        .set(worker_num as i64);
                }
            }
        });

        (join_handle, shutdown_tx)
    }

    /// A worker node will immediately register itself to meta when it bootstraps.
    /// The meta will assign it with a unique ID and set its state as `Starting`.
    /// When the worker node is fully ready to serve, it will request meta again
    /// (via `activate_worker_node`) to set its state to `Running`.
    pub async fn add_worker_node(
        &self,
        r#type: WorkerType,
        host_address: HostAddress,
        worker_node_parallelism: usize,
    ) -> MetaResult<WorkerNode> {
        let mut core = self.core.write().await;
        match core.get_worker_by_host(host_address.clone()) {
            // TODO(zehua): update parallelism when the worker exists.
            Some(worker) => Ok(worker.to_protobuf()),
            None => {
                // Generate worker id.
                let worker_id = self
                    .env
                    .id_gen_manager()
                    .generate::<{ IdCategory::Worker }>()
                    .await? as WorkerId;

                // Generate parallel units.
                let parallel_units = self
                    .generate_cn_parallel_units(worker_node_parallelism, worker_id)
                    .await?;

                // Construct worker.
                let worker_node = WorkerNode {
                    id: worker_id,
                    r#type: r#type as i32,
                    host: Some(host_address.clone()),
                    state: State::Starting as i32,
                    parallel_units,
                };

                let worker = Worker::from_protobuf(worker_node.clone());

                // Persist worker node.
                worker.insert(self.env.meta_store()).await?;

                // Update core.
                core.add_worker_node(worker);

                Ok(worker_node)
            }
        }
    }

    pub async fn activate_worker_node(&self, host_address: HostAddress) -> MetaResult<()> {
        let mut core = self.core.write().await;
        let mut worker = core.get_worker_by_host_checked(host_address.clone())?;
        if worker.worker_node.state == State::Running as i32 {
            return Ok(());
        }
        worker.worker_node.state = State::Running as i32;
        worker.insert(self.env.meta_store()).await?;

        core.update_worker_node(worker.clone());

        // Notify frontends of new compute node.
        let worker_type = worker.worker_type();
        if worker_type == WorkerType::ComputeNode {
            self.env
                .notification_manager()
                .notify_frontend(Operation::Add, Info::Node(worker.worker_node))
                .await;
        }

        Ok(())
    }

    pub async fn delete_worker_node(&self, host_address: HostAddress) -> MetaResult<WorkerType> {
        let mut core = self.core.write().await;
        let worker = core.get_worker_by_host_checked(host_address.clone())?;
        let worker_type = worker.worker_type();
        let worker_node = worker.to_protobuf();

        // Persist deletion.
        Worker::delete(self.env.meta_store(), &host_address).await?;

        // Update core.
        core.delete_worker_node(worker);

        // Notify frontends to delete compute node.
        if worker_type == WorkerType::ComputeNode {
            self.env
                .notification_manager()
                .notify_frontend(Operation::Delete, Info::Node(worker_node.clone()))
                .await;
        }

        // Notify local subscribers.
        self.env
            .notification_manager()
            .notify_local_subscribers(LocalNotification::WorkerNodeIsDeleted(worker_node))
            .await;

        Ok(worker_type)
    }

    /// Invoked when it receives a heartbeat from a worker node.
    pub async fn heartbeat(
        &self,
        worker_id: WorkerId,
        info: Vec<heartbeat_request::extra_info::Info>,
    ) -> MetaResult<()> {
        tracing::trace!(target: "events::meta::server_heartbeat", worker_id = worker_id, "receive heartbeat");
        let mut core = self.core.write().await;
        for worker in core.workers.values_mut() {
            if worker.worker_id() == worker_id {
                worker.update_ttl(self.max_heartbeat_interval);
                worker.update_info(info);
                return Ok(());
            }
        }
        Err(MetaError::invalid_worker(worker_id))
    }

    pub async fn start_heartbeat_checker(
        cluster_manager: ClusterManagerRef<S>,
        check_interval: Duration,
    ) -> (JoinHandle<()>, Sender<()>) {
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
        let join_handle = tokio::spawn(async move {
            let mut min_interval = tokio::time::interval(check_interval);
            min_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                tokio::select! {
                    // Wait for interval
                    _ = min_interval.tick() => {},
                    // Shutdown
                    _ = &mut shutdown_rx => {
                        tracing::info!("Heartbeat checker is stopped");
                        return;
                    }
                }
                let (workers_to_delete, now) = {
                    let mut core = cluster_manager.core.write().await;
                    let workers = &mut core.workers;
                    // 1. Initialize new workers' TTL.
                    for worker in workers
                        .values_mut()
                        .filter(|worker| worker.expire_at() == INVALID_EXPIRE_AT)
                    {
                        worker.update_ttl(cluster_manager.max_heartbeat_interval);
                    }
                    // 2. Collect expired workers.
                    let now = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .expect("Clock may have gone backwards")
                        .as_secs();
                    (
                        workers
                            .values()
                            .filter(|worker| worker.expire_at() < now)
                            .map(|worker| (worker.worker_id(), worker.key().unwrap()))
                            .collect_vec(),
                        now,
                    )
                };
                // 3. Delete expired workers.
                for (worker_id, key) in workers_to_delete {
                    match cluster_manager.delete_worker_node(key.clone()).await {
                        Ok(worker_type) => {
                            match worker_type {
                                WorkerType::Frontend
                                | WorkerType::ComputeNode
                                | WorkerType::Compactor
                                | WorkerType::RiseCtl => {
                                    cluster_manager
                                        .env
                                        .notification_manager()
                                        .delete_sender(worker_type, WorkerKey(key.clone()))
                                        .await
                                }
                                _ => {}
                            };
                            tracing::warn!(
                                "Deleted expired worker {} {:#?}, current timestamp {}",
                                worker_id,
                                key,
                                now,
                            );
                        }
                        Err(err) => {
                            tracing::warn!(
                                "Failed to delete expired worker {} {:#?}, current timestamp {}. {:?}",
                                worker_id,
                                key,
                                now,
                                err,
                            );
                        }
                    }
                }
            }
        });
        (join_handle, shutdown_tx)
    }

    /// Get live nodes with the specified type and state.
    /// # Arguments
    /// * `worker_type` `WorkerType` of the nodes
    /// * `worker_state` Filter by this state if it is not None.
    pub async fn list_worker_node(
        &self,
        worker_type: WorkerType,
        worker_state: Option<State>,
    ) -> Vec<WorkerNode> {
        let core = self.core.read().await;
        core.list_worker_node(worker_type, worker_state)
    }

    pub async fn list_active_parallel_units(&self) -> Vec<ParallelUnit> {
        let core = self.core.read().await;
        core.list_active_parallel_units()
    }

    pub async fn get_active_parallel_unit_count(&self) -> usize {
        let core = self.core.read().await;
        core.get_active_parallel_unit_count()
    }

    /// Generate `parallel_degree` parallel units.
    async fn generate_cn_parallel_units(
        &self,
        parallel_degree: usize,
        worker_id: WorkerId,
    ) -> MetaResult<Vec<ParallelUnit>> {
        let start_id = self
            .env
            .id_gen_manager()
            .generate_interval::<{ IdCategory::ParallelUnit }>(parallel_degree as u64)
            .await? as ParallelUnitId;
        let parallel_units = (start_id..start_id + parallel_degree as ParallelUnitId)
            .map(|id| ParallelUnit {
                id,
                worker_node_id: worker_id,
            })
            .collect();
        Ok(parallel_units)
    }

    pub async fn get_worker_by_id(&self, worker_id: WorkerId) -> Option<Worker> {
        self.core.read().await.get_worker_by_id(worker_id)
    }
}

pub struct ClusterManagerCore {
    /// Record for workers in the cluster.
    workers: HashMap<WorkerKey, Worker>,

    /// Record for parallel units.
    parallel_units: Vec<ParallelUnit>,
}

impl ClusterManagerCore {
    async fn new<S>(meta_store: Arc<S>) -> MetaResult<Self>
    where
        S: MetaStore,
    {
        let workers = Worker::list(&*meta_store).await?;
        let mut worker_map = HashMap::new();
        let mut parallel_units = Vec::new();

        workers.into_iter().for_each(|w| {
            worker_map.insert(WorkerKey(w.key().unwrap()), w.clone());
            parallel_units.extend(w.worker_node.parallel_units);
        });

        Ok(Self {
            workers: worker_map,
            parallel_units,
        })
    }

    /// If no worker exists, return an error.
    fn get_worker_by_host_checked(&self, host_address: HostAddress) -> MetaResult<Worker> {
        self.get_worker_by_host(host_address)
            .ok_or_else(|| anyhow::anyhow!("Worker node does not exist!").into())
    }

    pub fn get_worker_by_host(&self, host_address: HostAddress) -> Option<Worker> {
        self.workers.get(&WorkerKey(host_address)).cloned()
    }

    fn get_worker_by_id(&self, id: WorkerId) -> Option<Worker> {
        self.workers
            .iter()
            .find(|(_, worker)| worker.worker_id() == id)
            .map(|(_, worker)| worker.clone())
    }

    fn add_worker_node(&mut self, worker: Worker) {
        self.parallel_units
            .extend(worker.worker_node.parallel_units.clone());
        self.workers
            .insert(WorkerKey(worker.key().unwrap()), worker);
    }

    fn update_worker_node(&mut self, worker: Worker) {
        self.workers
            .insert(WorkerKey(worker.key().unwrap()), worker);
    }

    fn delete_worker_node(&mut self, worker: Worker) {
        worker
            .worker_node
            .parallel_units
            .iter()
            .for_each(|parallel_unit| {
                self.parallel_units.retain(|p| p.id != parallel_unit.id);
            });
        self.workers.remove(&WorkerKey(worker.key().unwrap()));
    }

    pub fn list_worker_node(
        &self,
        worker_type: WorkerType,
        worker_state: Option<State>,
    ) -> Vec<WorkerNode> {
        self.workers
            .iter()
            .map(|(_, worker)| worker.to_protobuf())
            .filter(|w| w.r#type == worker_type as i32)
            .filter(|w| match worker_state {
                None => true,
                Some(state) => state as i32 == w.state,
            })
            .collect_vec()
    }

    fn list_active_parallel_units(&self) -> Vec<ParallelUnit> {
        let active_workers: HashSet<_> = self
            .list_worker_node(WorkerType::ComputeNode, Some(State::Running))
            .into_iter()
            .map(|w| w.id)
            .collect();

        self.parallel_units
            .iter()
            .filter(|p| active_workers.contains(&p.worker_node_id))
            .cloned()
            .collect()
    }

    fn count_worker_node(&self) -> HashMap<WorkerType, u64> {
        const MONITORED_WORKER_TYPES: [WorkerType; 3] = [
            WorkerType::Compactor,
            WorkerType::ComputeNode,
            WorkerType::Frontend,
        ];
        let mut ret = HashMap::new();
        self.workers
            .iter()
            .map(|(_, worker)| worker.worker_type())
            .filter(|worker_type| MONITORED_WORKER_TYPES.contains(worker_type))
            .for_each(|worker_type| {
                ret.entry(worker_type)
                    .and_modify(|worker_num| *worker_num += 1)
                    .or_insert(1);
            });
        // Make sure all the monitored worker types exist in the map.
        for wt in MONITORED_WORKER_TYPES {
            ret.entry(wt).or_insert(0);
        }
        ret
    }

    fn get_active_parallel_unit_count(&self) -> usize {
        self.list_active_parallel_units().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemStore;

    #[tokio::test]
    async fn test_cluster_manager() -> MetaResult<()> {
        let env = MetaSrvEnv::for_test().await;

        let cluster_manager = Arc::new(
            ClusterManager::new(env.clone(), Duration::new(0, 0))
                .await
                .unwrap(),
        );

        let mut worker_nodes = Vec::new();
        let worker_count = 5usize;
        let fake_parallelism = 4;
        for i in 0..worker_count {
            let fake_host_address = HostAddress {
                host: "localhost".to_string(),
                port: 5000 + i as i32,
            };
            let worker_node = cluster_manager
                .add_worker_node(WorkerType::ComputeNode, fake_host_address, fake_parallelism)
                .await
                .unwrap();
            worker_nodes.push(worker_node);
        }

        // Since no worker is active, the parallel unit count should be 0.
        assert_cluster_manager(&cluster_manager, 0).await;

        for worker_node in worker_nodes {
            cluster_manager
                .activate_worker_node(worker_node.get_host().unwrap().clone())
                .await
                .unwrap();
        }

        let worker_count_map = cluster_manager.core.read().await.count_worker_node();
        assert_eq!(
            *worker_count_map.get(&WorkerType::ComputeNode).unwrap() as usize,
            worker_count
        );

        let parallel_count = fake_parallelism * worker_count;
        assert_cluster_manager(&cluster_manager, parallel_count).await;

        let worker_to_delete_count = 4usize;
        for i in 0..worker_to_delete_count {
            let fake_host_address = HostAddress {
                host: "localhost".to_string(),
                port: 5000 + i as i32,
            };
            cluster_manager
                .delete_worker_node(fake_host_address)
                .await
                .unwrap();
        }
        assert_cluster_manager(&cluster_manager, fake_parallelism).await;

        Ok(())
    }

    async fn assert_cluster_manager(
        cluster_manager: &ClusterManager<MemStore>,
        parallel_count: usize,
    ) {
        let parallel_units = cluster_manager.list_active_parallel_units().await;
        assert_eq!(parallel_units.len(), parallel_count);
    }

    // This test takes seconds because the TTL is measured in seconds.
    #[cfg(madsim)]
    #[tokio::test]
    async fn test_heartbeat() {
        use crate::hummock::test_utils::setup_compute_env;
        let (_env, _hummock_manager, cluster_manager, worker_node) = setup_compute_env(1).await;
        let context_id_1 = worker_node.id;
        let fake_host_address_2 = HostAddress {
            host: "127.0.0.1".to_string(),
            port: 2,
        };
        let fake_parallelism = 4;
        let _worker_node_2 = cluster_manager
            .add_worker_node(
                WorkerType::ComputeNode,
                fake_host_address_2,
                fake_parallelism,
            )
            .await
            .unwrap();
        // Two live nodes
        assert_eq!(
            cluster_manager
                .list_worker_node(WorkerType::ComputeNode, None)
                .await
                .len(),
            2
        );

        let ttl = cluster_manager.max_heartbeat_interval;
        let check_interval = std::cmp::min(Duration::from_millis(100), ttl / 4);

        // Keep worker 1 alive
        let cluster_manager_ref = cluster_manager.clone();
        let keep_alive_join_handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(cluster_manager_ref.max_heartbeat_interval / 3).await;
                cluster_manager_ref
                    .heartbeat(context_id_1, vec![])
                    .await
                    .unwrap();
            }
        });

        tokio::time::sleep(ttl * 2 + check_interval).await;

        // One node has actually expired but still got two, because heartbeat check is not
        // started.
        assert_eq!(
            cluster_manager
                .list_worker_node(WorkerType::ComputeNode, None)
                .await
                .len(),
            2
        );

        let (join_handle, shutdown_sender) =
            ClusterManager::start_heartbeat_checker(cluster_manager.clone(), check_interval).await;
        tokio::time::sleep(ttl * 2 + check_interval).await;

        // One live node left.
        assert_eq!(
            cluster_manager
                .list_worker_node(WorkerType::ComputeNode, None)
                .await
                .len(),
            1
        );

        shutdown_sender.send(()).unwrap();
        join_handle.await.unwrap();
        keep_alive_join_handle.abort();
    }
}
