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

use std::collections::VecDeque;
use std::iter::once;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use anyhow::anyhow;
use piestream_pb::hummock::HummockSnapshot;
use tokio::sync::{oneshot, watch, RwLock};

use super::notifier::Notifier;
use super::{Command, Scheduled};
use crate::hummock::HummockManagerRef;
use crate::storage::MetaStore;
use crate::MetaResult;

/// A queue for scheduling barriers.
///
/// We manually implement one here instead of using channels since we may need to update the front
/// of the queue to add some notifiers for instant flushes.
struct Inner {
    queue: RwLock<VecDeque<Scheduled>>,

    /// When `queue` is not empty anymore, all subscribers of this watcher will be notified.
    changed_tx: watch::Sender<()>,

    /// The numbers of barrier (checkpoint = false) since the last barrier (checkpoint = true)
    num_uncheckpointed_barrier: AtomicUsize,

    /// Force checkpoint in next barrier.
    force_checkpoint: AtomicBool,

    checkpoint_frequency: usize,
}

/// The sender side of the barrier scheduling queue.
/// Can be cloned and held by other managers to schedule and run barriers.
#[derive(Clone)]
pub struct BarrierScheduler<S: MetaStore> {
    inner: Arc<Inner>,

    /// Used for getting the latest snapshot after `FLUSH`.
    hummock_manager: HummockManagerRef<S>,
}

impl<S: MetaStore> BarrierScheduler<S> {
    /// Create a pair of [`BarrierScheduler`] and [`ScheduledBarriers`], for scheduling barriers
    /// from different managers, and executing them in the barrier manager, respectively.
    pub fn new_pair(
        hummock_manager: HummockManagerRef<S>,
        checkpoint_frequency: usize,
    ) -> (Self, ScheduledBarriers) {
        tracing::info!(
            "Starting barrier scheduler with: checkpoint_frequency={:?}",
            checkpoint_frequency,
        );
        let inner = Arc::new(Inner {
            queue: RwLock::new(VecDeque::new()),
            changed_tx: watch::channel(()).0,
            num_uncheckpointed_barrier: AtomicUsize::new(0),
            checkpoint_frequency,
            force_checkpoint: AtomicBool::new(false),
        });

        (
            Self {
                inner: inner.clone(),
                hummock_manager,
            },
            ScheduledBarriers { inner },
        )
    }

    /// Push a scheduled barrier into the queue.
    async fn push(&self, scheduleds: impl IntoIterator<Item = Scheduled>) {
        let mut queue = self.inner.queue.write().await;
        for scheduled in scheduleds {
            queue.push_back(scheduled);
            if queue.len() == 1 {
                self.inner.changed_tx.send(()).ok();
            }
        }
    }

    /// Attach `new_notifiers` to the very first scheduled barrier. If there's no one scheduled, a
    /// default barrier will be created. If `new_checkpoint` is true, the barrier will become a
    /// checkpoint.
    async fn attach_notifiers(&self, new_notifiers: Vec<Notifier>, new_checkpoint: bool) {
        let mut queue = self.inner.queue.write().await;
        match queue.front_mut() {
            Some(Scheduled {
                notifiers,
                checkpoint,
                ..
            }) => {
                notifiers.extend(new_notifiers);
                *checkpoint = *checkpoint || new_checkpoint;
            }
            None => {
                // If no command scheduled, create a periodic barrier by default.
                queue.push_back(Scheduled {
                    notifiers: new_notifiers,
                    command: Command::barrier(),
                    checkpoint: new_checkpoint,
                });
                self.inner.changed_tx.send(()).ok();
            }
        }
    }

    /// Wait for the next barrier to collect. Note that the barrier flowing in our stream graph is
    /// ignored, if exists.
    pub async fn wait_for_next_barrier_to_collect(&self, checkpoint: bool) -> MetaResult<()> {
        let (tx, rx) = oneshot::channel();
        let notifier = Notifier {
            collected: Some(tx),
            ..Default::default()
        };
        self.attach_notifiers(vec![notifier], checkpoint).await;
        rx.await.unwrap()
    }

