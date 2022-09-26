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

use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

use anyhow::anyhow;
use arc_swap::ArcSwap;
use futures::{stream, StreamExt};
use itertools::Itertools;
use piestream_pb::batch_plan::plan_node::NodeBody;
use piestream_pb::batch_plan::{
    ExchangeNode, ExchangeSource, MergeSortExchangeNode, PlanFragment, PlanNode as PlanNodeProst,
    TaskId as TaskIdProst, TaskOutputId,
};
use piestream_pb::common::{Buffer, HostAddress, WorkerNode};
use piestream_rpc_client::ComputeClientPoolRef;
use tokio::spawn;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{error, info};
use uuid::Uuid;
use StageEvent::Failed;

use crate::optimizer::plan_node::PlanNodeType;
use crate::scheduler::distributed::stage::StageState::Pending;
use crate::scheduler::distributed::QueryMessage;
use crate::scheduler::plan_fragmenter::{ExecutionPlanNode, QueryStageRef, StageId, TaskId};
use crate::scheduler::worker_node_manager::WorkerNodeManagerRef;
use crate::scheduler::SchedulerError::Internal;
use crate::scheduler::{SchedulerError, SchedulerResult};

const TASK_SCHEDULING_PARALLELISM: usize = 10;

enum StageState {
    Pending,
    Started {
        sender: Sender<StageMessage>,
        handle: JoinHandle<SchedulerResult<()>>,
    },
    Running {
        _sender: Sender<StageMessage>,
        _handle: JoinHandle<SchedulerResult<()>>,
    },
    Completed,
    Failed,
}

enum StageMessage {
    Stop,
}

#[derive(Debug)]
pub enum StageEvent {
    Scheduled(StageId),
    /// Stage failed.
    Failed {
        id: StageId,
        reason: SchedulerError,
    },
    Completed(StageId),
}

#[derive(Clone)]
pub struct TaskStatus {
    _task_id: TaskId,

    // None before task is scheduled.
    location: Option<HostAddress>,
}

struct TaskStatusHolder {
    inner: ArcSwap<TaskStatus>,
}

pub struct StageExecution {
    epoch: u64,
    stage: QueryStageRef,
    worker_node_manager: WorkerNodeManagerRef,
    tasks: Arc<HashMap<TaskId, TaskStatusHolder>>,
    state: Arc<RwLock<StageState>>,
    msg_sender: Sender<QueryMessage>,

    /// Children stage executions.
    ///
    /// We use `Vec` here since children's size is usually small.
    children: Vec<Arc<StageExecution>>,
    compute_client_pool: ComputeClientPoolRef,
}

struct StageRunner {
    epoch: u64,
    state: Arc<RwLock<StageState>>,
    stage: QueryStageRef,
    worker_node_manager: WorkerNodeManagerRef,
    tasks: Arc<HashMap<TaskId, TaskStatusHolder>>,
    _receiver: Receiver<StageMessage>,
    // Send message to `QueryRunner` to notify stage state change.
    msg_sender: Sender<QueryMessage>,
    children: Vec<Arc<StageExecution>>,
    compute_client_pool: ComputeClientPoolRef,
}

impl TaskStatusHolder {
    fn new(task_id: TaskId) -> Self {
        let task_status = TaskStatus {
            _task_id: task_id,
            location: None,
        };

        Self {
            inner: ArcSwap::new(Arc::new(task_status)),
        }
    }

    fn get_status(&self) -> Arc<TaskStatus> {
        self.inner.load_full()
    }
}

impl StageExecution {
    pub fn new(
        epoch: u64,
        stage: QueryStageRef,
        worker_node_manager: WorkerNodeManagerRef,
        msg_sender: Sender<QueryMessage>,
        children: Vec<Arc<StageExecution>>,
        compute_client_pool: ComputeClientPoolRef,
    ) -> Self {
        //构建tasks
        let tasks = (0..stage.parallelism)
            .into_iter()
            .map(|task_id| (task_id, TaskStatusHolder::new(task_id)))
            .collect();
        Self {
            epoch,
            stage,
            worker_node_manager,
            tasks: Arc::new(tasks),
            state: Arc::new(RwLock::new(Pending)),
            msg_sender,
            children,
            compute_client_pool,
        }
    }

