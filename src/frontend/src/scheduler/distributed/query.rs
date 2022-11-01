// Copyright 2022 Piedb Data
// Licensed under the Apache License, Version 2.0 (the "License");
//
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

use std::collections::HashMap;
use std::default::Default;
use std::mem;
use std::sync::Arc;

use anyhow::anyhow;
use pgwire::pg_server::SessionId;
use piestream_common::array::DataChunk;
use piestream_pb::batch_plan::{TaskId as TaskIdProst, TaskOutputId as TaskOutputIdProst};
use piestream_pb::common::HostAddress;
use piestream_rpc_client::ComputeClientPoolRef;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::{oneshot, RwLock};
use tracing::{debug, error, info, warn};

use super::{QueryResultFetcher, StageEvent};
use crate::catalog::catalog_service::CatalogReader;
use crate::scheduler::distributed::query::QueryMessage::Stage;
use crate::scheduler::distributed::stage::StageEvent::ScheduledRoot;
use crate::scheduler::distributed::StageEvent::Scheduled;
use crate::scheduler::distributed::StageExecution;
use crate::scheduler::plan_fragmenter::{Query, StageId, ROOT_TASK_ID, ROOT_TASK_OUTPUT_ID};
use crate::scheduler::worker_node_manager::WorkerNodeManagerRef;
use crate::scheduler::{
    ExecutionContextRef, HummockSnapshotManagerRef, SchedulerError, SchedulerResult,
};

/// Message sent to a `QueryRunner` to control its execution.
#[derive(Debug)]
pub enum QueryMessage {
    /// Events passed running execution.
    Stage(StageEvent),
    CancelQuery,
}

enum QueryState {
    /// Not scheduled yet.
    ///
    /// In this state, some data structures for starting executions are created to avoid holding
    /// them `QueryExecution`
    Pending {
        msg_receiver: Receiver<QueryMessage>,
    },

    /// Running
    Running,

    /// Failed
    Failed,

    /// Completed
    Completed,
}

pub struct QueryExecution {
    query: Arc<Query>,
    state: Arc<RwLock<QueryState>>,

    /// These fields are just used for passing to Query Runner.
    epoch: u64,
    stage_executions: Arc<HashMap<StageId, Arc<StageExecution>>>,
    hummock_snapshot_manager: HummockSnapshotManagerRef,
    compute_client_pool: ComputeClientPoolRef,

    shutdown_tx: Sender<QueryMessage>,

    /// Identified by process_id, secret_key. Query in the same session should have same key.
    pub session_id: SessionId,
}

struct QueryRunner {
    query: Arc<Query>,
    stage_executions: Arc<HashMap<StageId, Arc<StageExecution>>>,
    scheduled_stages_count: usize,
    /// Query messages receiver. For example, stage state change events, query commands.
    msg_receiver: Receiver<QueryMessage>,

    /// Will be set to `None` after all stage scheduled.
    root_stage_sender: Option<oneshot::Sender<SchedulerResult<QueryResultFetcher>>>,

    epoch: u64,
    hummock_snapshot_manager: HummockSnapshotManagerRef,
    compute_client_pool: ComputeClientPoolRef,
}

