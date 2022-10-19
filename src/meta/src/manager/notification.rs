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
use std::sync::Arc;

use piestream_pb::common::{WorkerNode, WorkerType};
use piestream_pb::hummock::CompactTask;
use piestream_pb::meta::subscribe_response::{Info, Operation};
use piestream_pb::meta::{SubscribeResponse, SubscribeType};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::sync::{oneshot, Mutex};
use tonic::Status;

use crate::manager::cluster::WorkerKey;
use crate::model::NotificationVersion as Version;
use crate::storage::MetaStore;

pub type MessageStatus = Status;
pub type Notification = Result<SubscribeResponse, Status>;
pub type NotificationManagerRef<S> = Arc<NotificationManager<S>>;
pub type NotificationVersion = u64;

#[derive(Clone, Debug)]
pub enum LocalNotification {
    WorkerNodeIsDeleted(WorkerNode),
    CompactionTaskNeedCancel(CompactTask),
}

#[derive(Debug)]
struct Task {
    target: SubscribeType,
    callback_tx: Option<oneshot::Sender<NotificationVersion>>,
    operation: Operation,
    info: Info,
}

/// [`NotificationManager`] is used to send notification to frontends and compute nodes.
pub struct NotificationManager<S> {
    core: Arc<Mutex<NotificationManagerCore<S>>>,
    /// Sender used to add a notification into the waiting queue.
    task_tx: UnboundedSender<Task>,
}

impl<S> NotificationManager<S>
where
    S: MetaStore,
{
    pub async fn new(meta_store: Arc<S>) -> Self {
        // notification waiting queue.
        let (task_tx, mut task_rx) = mpsc::unbounded_channel::<Task>();
        let core = Arc::new(Mutex::new(NotificationManagerCore::new(meta_store).await));
        let core_clone = core.clone();

        tokio::spawn(async move {
            while let Some(task) = task_rx.recv().await {
                let mut guard = core.lock().await;
                guard.notify(task.target, task.operation, &task.info).await;
                if let Some(tx) = task.callback_tx {
                    tx.send(guard.current_version.version()).unwrap();
                }
            }
        });

        Self {
            core: core_clone,
            task_tx,
        }
    }

    /// Add a notification to the waiting queue and return immediately
    pub fn notify_asynchronously(&self, target: SubscribeType, operation: Operation, info: Info) {
        let task = Task {
            target,
            callback_tx: None,
            operation,
            info,
        };
        self.task_tx.send(task).unwrap();
    }

    /// Add a notification to the waiting queue, and will not return until the notification is
    /// sent successfully
    async fn notify(
        &self,
        target: SubscribeType,
        operation: Operation,
        info: Info,
    ) -> NotificationVersion {
        let (callback_tx, callback_rx) = oneshot::channel();
        let task = Task {
            target,
            callback_tx: Some(callback_tx),
            operation,
            info,
        };
        self.task_tx.send(task).unwrap();
        callback_rx.await.unwrap()
    }

    pub fn notify_frontend_asynchronously(&self, operation: Operation, info: Info) {
        self.notify_asynchronously(SubscribeType::Frontend, operation, info);
    }

    pub async fn notify_frontend(&self, operation: Operation, info: Info) -> NotificationVersion {
        self.notify(SubscribeType::Frontend, operation, info).await
    }

    pub async fn notify_hummock(&self, operation: Operation, info: Info) -> NotificationVersion {
        self.notify(SubscribeType::Hummock, operation, info).await
    }

    pub async fn notify_compactor(&self, operation: Operation, info: Info) -> NotificationVersion {
        self.notify(SubscribeType::Compactor, operation, info).await
    }

    pub fn notify_hummock_asynchronously(&self, operation: Operation, info: Info) {
        self.notify_asynchronously(SubscribeType::Hummock, operation, info);
    }

    pub async fn notify_local_subscribers(&self, notification: LocalNotification) {
        let mut core_guard = self.core.lock().await;
        core_guard.local_senders.retain(|sender| {
            if let Err(err) = sender.send(notification.clone()) {
                tracing::warn!("Failed to notify local subscriber. {}", err);
                return false;
            }
            true
        });
    }

    /// Tell `NotificationManagerCore` to delete sender.
    pub async fn delete_sender(&self, worker_type: WorkerType, worker_key: WorkerKey) {
        let mut core_guard = self.core.lock().await;
        // TODO: we may avoid passing the worker_type and remove the `worker_key` in all sender
        // holders anyway
        match worker_type {
            WorkerType::Frontend => core_guard.frontend_senders.remove(&worker_key),
            WorkerType::ComputeNode | WorkerType::RiseCtl => {
                core_guard.hummock_senders.remove(&worker_key)
            }
            WorkerType::Compactor => core_guard.compactor_senders.remove(&worker_key),
            _ => unreachable!(),
        };
    }

    /// Tell `NotificationManagerCore` to insert sender by `worker_type`.
    pub async fn insert_sender(
        &self,
        subscribe_type: SubscribeType,
        worker_key: WorkerKey,
        sender: UnboundedSender<Notification>,
    ) {
        let mut core_guard = self.core.lock().await;
        let senders = match subscribe_type {
            SubscribeType::Frontend => &mut core_guard.frontend_senders,
            SubscribeType::Hummock => &mut core_guard.hummock_senders,
            SubscribeType::Compactor => &mut core_guard.compactor_senders,
            _ => unreachable!(),
        };

        senders.insert(worker_key, sender);
    }

    pub async fn insert_local_sender(&self, sender: UnboundedSender<LocalNotification>) {
        let mut core_guard = self.core.lock().await;
        core_guard.local_senders.push(sender);
    }

    pub async fn current_version(&self) -> NotificationVersion {
        let core_guard = self.core.lock().await;
        core_guard.current_version.version()
    }
}