    /// Starts execution of this stage, returns error if already started.
    /// 具体的stage
    pub async fn start(&self) -> SchedulerResult<()> {
        let mut s = self.state.write().await;
        match &*s {
            &StageState::Pending => {
                let (sender, receiver) = channel(100);
                let runner = StageRunner {
                    epoch: self.epoch,
                    stage: self.stage.clone(),
                    worker_node_manager: self.worker_node_manager.clone(),
                    tasks: self.tasks.clone(),
                    _receiver: receiver,
                    msg_sender: self.msg_sender.clone(),
                    children: self.children.clone(),
                    state: self.state.clone(),
                    compute_client_pool: self.compute_client_pool.clone(),
                };
                let handle = spawn(async move {
                    //runner.run update 
                    if let Err(e) = runner.run().await {
                        error!("Stage failed: {:?}", e);
                        Err(e)
                    } else {
                        Ok(())
                    }
                });
                // updated StageState state    
                *s = StageState::Started { sender, handle };
                Ok(())
            }
            _ => {
                // This is possible since we notify stage schedule event to query runner, which may
                // receive multi events and start stage multi times.
                info!(
                    "Staged {:?}-{:?} already started, skipping.",
                    &self.stage.query_id, &self.stage.id
                );
                Ok(())
            }
        }
    }

    pub async fn stop(&self) -> SchedulerResult<()> {
        todo!()
    }

    pub async fn is_scheduled(&self) -> bool {
        let s = self.state.read().await;
        matches!(*s, StageState::Running { .. })
    }

    pub fn get_task_status_unchecked(&self, task_id: TaskId) -> Arc<TaskStatus> {
        self.tasks[&task_id].get_status()
    }

    /// Returns all exchange sources for `output_id`. Each `ExchangeSource` is identified by
    /// producer's `TaskId` and `output_id` (consumer's `TaskId`), since each task may produce
    /// output to several channels.
    ///
    /// When this method is called, all tasks should have been scheduled, and their `worker_node`
    /// should have been set.
    /*
        返回 `output_id` 的所有交换源。每个 `ExchangeSource` 都由生产者的 `TaskId` 和 `output_id`（消费者的 `TaskId`）标识，因为每个任务可能会产生多个通道的输出。
        当这个方法被调用时，所有的任务应该已经被调度了，它们的 `worker_node`应该已经设置好了。
     */
    fn all_exchange_sources_for(&self, output_id: u32) -> Vec<ExchangeSource> {
        //self.tasks数组大小为parallelism
        self.tasks
            .iter()
            .map(|(task_id, status_holder)| {
                let task_output_id = TaskOutputId {
                    task_id: Some(TaskIdProst {
                        query_id: self.stage.query_id.id.clone(),
                        stage_id: self.stage.id,
                        task_id: *task_id,
                    }),
                    output_id, //父节点任务id
                };
                //带上compute节点路由信息,querymanage遍历过程儿子stage运行结束后在运行父节点
                ExchangeSource {
                    task_output_id: Some(task_output_id),
                    host: Some(status_holder.inner.load_full().location.clone().unwrap()),
                    local_execute_plan: None,
                }
            })
            .collect()
    }
}