impl QueryExecution {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        context: ExecutionContextRef,
        query: Query,
        epoch: u64,
        worker_node_manager: WorkerNodeManagerRef,
        hummock_snapshot_manager: HummockSnapshotManagerRef,
        compute_client_pool: ComputeClientPoolRef,
        catalog_reader: CatalogReader,
        session_id: SessionId,
    ) -> Self {
        let query = Arc::new(query);
        let (sender, receiver) = channel(100);
        let stage_executions = {
            let mut stage_executions: HashMap<StageId, Arc<StageExecution>> =
                HashMap::with_capacity(query.stage_graph.stages.len());

            for stage_id in query.stage_graph.stage_ids_by_topo_order() {
                let children_stages = query
                    .stage_graph
                    .get_child_stages_unchecked(&stage_id)
                    .iter()
                    .map(|s| stage_executions[s].clone())
                    .collect::<Vec<Arc<StageExecution>>>();

                let stage_exec = Arc::new(StageExecution::new(
                    epoch,
                    query.stage_graph.stages[&stage_id].clone(),
                    worker_node_manager.clone(),
                    sender.clone(),
                    children_stages,
                    compute_client_pool.clone(),
                    catalog_reader.clone(),
                    context.clone(),
                ));
                stage_executions.insert(stage_id, stage_exec);
            }
            Arc::new(stage_executions)
        };

        let state = QueryState::Pending {
            msg_receiver: receiver,
        };

        Self {
            query,
            state: Arc::new(RwLock::new(state)),
            stage_executions,
            epoch,
            compute_client_pool,
            hummock_snapshot_manager,
            shutdown_tx: sender,
            session_id,
        }
    }

    /// Start execution of this query.
    /// Note the two shutdown channel sender and receivers are not dual.
    /// One is used for propagate error to `QueryResultFetcher`, one is used for listening on
    /// cancel request (from ctrl-c, cli, ui etc).
    pub async fn start(&self) -> SchedulerResult<QueryResultFetcher> {
        let mut state = self.state.write().await;
        let cur_state = mem::replace(&mut *state, QueryState::Failed);

        match cur_state {
            QueryState::Pending { msg_receiver } => {
                *state = QueryState::Running;

                let (root_stage_sender, root_stage_receiver) =
                    oneshot::channel::<SchedulerResult<QueryResultFetcher>>();

                let runner = QueryRunner {
                    query: self.query.clone(),
                    stage_executions: self.stage_executions.clone(),
                    msg_receiver,
                    root_stage_sender: Some(root_stage_sender),
                    scheduled_stages_count: 0,
                    epoch: self.epoch,
                    hummock_snapshot_manager: self.hummock_snapshot_manager.clone(),
                    compute_client_pool: self.compute_client_pool.clone(),
                };

                // Not trace the error here, it will be processed in scheduler.
                tokio::spawn(async move { runner.run().await });

                let root_stage = root_stage_receiver
                    .await
                    .map_err(|e| anyhow!("Starting query execution failed: {:?}", e))??;

                tracing::trace!(
                    "Received root stage query result fetcher: {:?}, query id: {:?}",
                    root_stage,
                    self.query.query_id
                );

                Ok(root_stage)
            }
            _ => {
                unreachable!("The query runner should not be scheduled twice");
            }
        }
    }

    /// Cancel execution of this query.
    pub async fn abort(self: Arc<Self>) {
        if self
            .shutdown_tx
            .send(QueryMessage::CancelQuery)
            .await
            .is_err()
        {
            warn!("Send cancel query request failed: the query has ended");
        } else {
            info!("Send cancel request to query-{:?}", self.query.query_id);
        };
    }
}

impl QueryRunner {
    async fn run(mut self) {
        // Start leaf stages.
        let leaf_stages = self.query.leaf_stages();
        for stage_id in &leaf_stages {
            self.stage_executions[stage_id].start().await;
            tracing::trace!(
                "Query stage {:?}-{:?} started.",
                self.query.query_id,
                stage_id
            );
        }
        let mut stages_with_table_scan = self.query.stages_with_table_scan();

        while let Some(msg_inner) = self.msg_receiver.recv().await {
            match msg_inner {
                Stage(Scheduled(stage_id)) => {
                    tracing::trace!(
                        "Query stage {:?}-{:?} scheduled.",
                        self.query.query_id,
                        stage_id
                    );
                    self.scheduled_stages_count += 1;
                    stages_with_table_scan.remove(&stage_id);
                    if stages_with_table_scan.is_empty() {
                        // We can be sure here that all the Hummock iterators have been created,
                        // thus they all successfully pinned a HummockVersion.
                        // So we can now unpin their epoch.
                        tracing::trace!("Query {:?} has scheduled all of its stages that have table scan (iterator creation).", self.query.query_id);
                        self.hummock_snapshot_manager
                            .release(self.epoch, self.query.query_id())
                            .await;
                    }

                    // For root stage, we execute in frontend local. We will pass the root fragment
                    // to QueryResultFetcher and execute to get a Chunk stream.
                    for parent in self.query.get_parents(&stage_id) {
                        if self.all_children_scheduled(parent).await
                                // Do not schedule same stage twice.
                                && self.stage_executions[parent].is_pending().await
                        {
                            self.stage_executions[parent].start().await;
                        }
                    }
                }
                Stage(ScheduledRoot(receiver)) => {
                    // We already schedule the root fragment, therefore we can notify query result
                    // fetcher.
                    self.send_root_stage_info(receiver);
                }
                Stage(StageEvent::Failed { id, reason }) => {
                    error!(
                        "Query stage {:?}-{:?} failed: {:?}.",
                        self.query.query_id, id, reason
                    );

                    self.handle_cancel_or_failed_stage(reason).await;
                    // One stage failed, not necessary to execute schedule stages.
                    break;
                }
                QueryMessage::CancelQuery => {
                    self.handle_cancel_or_failed_stage(SchedulerError::QueryCancelError)
                        .await;
                    // One stage failed, not necessary to execute schedule stages.
                    break;
                }
                rest => {
                    unimplemented!("unsupported message \"{:?}\" for QueryRunner.run", rest);
                }
            }
        }
    }