    /// Run multiple commands and return when they're all completely finished. It's ensured that
    /// multiple commands is executed continuously and atomically.
    pub async fn run_multiple_commands(&self, commands: Vec<Command>) -> MetaResult<()> {
        struct Context {
            collect_rx: oneshot::Receiver<MetaResult<()>>,
            finish_rx: oneshot::Receiver<()>,
        }

        let mut contexts = Vec::with_capacity(commands.len());
        let mut scheduleds = Vec::with_capacity(commands.len());

        for command in commands {
            let (collect_tx, collect_rx) = oneshot::channel();
            let (finish_tx, finish_rx) = oneshot::channel();

            contexts.push(Context {
                collect_rx,
                finish_rx,
            });
            scheduleds.push(Scheduled {
                checkpoint: command.need_checkpoint(),
                command,
                notifiers: once(Notifier {
                    collected: Some(collect_tx),
                    finished: Some(finish_tx),
                    ..Default::default()
                })
                .collect(),
            });
        }

        self.push(scheduleds).await;

        for Context {
            collect_rx,
            finish_rx,
        } in contexts
        {
            // Throw the error if it occurs when collecting this barrier.
            collect_rx
                .await
                .map_err(|e| anyhow!("failed to collect barrier: {}", e))??;

            // Wait for this command to be finished.
            finish_rx
                .await
                .map_err(|e| anyhow!("failed to finish command: {}", e))?;
        }

        Ok(())
    }

    /// Run a command and return when it's completely finished.
    pub async fn run_command(&self, command: Command) -> MetaResult<()> {
        self.run_multiple_commands(vec![command]).await
    }

    /// Flush means waiting for the next barrier to collect.
    pub async fn flush(&self, checkpoint: bool) -> MetaResult<HummockSnapshot> {
        let start = Instant::now();

        tracing::debug!("start barrier flush");
        self.wait_for_next_barrier_to_collect(checkpoint).await?;

        let elapsed = Instant::now().duration_since(start);
        tracing::debug!("barrier flushed in {:?}", elapsed);

        let snapshot = self.hummock_manager.get_last_epoch()?;
        Ok(snapshot)
    }
}

/// The receiver side of the barrier scheduling queue.
/// Held by the [`super::GlobalBarrierManager`] to execute these commands.
pub struct ScheduledBarriers {
    inner: Arc<Inner>,
}

impl ScheduledBarriers {
    /// Pop a scheduled barrier from the queue, or a default checkpoint barrier if not exists.
    pub(super) async fn pop_or_default(&self) -> Scheduled {
        let mut queue = self.inner.queue.write().await;
        let checkpoint = self.try_get_checkpoint();
        let scheduled = match queue.pop_front() {
            Some(mut scheduled) => {
                scheduled.checkpoint = scheduled.checkpoint || checkpoint;
                scheduled
            }
            None => {
                // If no command scheduled, create a periodic barrier by default.
                Scheduled {
                    command: Command::barrier(),
                    notifiers: Default::default(),
                    checkpoint,
                }
            }
        };
        self.update_num_uncheckpointed_barrier(scheduled.checkpoint);
        scheduled
    }

    /// Wait for at least one scheduled barrier in the queue.
    pub(super) async fn wait_one(&self) {
        let queue = self.inner.queue.read().await;
        if queue.len() > 0 {
            return;
        }
        let mut rx = self.inner.changed_tx.subscribe();
        drop(queue);

        rx.changed().await.unwrap();
    }

    /// Clear all queued scheduled barriers, and notify their subscribers with failed as aborted.
    pub(super) async fn abort(&self) {
        let mut queue = self.inner.queue.write().await;
        while let Some(Scheduled { notifiers, .. }) = queue.pop_front() {
            notifiers.into_iter().for_each(|notify| {
                notify.notify_collection_failed(anyhow!("Scheduled barrier abort.").into())
            })
        }
    }

    /// Whether the barrier(checkpoint = true) should be injected.
    fn try_get_checkpoint(&self) -> bool {
        self.inner
            .num_uncheckpointed_barrier
            .load(Ordering::Relaxed)
            >= self.inner.checkpoint_frequency
            || self.inner.force_checkpoint.load(Ordering::Relaxed)
    }

    /// Make the `checkpoint` of the next barrier must be true
    pub(crate) fn force_checkpoint_in_next_barrier(&self) {
        self.inner.force_checkpoint.store(true, Ordering::Relaxed)
    }

    /// Update the `num_uncheckpointed_barrier`
    fn update_num_uncheckpointed_barrier(&self, checkpoint: bool) {
        if checkpoint {
            self.inner
                .num_uncheckpointed_barrier
                .store(0, Ordering::Relaxed);
            self.inner.force_checkpoint.store(false, Ordering::Relaxed);
        } else {
            self.inner
                .num_uncheckpointed_barrier
                .fetch_add(1, Ordering::Relaxed);
        }
    }
}