impl StageRunner {
    async fn run(self) -> SchedulerResult<()> {
        //构建查询任务并发送给对应计算节点
        if let Err(e) = self.schedule_tasks().await {
            error!(
                "Stage {:?}-{:?} failed to schedule tasks, error: {:?}",
                self.stage.query_id, self.stage.id, e
            );
            // TODO: We should cancel all scheduled tasks
            //发送失败信息
            self.send_event(QueryMessage::Stage(Failed {
                id: self.stage.id,
                reason: e,
            }))
            .await?;
            return Ok(());
        }

        {
            // Changing state  to Running
            let mut s = self.state.write().await;
            match mem::replace(&mut *s, StageState::Failed) {
                StageState::Started { sender, handle } => {
                    //更新状态
                    *s = StageState::Running {
                        _sender: sender,
                        _handle: handle,
                    };
                }
                _ => unreachable!(),
            }
        }

        // All tasks scheduled, send `StageScheduled` event to `QueryRunner`.
        //send stage.id to all tasks scheduled
        self.send_event(QueryMessage::Stage(StageEvent::Scheduled(self.stage.id)))
            .await?;

        Ok(())
    }

    /// Send stage event to listener.
    async fn send_event(&self, event: QueryMessage) -> SchedulerResult<()> {
        self.msg_sender.send(event).await.map_err(|e| {
            {
                Internal(anyhow!(
                    "Failed to send stage scheduled event: {:?}, reason: {:?}",
                    self.stage.id,
                    e
                ))
            }
        })
    }

    async fn schedule_tasks(&self) -> SchedulerResult<()> {
        let mut futures = vec![];

        //需要扫表的任务
        if let Some(table_scan_info) = self.stage.table_scan_info.as_ref() 
                        && let Some(vnode_bitmaps) = table_scan_info.vnode_bitmaps.as_ref() {
            // If the stage has table scan nodes, we create tasks according to the data distribution
            // and partition of the table.
            // We let each task read one partition by setting the `vnode_ranges` of the scan node in
            // the task.
            // We schedule the task to the worker node that owns the data partition.
            /*
                如果stage有表扫描节点，我们根据数据分布和表的分区创建任务
                我们通过设置任务中扫描节点的`vnode_ranges`让每个任务读取一个分区。
                我们将任务调度到拥有数据分区的工作节点。
            */
            let parallel_unit_ids = vnode_bitmaps.keys().cloned().collect_vec();
            let workers = self
                .worker_node_manager
                .get_workers_by_parallel_unit_ids(&parallel_unit_ids)?;

            //指定worker节点
            for (i, (parallel_unit_id, worker)) in parallel_unit_ids
                .into_iter()
                .zip_eq(workers.into_iter())
                .enumerate()
            {
                let task_id = TaskIdProst {
                    query_id: self.stage.query_id.id.clone(),
                    stage_id: self.stage.id,
                    task_id: i as u32,
                };
                //vnode_ranges存储是当前parallel_unit_id对应的vnode信息
                let vnode_ranges = vnode_bitmaps[&parallel_unit_id].clone();
                let plan_fragment = self.create_plan_fragment(i as u32, Some(vnode_ranges));
                futures.push(self.schedule_task(task_id, plan_fragment, Some(worker)));
            }
        } else {
            println!("self.stage.parallelism={:?}", self.stage.parallelism);
            //Single调度
            for id in 0..self.stage.parallelism {
                let task_id = TaskIdProst {
                    query_id: self.stage.query_id.id.clone(),
                    stage_id: self.stage.id,
                    task_id: id,
                };
                //创建执行计划,每个工作节点创建不同的plan_fragment
                let plan_fragment = self.create_plan_fragment(id, None);
                //随机woker节点
                futures.push(self.schedule_task(task_id, plan_fragment, None));
            }
        }
        //统一开始执行,buffer_unordered是缓冲区大小，设置为0会卡住
        let mut buffered = stream::iter(futures).buffer_unordered(TASK_SCHEDULING_PARALLELISM);
        while let Some(result) = buffered.next().await {
            result?;
        }
        //所有任务都发送成功
        Ok(())
    }