    /// The `shutdown_tx` will only be Some if the stage is 1. In that case, we should keep the life
    /// of shutdown sender so that shutdown receiver won't be triggered.
    fn send_root_stage_info(&mut self, chunk_rx: Receiver<SchedulerResult<DataChunk>>) {
        let root_task_output_id = {
            let root_task_id_prost = TaskIdProst {
                query_id: self.query.query_id.clone().id,
                stage_id: self.query.root_stage_id(),
                task_id: ROOT_TASK_ID,
            };

            TaskOutputIdProst {
                task_id: Some(root_task_id_prost),
                output_id: ROOT_TASK_OUTPUT_ID,
            }
        };

        let root_stage_result = QueryResultFetcher::new(
            self.epoch,
            self.hummock_snapshot_manager.clone(),
            root_task_output_id,
            // Execute in local, so no need to fill meaningful address.
            HostAddress {
                ..Default::default()
            },
            self.compute_client_pool.clone(),
            chunk_rx,
        );

        // Consume sender here.
        let root_stage_sender = mem::take(&mut self.root_stage_sender);

        if let Err(e) = root_stage_sender.unwrap().send(Ok(root_stage_result)) {
            warn!("Query execution dropped: {:?}", e);
        } else {
            debug!("Root stage for {:?} sent.", self.query.query_id);
        }
    }

    async fn all_children_scheduled(&self, stage_id: &StageId) -> bool {
        for child in self.query.stage_graph.get_child_stages_unchecked(stage_id) {
            if !self.stage_executions[child].is_scheduled().await {
                return false;
            }
        }
        true
    }

