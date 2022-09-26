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
use std::iter::once;

use piestream_pb::stream_service::barrier_complete_response::CreateMviewProgress;
use tokio::sync::oneshot;

use super::progress::ChainState;
use super::CollectResult;
use crate::executor::Barrier;
use crate::task::ActorId;

/// The state machine of local barrier manager.
#[derive(Debug)]
enum ManagedBarrierStateInner {
    /// Barriers from some actors have been collected and stashed, however no `send_barrier`
    /// request from the meta service is issued.
    /// Barriers被收集和隐藏，但meta service没有‘send_barrier’请求，即不会对外发布
    Stashed {
        /// Actor ids we've collected and stashed.
        collected_actors: HashSet<ActorId>,
    },

    /// Meta service has issued a `send_barrier` request. We're collecting barriers now.
    Issued {
        /// Actor ids remaining to be collected.
        remaining_actors: HashSet<ActorId>,

        /// Notify that the collection is finished.
        collect_notifier: oneshot::Sender<CollectResult>,
    },
}

#[derive(Debug)]
pub(super) struct ManagedBarrierState {
    /// Record barrier state for each epoch of concurrent checkpoints.
    epoch_barrier_state_map: HashMap<u64, ManagedBarrierStateInner>,

    /// Record the progress updates of creating mviews for each epoch of concurrent checkpoints.
    pub(super) create_mview_progress: HashMap<u64, HashMap<ActorId, ChainState>>,
}

impl ManagedBarrierState {
    /// Create a barrier manager state. This will be called only once.
    pub(super) fn new() -> Self {
        Self {
            epoch_barrier_state_map: HashMap::new(),
            create_mview_progress: Default::default(),
        }
    }

    /// Notify if we have collected barriers from all actor ids. The state must be `Issued`.
    fn may_notify(&mut self, curr_epoch: u64) {
        //remaining_actors为空即都处理完才发送
        let to_notify = match self.epoch_barrier_state_map.get(&curr_epoch) {
            Some(ManagedBarrierStateInner::Issued {
                remaining_actors, ..
            }) => (remaining_actors.is_empty()), 
            _ => unreachable!(),
        };

        if to_notify {
            //println!("444444444444444**************create_mview_progress.len={:?}", self.create_mview_progress.len());
            let inner = self.epoch_barrier_state_map.remove(&curr_epoch).unwrap();

            //构建create_mview_progress列表
            let create_mview_progress = self
                .create_mview_progress
                .remove(&curr_epoch)
                .unwrap_or_default()
                .into_iter()
                .map(|(actor, state)| CreateMviewProgress {
                    chain_actor_id: actor,
                    done: matches!(state, ChainState::Done),
                    consumed_epoch: match state {
                        ChainState::ConsumingUpstream(consumed_epoch) => {
                            // assert!(consumed_epoch <=
                            // curr_epoch,"con{:?},cu{:?}",consumed_epoch,curr_epoch);
                            consumed_epoch
                        }
                        ChainState::Done => curr_epoch,
                    },
                })
                .collect();

            //println!("create_mview_progress={:?}", create_mview_progress);
            match inner {
                ManagedBarrierStateInner::Issued {
                    collect_notifier, ..
                } => {
                    // Notify about barrier finishing.
                    let result = CollectResult {
                        create_mview_progress,
                    };
                    if collect_notifier.send(result).is_err() {
                        warn!(
                            "failed to notify barrier collection with epoch {}",
                            curr_epoch
                        )
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    /// Collect a `barrier` from the actor with `actor_id`.
    pub(super) fn collect(&mut self, actor_id: ActorId, barrier: &Barrier) {
        tracing::trace!(
            target: "events::stream::barrier::collect_barrier",
            "collect_barrier: epoch = {}, actor_id = {}, state = {:#?}",
            barrier.epoch.curr,
            actor_id,
            self
        );

        match self.epoch_barrier_state_map.get_mut(&barrier.epoch.curr) {
            
            Some(ManagedBarrierStateInner::Stashed { collected_actors }) => {
                //增加
                let new = collected_actors.insert(actor_id);
                assert!(new);
                //println!("222111111111111111111");
            }
            Some(ManagedBarrierStateInner::Issued {
                remaining_actors, ..
            }) => {
                //删除actor_id
                let exist = remaining_actors.remove(&actor_id);
                assert!(exist);
                //发送消息
                self.may_notify(barrier.epoch.curr);
                //println!("333333222111111111111111111");
            }
            None => {
                //println!("111111111111111111");
                //没有就插入
                self.epoch_barrier_state_map.insert(
                    barrier.epoch.curr,
                    ManagedBarrierStateInner::Stashed {
                        collected_actors: once(actor_id).collect(),
                    },
                );
            }
        }
    }

    /// When the meta service issues a `send_barrier` request, call this function to transform to
    /// `Issued` and start to collect or to notify.
    pub(super) fn transform_to_issued(
        &mut self,
        barrier: &Barrier,
        actor_ids_to_collect: impl IntoIterator<Item = ActorId>,
        collect_notifier: oneshot::Sender<CollectResult>,
    ) {
        //转换为Issued状态
        let inner = match self.epoch_barrier_state_map.get(&barrier.epoch.curr) {
            Some(ManagedBarrierStateInner::Stashed { collected_actors }) => {
                ///TOTO(liqiu):逻辑有些不理解
                ///actor_ids_to_collect减去collected_actors
                ///即在actor_ids_to_collect列表不在collected_actors数据转ManagedBarrierStateInner::Issued状态
                let remaining_actors = actor_ids_to_collect
                    .into_iter()
                    .filter(|a| !collected_actors.contains(a))
                    .collect();
                println!("remaining_actors={:?}", remaining_actors);
                ManagedBarrierStateInner::Issued {
                    remaining_actors,
                    collect_notifier,
                }
            }
            Some(ManagedBarrierStateInner::Issued { .. }) => {
                panic!(
                    "barrier epochs{:?} state has already been `Issued`",
                    barrier.epoch
                );
            }
            None => {
                let remaining_actors = actor_ids_to_collect.into_iter().collect();
                ManagedBarrierStateInner::Issued {
                    remaining_actors,
                    collect_notifier,
                }
            }
        };
        //重新插入
        self.epoch_barrier_state_map
            .insert(barrier.epoch.curr, inner);
        self.may_notify(barrier.epoch.curr);
    }
}