    async fn schedule_task(
        &self,
        task_id: TaskIdProst,
        plan_fragment: PlanFragment,
        worker: Option<WorkerNode>,
    ) -> SchedulerResult<()> {
        //worker为None则随机指定
        let worker_node_addr = worker
            .unwrap_or(self.worker_node_manager.next_random()?)
            .host
            .unwrap();

        let compute_client = self
            .compute_client_pool
            .get_client_for_addr((&worker_node_addr).into())
            .await
            .map_err(|e| anyhow!(e))?;

        //创建任务并发送任务给compute    
        let t_id = task_id.task_id;
        compute_client
            .create_task2(task_id, plan_fragment, self.epoch)
            .await
            .map_err(|e| anyhow!(e))?;

        //更新t_id任务的路由以及id信息
        self.tasks[&t_id].inner.store(Arc::new(TaskStatus {
            _task_id: t_id,
            location: Some(worker_node_addr),
        }));

        Ok(())
    }

    fn create_plan_fragment(&self, task_id: TaskId, vnode_bitmap: Option<Buffer>) -> PlanFragment {
        let plan_node_prost = self.convert_plan_node(&self.stage.root, task_id, vnode_bitmap);
        let exchange_info = self.stage.exchange_info.clone();

        PlanFragment {
            root: Some(plan_node_prost),
            exchange_info: Some(exchange_info),
        }
    }

    fn convert_plan_node(
        &self,
        execution_plan_node: &ExecutionPlanNode,
        task_id: TaskId,
        vnode_bitmap: Option<Buffer>,
    ) -> PlanNodeProst {
        match execution_plan_node.plan_node_type {
            PlanNodeType::BatchExchange => {
                // Find the stage this exchange node should fetch from and get all exchange sources.
                // exchange node都必须由下游stage节点提供数据，并且exchange应该只有一个节点
                let child_stage = self
                    .children
                    .iter()
                    .find(|child_stage| {
                        child_stage.stage.id == execution_plan_node.source_stage_id.unwrap()
                    })
                    .unwrap();
                //将task_id关联到其stage子任务，存储在exchange_sources，一个exchange节点只接入一个下游stage节点
                let exchange_sources = child_stage.all_exchange_sources_for(task_id);

                match &execution_plan_node.node {
                    NodeBody::Exchange(_exchange_node) => {
                        PlanNodeProst {
                            children: vec![],
                            // TODO: Generate meaningful identify
                            identity: Uuid::new_v4().to_string(),
                            node_body: Some(NodeBody::Exchange(ExchangeNode {
                                sources: exchange_sources,
                                input_schema: execution_plan_node.schema.clone(),
                            })),
                        }
                    }
                    NodeBody::MergeSortExchange(sort_merge_exchange_node) => {
                        PlanNodeProst {
                            children: vec![],
                            // TODO: Generate meaningful identify
                            identity: Uuid::new_v4().to_string(),
                            node_body: Some(NodeBody::MergeSortExchange(MergeSortExchangeNode {
                                exchange: Some(ExchangeNode {
                                    sources: exchange_sources,
                                    input_schema: execution_plan_node.schema.clone(),
                                }),
                                column_orders: sort_merge_exchange_node.column_orders.clone(),
                            })),
                        }
                    }
                    _ => unreachable!(),
                }
            }
            PlanNodeType::BatchSeqScan => {
                let node_body = execution_plan_node.node.clone();
                //按行扫表
                let NodeBody::RowSeqScan(mut scan_node) = node_body else {
                    unreachable!();
                };
                //设置下发信息
                scan_node.vnode_bitmap = vnode_bitmap;
                PlanNodeProst {
                    children: vec![],
                    // TODO: Generate meaningful identify
                    identity: Uuid::new_v4().to_string(),
                    node_body: Some(NodeBody::RowSeqScan(scan_node)),
                }
            }
            _ => {
                //stage内父子节点
                let children = execution_plan_node
                    .children
                    .iter()
                    .map(|e| self.convert_plan_node(e, task_id, vnode_bitmap.clone()))
                    .collect();

                PlanNodeProst {
                    children,
                    // TODO: Generate meaningful identify
                    identity: Uuid::new_v4().to_string(),
                    node_body: Some(execution_plan_node.node.clone()),
                }
            }
        }
    }
}

impl TaskStatus {
    pub fn task_host_unchecked(&self) -> HostAddress {
        self.location.clone().unwrap()
    }
}
