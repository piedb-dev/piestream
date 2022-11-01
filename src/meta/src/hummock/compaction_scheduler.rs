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

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use piestream_hummock_sdk::compact::compact_task_to_string;
use piestream_hummock_sdk::CompactionGroupId;
use piestream_pb::hummock::compact_task::TaskStatus;
use piestream_pb::hummock::subscribe_compact_tasks_response::Task;
use piestream_pb::hummock::CompactTask;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot::Receiver;
use tokio::sync::Notify;

use super::Compactor;
use crate::hummock::error::Error;
use crate::hummock::{CompactorManagerRef, HummockManagerRef};
use crate::manager::{LocalNotification, MetaSrvEnv};
use crate::storage::MetaStore;

pub type CompactionSchedulerRef<S> = Arc<CompactionScheduler<S>>;
pub type CompactionRequestChannelRef = Arc<CompactionRequestChannel>;

/// [`CompactionRequestChannel`] wrappers a mpsc channel and deduplicate requests from same
/// compaction groups.
pub struct CompactionRequestChannel {
    request_tx: UnboundedSender<CompactionGroupId>,
    scheduled: Mutex<HashSet<CompactionGroupId>>,
}

#[derive(Debug, PartialEq)]
pub enum ScheduleStatus {
    Ok,
    NoTask,
    PickFailure,
    AssignFailure(CompactTask),
    SendFailure(CompactTask),
}

impl CompactionRequestChannel {
    fn new(request_tx: UnboundedSender<CompactionGroupId>) -> Self {
        Self {
            request_tx,
            scheduled: Default::default(),
        }
    }

    /// Enqueues only if the target is not yet in queue.
    pub fn try_sched_compaction(
        &self,
        compaction_group: CompactionGroupId,
    ) -> Result<bool, SendError<CompactionGroupId>> {
        let mut guard = self.scheduled.lock();
        if guard.contains(&compaction_group) {
            return Ok(false);
        }
        self.request_tx.send(compaction_group)?;
        guard.insert(compaction_group);
        Ok(true)
    }

    fn unschedule(&self, compaction_group: CompactionGroupId) {
        self.scheduled.lock().remove(&compaction_group);
    }
}

/// Schedules compaction task picking and assignment.
///
/// When no idle compactor is available, the scheduling will be paused until
/// `compaction_resume_notifier` is `notified`. Compaction should only be resumed by calling
/// `HummockManager::try_resume_compaction`. See [`CompactionResumeTrigger`] for all cases that can
/// resume compaction.
pub struct CompactionScheduler<S>
where
    S: MetaStore,
{
    env: MetaSrvEnv<S>,
    hummock_manager: HummockManagerRef<S>,
    compactor_manager: CompactorManagerRef,
    compaction_resume_notifier: Arc<Notify>,
}