    /// Handle ctrl-c query or failed execution. Should stop all executions and send error to query
    /// result fetcher.
    async fn handle_cancel_or_failed_stage(mut self, reason: SchedulerError) {
        let err_str = reason.to_string();
        // Consume sender here and send error to root stage.
        let root_stage_sender = mem::take(&mut self.root_stage_sender);
        // It's possible we receive stage failed event message multi times and the
        // sender has been consumed in first failed event.
        if let Some(sender) = root_stage_sender {
            if let Err(e) = sender.send(Err(reason)) {
                warn!("Query execution dropped: {:?}", e);
            } else {
                debug!(
                    "Root stage failure event for {:?} sent.",
                    self.query.query_id
                );
            }
        }

        // If root stage has been taken (None), then root stage is responsible for send error to
        // Query Result Fetcher.

        // Stop all running stages.
        for (_stage_id, stage_execution) in self.stage_executions.iter() {
            // The stop is return immediately so no need to spawn tasks.
            stage_execution.stop(err_str.clone()).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::sync::Arc;

    use parking_lot::RwLock;
    use piestream_common::catalog::{ColumnDesc, TableDesc};
    use piestream_common::config::constant::hummock::TABLE_OPTION_DUMMY_RETENTION_SECOND;
    use piestream_common::types::DataType;
    use piestream_pb::common::{HostAddress, ParallelUnit, WorkerNode, WorkerType};
    use piestream_pb::plan_common::JoinType;
    use piestream_rpc_client::ComputeClientPool;

    use crate::catalog::catalog_service::CatalogReader;
    use crate::catalog::root_catalog::Catalog;
    use crate::expr::InputRef;
    use crate::optimizer::plan_node::{
        BatchExchange, BatchHashJoin, EqJoinPredicate, LogicalJoin, LogicalScan, ToBatch,
    };
    use crate::optimizer::property::{Distribution, Order};
    use crate::optimizer::PlanRef;
    use crate::scheduler::distributed::QueryExecution;
    use crate::scheduler::plan_fragmenter::{BatchPlanFragmenter, Query};
    use crate::scheduler::worker_node_manager::WorkerNodeManager;
    use crate::scheduler::{ExecutionContext, HummockSnapshotManager};
    use crate::session::{OptimizerContext, SessionImpl};
    use crate::test_utils::MockFrontendMetaClient;
    use crate::utils::Condition;

    #[tokio::test]
    async fn test_query_should_not_hang_with_empty_worker() {
        let worker_node_manager = Arc::new(WorkerNodeManager::mock(vec![]));
        let compute_client_pool = Arc::new(ComputeClientPool::default());
        let catalog_reader = CatalogReader::new(Arc::new(RwLock::new(Catalog::default())));
        let query_execution = QueryExecution::new(
            ExecutionContext::new(SessionImpl::mock().into()).into(),
            create_query().await,
            100,
            worker_node_manager,
            Arc::new(HummockSnapshotManager::new(Arc::new(
                MockFrontendMetaClient {},
            ))),
            compute_client_pool,
            catalog_reader,
            (0, 0),
        );
        assert!(query_execution.start().await.is_err());
    }

    async fn create_query() -> Query {
        // Construct a Hash Join with Exchange node.
        // Logical plan:
        //
        //    HashJoin
        //     /    \
        //   Scan  Scan
        //
        let ctx = OptimizerContext::mock().await;
        let table_id = 0.into();
        let batch_plan_node: PlanRef = LogicalScan::create(
            "".to_string(),
            false,
            Rc::new(TableDesc {
                table_id,
                stream_key: vec![],
                pk: vec![],
                columns: vec![
                    ColumnDesc {
                        data_type: DataType::Int32,
                        column_id: 0.into(),
                        name: "a".to_string(),
                        type_name: String::new(),
                        field_descs: vec![],
                    },
                    ColumnDesc {
                        data_type: DataType::Float64,
                        column_id: 1.into(),
                        name: "b".to_string(),
                        type_name: String::new(),
                        field_descs: vec![],
                    },
                ],
                distribution_key: vec![],
                appendonly: false,
                retention_seconds: TABLE_OPTION_DUMMY_RETENTION_SECOND,
                value_indices: vec![0, 1],
            }),
            vec![],
            ctx,
        )
        .to_batch()
        .unwrap()
        .to_distributed()
        .unwrap();
        let batch_exchange_node1: PlanRef = BatchExchange::new(
            batch_plan_node.clone(),
            Order::default(),
            Distribution::HashShard(vec![0, 1]),
        )
        .into();
        let batch_exchange_node2: PlanRef = BatchExchange::new(
            batch_plan_node.clone(),
            Order::default(),
            Distribution::HashShard(vec![0, 1]),
        )
        .into();
        let hash_join_node: PlanRef = BatchHashJoin::new(
            LogicalJoin::new(
                batch_exchange_node1.clone(),
                batch_exchange_node2.clone(),
                JoinType::Inner,
                Condition::true_cond(),
            ),
            EqJoinPredicate::new(
                Condition::true_cond(),
                vec![
                    (
                        InputRef {
                            index: 0,
                            data_type: DataType::Int32,
                        },
                        InputRef {
                            index: 2,
                            data_type: DataType::Int32,
                        },
                        false,
                    ),
                    (
                        InputRef {
                            index: 1,
                            data_type: DataType::Float64,
                        },
                        InputRef {
                            index: 3,
                            data_type: DataType::Float64,
                        },
                        false,
                    ),
                ],
                2,
            ),
        )
        .into();
        let batch_exchange_node3: PlanRef = BatchExchange::new(
            hash_join_node.clone(),
            Order::default(),
            Distribution::Single,
        )
        .into();

        let worker1 = WorkerNode {
            id: 0,
            r#type: WorkerType::ComputeNode as i32,
            host: Some(HostAddress {
                host: "127.0.0.1".to_string(),
                port: 5687,
            }),
            state: piestream_pb::common::worker_node::State::Running as i32,
            parallel_units: generate_parallel_units(0, 0),
        };
        let worker2 = WorkerNode {
            id: 1,
            r#type: WorkerType::ComputeNode as i32,
            host: Some(HostAddress {
                host: "127.0.0.1".to_string(),
                port: 5688,
            }),
            state: piestream_pb::common::worker_node::State::Running as i32,
            parallel_units: generate_parallel_units(8, 1),
        };
        let worker3 = WorkerNode {
            id: 2,
            r#type: WorkerType::ComputeNode as i32,
            host: Some(HostAddress {
                host: "127.0.0.1".to_string(),
                port: 5689,
            }),
            state: piestream_pb::common::worker_node::State::Running as i32,
            parallel_units: generate_parallel_units(16, 2),
        };
        let workers = vec![worker1, worker2, worker3];
        let worker_node_manager = Arc::new(WorkerNodeManager::mock(workers));
        worker_node_manager.insert_fragment_mapping(0, vec![]);
        let catalog = Arc::new(RwLock::new(Catalog::default()));
        catalog.write().insert_table_id_mapping(table_id, 0);
        let catalog_reader = CatalogReader::new(catalog);
        // Break the plan node into fragments.
        let fragmenter = BatchPlanFragmenter::new(worker_node_manager, catalog_reader);
        fragmenter.split(batch_exchange_node3.clone()).unwrap()
    }

    fn generate_parallel_units(start_id: u32, node_id: u32) -> Vec<ParallelUnit> {
        let parallel_degree = 8;
        (start_id..start_id + parallel_degree)
            .map(|id| ParallelUnit {
                id,
                worker_node_id: node_id,
            })
            .collect()
    }
}