struct NotificationManagerCore<S> {
    /// The notification sender to frontends.
    frontend_senders: HashMap<WorkerKey, UnboundedSender<Notification>>,
    /// The notification sender to nodes that subscribes the hummock.
    hummock_senders: HashMap<WorkerKey, UnboundedSender<Notification>>,
    /// The notification sender to compactor nodes.
    compactor_senders: HashMap<WorkerKey, UnboundedSender<Notification>>,
    /// The notification sender to local subscribers.
    local_senders: Vec<UnboundedSender<LocalNotification>>,

    /// The current notification version.
    current_version: Version,
    meta_store: Arc<S>,
}

impl<S> NotificationManagerCore<S>
where
    S: MetaStore,
{
    async fn new(meta_store: Arc<S>) -> Self {
        Self {
            frontend_senders: HashMap::new(),
            hummock_senders: HashMap::new(),
            compactor_senders: HashMap::new(),
            local_senders: vec![],
            current_version: Version::new(&*meta_store).await,
            meta_store,
        }
    }

    async fn notify(&mut self, subscribe_type: SubscribeType, operation: Operation, info: &Info) {
        self.current_version
            .increase_version(&*self.meta_store)
            .await
            .unwrap();
        let senders = match subscribe_type {
            SubscribeType::Frontend => &mut self.frontend_senders,
            SubscribeType::Hummock => &mut self.hummock_senders,
            SubscribeType::Compactor => &mut self.compactor_senders,
            _ => unreachable!(),
        };

        senders.retain(|worker_key, sender| {
            sender
                .send(Ok(SubscribeResponse {
                    status: None,
                    operation: operation as i32,
                    info: Some(info.clone()),
                    version: self.current_version.version(),
                }))
                .inspect_err(|err| {
                    tracing::warn!(
                        "Failed to notify {:?} {:?}: {}",
                        subscribe_type,
                        worker_key,
                        err
                    )
                })
                .is_ok()
        });
    }
}