impl<S> CompactionScheduler<S>
where
    S: MetaStore,
{
    pub fn new(
        env: MetaSrvEnv<S>,
        hummock_manager: HummockManagerRef<S>,
        compactor_manager: CompactorManagerRef,
    ) -> Self {
        Self {
            env,
            hummock_manager,
            compactor_manager,
            compaction_resume_notifier: Arc::new(Notify::new()),
        }
    }

    pub async fn start(&self, mut shutdown_rx: Receiver<()>) {
        let (sched_tx, mut sched_rx) = tokio::sync::mpsc::unbounded_channel::<CompactionGroupId>();
        let sched_channel = Arc::new(CompactionRequestChannel::new(sched_tx));

        self.hummock_manager.init_compaction_scheduler(
            sched_channel.clone(),
            self.compaction_resume_notifier.clone(),
        );

        tracing::info!("Start compaction scheduler.");
        let mut min_trigger_interval = tokio::time::interval(Duration::from_secs(
            self.env.opts.periodic_compaction_interval_sec,
        ));
        min_trigger_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            let compaction_group: CompactionGroupId = tokio::select! {
                compaction_group = sched_rx.recv() => {
                    match compaction_group {
                        Some(compaction_group) => compaction_group,
                        None => {
                            tracing::warn!("Compactor Scheduler: The Hummock manager has dropped the connection,
                                it means it has either died or started a new session. Exiting.");
                            return;
                        }
                    }
                },
                _ = min_trigger_interval.tick() => {
                    // Periodically trigger compaction for all compaction groups.
                    for cg_id in self.hummock_manager.compaction_group_manager().compaction_group_ids().await {
                        if let Err(e) = sched_channel.try_sched_compaction(cg_id) {
                            tracing::warn!("Failed to schedule compaction for compaction group {}. {}", cg_id, e);
                        }
                    }
                    continue;
                },
                // Shutdown compactor scheduler
                _ = &mut shutdown_rx => {
                    break;
                }
            };

            sync_point::sync_point!("BEFORE_SCHEDULE_COMPACTION_TASK");
            sched_channel.unschedule(compaction_group);

            // Wait for a compactor to become available.
            let compactor = loop {
                if let Some(compactor) = self.hummock_manager.get_idle_compactor().await {
                    break compactor;
                } else {
                    tracing::debug!("No available compactor, pausing compaction.");
                    tokio::select! {
                        _ = self.compaction_resume_notifier.notified() => {},
                        _ = &mut shutdown_rx => {
                            return;
                        }
                    }
                }
            };

            // Pick a task and assign it to this compactor.
            self.pick_and_assign(compaction_group, compactor, sched_channel.clone())
                .await;
        }
    }

    /// Tries to pick a compaction task, schedule it to a compactor.
    ///
    /// Returns true if a task is successfully picked and sent.
    async fn pick_and_assign(
        &self,
        compaction_group: CompactionGroupId,
        compactor: Arc<Compactor>,
        sched_channel: Arc<CompactionRequestChannel>,
    ) -> ScheduleStatus {
        let schedule_status = self
            .pick_and_assign_impl(compaction_group, compactor, sched_channel)
            .await;

        // Self::unschedule(sched_channel, &side_sched_channel, compaction_group);
        let cancel_state = match &schedule_status {
            ScheduleStatus::Ok => None,
            ScheduleStatus::NoTask | ScheduleStatus::PickFailure => None,
            ScheduleStatus::AssignFailure(task) => {
                Some((task.clone(), TaskStatus::AssignFailCanceled))
            }
            ScheduleStatus::SendFailure(task) => Some((task.clone(), TaskStatus::SendFailCanceled)),
        };

        if let Some((mut compact_task, task_state)) = cancel_state {
            // Try to cancel task immediately.
            if let Err(err) = self
                .hummock_manager
                .cancel_compact_task(&mut compact_task, task_state)
                .await
            {
                // Cancel task asynchronously.
                tracing::warn!(
                    "Failed to cancel task {}. {}. {:?} It will be cancelled asynchronously.",
                    compact_task.task_id,
                    err,
                    task_state
                );
                self.env
                    .notification_manager()
                    .notify_local_subscribers(LocalNotification::CompactionTaskNeedCancel(
                        compact_task,
                    ))
                    .await;
            }
        }
        schedule_status
    }

    async fn pick_and_assign_impl(
        &self,
        compaction_group: CompactionGroupId,
        compactor: Arc<Compactor>,
        sched_channel: Arc<CompactionRequestChannel>,
    ) -> ScheduleStatus {
        // 1. Pick a compaction task.
        let compact_task = self
            .hummock_manager
            .get_compact_task(compaction_group)
            .await;
        let compact_task = match compact_task {
            Ok(Some(compact_task)) => compact_task,
            Ok(None) => {
                return ScheduleStatus::NoTask;
            }
            Err(err) => {
                tracing::warn!("Failed to get compaction task: {:#?}.", err);
                return ScheduleStatus::PickFailure;
            }
        };
        tracing::trace!(
            "Picked compaction task. {}",
            compact_task_to_string(&compact_task)
        );

        // 2. Assign the compaction task to a compactor.
        match self
            .hummock_manager
            .assign_compaction_task(&compact_task, compactor.context_id())
            .await
        {
            Ok(_) => {
                tracing::trace!(
                    "Assigned compaction task. {}",
                    compact_task_to_string(&compact_task)
                );
            }
            Err(err) => {
                tracing::warn!("Failed to assign compaction task to compactor: {:#?}", err);
                match err {
                    Error::CompactionTaskAlreadyAssigned(_, _) => {
                        panic!("Compaction scheduler is the only tokio task that can assign task.");
                    }
                    Error::InvalidContext(context_id) => {
                        self.compactor_manager.remove_compactor(context_id);
                        return ScheduleStatus::AssignFailure(compact_task);
                    }
                    _ => {
                        return ScheduleStatus::AssignFailure(compact_task);
                    }
                }
            }
        };

        // 3. Send the compaction task.
        if let Err(e) = compactor
            .send_task(Task::CompactTask(compact_task.clone()))
            .await
        {
            tracing::warn!(
                "Failed to send task {} to {}. {:#?}",
                compact_task.task_id,
                compactor.context_id(),
                e
            );
            self.compactor_manager
                .pause_compactor(compactor.context_id());
            return ScheduleStatus::SendFailure(compact_task);
        }

        // Bypass reschedule if we want compaction scheduling in a deterministic way
        if self.env.opts.compaction_deterministic_test {
            return ScheduleStatus::Ok;
        }

        // 4. Reschedule it with best effort, in case there are more tasks.
        if let Err(e) = sched_channel.try_sched_compaction(compaction_group) {
            tracing::error!(
                "Failed to reschedule compaction group {} after sending new task {}. {:#?}",
                compaction_group,
                compact_task.task_id,
                e
            );
        }
        ScheduleStatus::Ok
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use assert_matches::assert_matches;
    use piestream_hummock_sdk::compaction_group::StaticCompactionGroupId;
    use piestream_hummock_sdk::CompactionGroupId;

    use crate::hummock::compaction_scheduler::{CompactionRequestChannel, ScheduleStatus};
    use crate::hummock::test_utils::{add_ssts, setup_compute_env};
    use crate::hummock::CompactionScheduler;

    #[tokio::test]
    async fn test_pick_and_assign() {
        let (env, hummock_manager, _cluster_manager, worker_node) = setup_compute_env(80).await;
        let context_id = worker_node.id;
        let compactor_manager = hummock_manager.compactor_manager_ref_for_test();
        let compaction_scheduler =
            CompactionScheduler::new(env, hummock_manager.clone(), compactor_manager.clone());

        let (request_tx, _request_rx) = tokio::sync::mpsc::unbounded_channel::<CompactionGroupId>();
        let request_channel = Arc::new(CompactionRequestChannel::new(request_tx));

        // Add a compactor with invalid context_id.
        let _receiver = compactor_manager.add_compactor(1234, 1);
        assert_eq!(compactor_manager.compactor_num(), 1);

        // No task
        let compactor = hummock_manager.get_idle_compactor().await.unwrap();
        assert_eq!(
            ScheduleStatus::NoTask,
            compaction_scheduler
                .pick_and_assign(
                    StaticCompactionGroupId::StateDefault.into(),
                    compactor,
                    request_channel.clone()
                )
                .await
        );

        let _sst_infos = add_ssts(1, hummock_manager.as_ref(), context_id).await;
        let compactor = hummock_manager.get_idle_compactor().await.unwrap();
        // Cannot assign because of invalid compactor
        assert_matches!(
            compaction_scheduler
                .pick_and_assign(
                    StaticCompactionGroupId::StateDefault.into(),
                    compactor,
                    request_channel.clone()
                )
                .await,
            ScheduleStatus::AssignFailure(_)
        );
        assert_eq!(compactor_manager.compactor_num(), 0);

        // Add a valid compactor and succeed
        let _receiver = compactor_manager.add_compactor(context_id, 1);
        assert_eq!(compactor_manager.compactor_num(), 1);
        let compactor = hummock_manager.get_idle_compactor().await.unwrap();
        assert_eq!(
            ScheduleStatus::Ok,
            compaction_scheduler
                .pick_and_assign(
                    StaticCompactionGroupId::StateDefault.into(),
                    compactor,
                    request_channel.clone()
                )
                .await
        );

        // Add more SSTs for compaction.
        let _sst_infos = add_ssts(2, hummock_manager.as_ref(), context_id).await;

        // No idle compactor
        assert_eq!(
            hummock_manager.get_assigned_tasks_number(context_id).await,
            1
        );
        assert_eq!(compactor_manager.compactor_num(), 1);
        assert_matches!(hummock_manager.get_idle_compactor().await, None);

        // Increase compactor concurrency and succeed
        let _receiver = compactor_manager.add_compactor(context_id, 10);
        assert_eq!(
            hummock_manager.get_assigned_tasks_number(context_id).await,
            1
        );
        let compactor = hummock_manager.get_idle_compactor().await.unwrap();
        assert_eq!(
            ScheduleStatus::Ok,
            compaction_scheduler
                .pick_and_assign(
                    StaticCompactionGroupId::StateDefault.into(),
                    compactor,
                    request_channel.clone()
                )
                .await
        );
        assert_eq!(
            hummock_manager.get_assigned_tasks_number(context_id).await,
            2
        );
    }

    #[tokio::test]
    #[cfg(all(test, feature = "failpoints"))]
    async fn test_failpoints() {
        use piestream_pb::hummock::compact_task::TaskStatus;

        use crate::manager::LocalNotification;

        let (env, hummock_manager, _cluster_manager, worker_node) = setup_compute_env(80).await;
        let context_id = worker_node.id;
        let compactor_manager = hummock_manager.compactor_manager_ref_for_test();
        let compaction_scheduler = CompactionScheduler::new(
            env.clone(),
            hummock_manager.clone(),
            compactor_manager.clone(),
        );

        let (request_tx, _request_rx) = tokio::sync::mpsc::unbounded_channel::<CompactionGroupId>();
        let request_channel = Arc::new(CompactionRequestChannel::new(request_tx));

        let _sst_infos = add_ssts(1, hummock_manager.as_ref(), context_id).await;
        let _receiver = compactor_manager.add_compactor(context_id, 1);

        // Pick failure
        let fp_get_compact_task = "fp_get_compact_task";
        fail::cfg(fp_get_compact_task, "return").unwrap();
        let compactor = hummock_manager.get_idle_compactor().await.unwrap();
        assert_eq!(
            ScheduleStatus::PickFailure,
            compaction_scheduler
                .pick_and_assign(
                    StaticCompactionGroupId::StateDefault.into(),
                    compactor,
                    request_channel.clone()
                )
                .await
        );
        fail::remove(fp_get_compact_task);

        // Assign failed and task cancelled.
        let fp_assign_compaction_task_fail = "assign_compaction_task_fail";
        fail::cfg(fp_assign_compaction_task_fail, "return").unwrap();
        let compactor = hummock_manager.get_idle_compactor().await.unwrap();
        assert_matches!(
            compaction_scheduler
                .pick_and_assign(
                    StaticCompactionGroupId::StateDefault.into(),
                    compactor,
                    request_channel.clone()
                )
                .await,
            ScheduleStatus::AssignFailure(_)
        );
        fail::remove(fp_assign_compaction_task_fail);
        assert!(hummock_manager.list_all_tasks_ids().await.is_empty());

        // Send failed and task cancelled.
        let fp_compaction_send_task_fail = "compaction_send_task_fail";
        fail::cfg(fp_compaction_send_task_fail, "return").unwrap();
        let compactor = hummock_manager.get_idle_compactor().await.unwrap();
        assert_matches!(
            compaction_scheduler
                .pick_and_assign(
                    StaticCompactionGroupId::StateDefault.into(),
                    compactor,
                    request_channel.clone()
                )
                .await,
            ScheduleStatus::SendFailure(_)
        );
        fail::remove(fp_compaction_send_task_fail);
        assert!(hummock_manager.list_all_tasks_ids().await.is_empty());

        // There is no idle compactor, because the compactor is paused after send failure.
        assert_matches!(hummock_manager.get_idle_compactor().await, None);
        assert!(hummock_manager.list_all_tasks_ids().await.is_empty());
        let _receiver = compactor_manager.add_compactor(context_id, 1);

        // Assign failed and task cancellation failed.
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        env.notification_manager().insert_local_sender(tx).await;
        let fp_cancel_compact_task = "fp_cancel_compact_task";
        fail::cfg(fp_assign_compaction_task_fail, "return").unwrap();
        fail::cfg(fp_cancel_compact_task, "return").unwrap();
        let compactor = hummock_manager.get_idle_compactor().await.unwrap();
        assert_matches!(
            compaction_scheduler
                .pick_and_assign(
                    StaticCompactionGroupId::StateDefault.into(),
                    compactor,
                    request_channel.clone()
                )
                .await,
            ScheduleStatus::AssignFailure(_)
        );
        fail::remove(fp_assign_compaction_task_fail);
        fail::remove(fp_cancel_compact_task);
        assert_eq!(hummock_manager.list_all_tasks_ids().await.len(), 1);
        // Notified to retry cancellation.
        let mut task_to_cancel = match rx.recv().await.unwrap() {
            LocalNotification::WorkerNodeIsDeleted(_) => {
                panic!()
            }
            LocalNotification::CompactionTaskNeedCancel(task_to_cancel) => task_to_cancel,
        };
        hummock_manager
            .cancel_compact_task(&mut task_to_cancel, TaskStatus::ManualCanceled)
            .await
            .unwrap();
        assert!(hummock_manager.list_all_tasks_ids().await.is_empty());

        // Succeeded.
        let compactor = hummock_manager.get_idle_compactor().await.unwrap();
        assert_matches!(
            compaction_scheduler
                .pick_and_assign(
                    StaticCompactionGroupId::StateDefault.into(),
                    compactor,
                    request_channel.clone()
                )
                .await,
            ScheduleStatus::Ok
        );
        assert_eq!(hummock_manager.list_all_tasks_ids().await.len(), 1);
    }
}
