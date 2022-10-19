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

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;

use piestream_common::util::epoch::Epoch;
use piestream_pb::stream_service::barrier_complete_response::CreateMviewProgress;

use super::command::CommandContext;
use super::notifier::Notifier;
use crate::model::ActorId;
use crate::storage::MetaStore;

type CreateMviewEpoch = Epoch;

#[derive(Clone, Copy)]
enum ChainState {
    ConsumingSnapshot,
    ConsumingUpstream(Epoch),
    Done,
}

/// Progress of all actors containing chain nodes while creating mview.
struct Progress {
    states: HashMap<ActorId, ChainState>,

    done_count: usize,
}

impl Progress {
    /// Create a [`Progress`] for some creating mview, with all `actors` containing the chain nodes.
    fn new(actors: impl IntoIterator<Item = ActorId>) -> Self {
        let states = actors
            .into_iter()
            .map(|a| (a, ChainState::ConsumingSnapshot))
            .collect::<HashMap<_, _>>();
        assert!(!states.is_empty());

        Self {
            states,
            done_count: 0,
        }
    }

    /// Update the progress of `actor`.
    fn update(&mut self, actor: ActorId, new_state: ChainState) {
        match self.states.get_mut(&actor).unwrap() {
            state @ (ChainState::ConsumingSnapshot | ChainState::ConsumingUpstream(_)) => {
                if matches!(new_state, ChainState::Done) {
                    self.done_count += 1;
                }
                *state = new_state;
            }
            ChainState::Done => panic!("should not report done multiple times"),
        }
    }

    /// Returns whether all chains are done.
    fn is_done(&self) -> bool {
        self.done_count == self.states.len()
    }

    /// Returns the ids of all actors containing the chain nodes for the mview tracked by this
    /// [`Progress`].
    fn actors(&self) -> impl Iterator<Item = ActorId> + '_ {
        self.states.keys().cloned()
    }
}

/// The command tracking by the [`CreateMviewProgressTracker`].
pub(super) struct TrackingCommand<S: MetaStore> {
    /// The context of the command.
    pub context: Arc<CommandContext<S>>,

    /// Should be called when the command is finished.
    pub notifiers: Vec<Notifier>,
}

/// Track the progress of all creating mviews. When creation is done, `notify_finished` will be
/// called on registered notifiers.
pub(super) struct CreateMviewProgressTracker<S: MetaStore> {
    /// Progress of the create-mview DDL indicated by the epoch.
    progress_map: HashMap<CreateMviewEpoch, (Progress, TrackingCommand<S>)>,

    /// Find the epoch of the create-mview DDL by the actor containing the chain node.
    actor_map: HashMap<ActorId, CreateMviewEpoch>,
}

impl<S: MetaStore> CreateMviewProgressTracker<S> {
    pub fn new() -> Self {
        Self {
            progress_map: Default::default(),
            actor_map: Default::default(),
        }
    }

    /// Add a new create-mview DDL command to track.
    ///
    /// If the actors to track is empty, return the given command as it can be finished immediately.
    pub fn add(&mut self, command: TrackingCommand<S>) -> Option<TrackingCommand<S>> {
        let actors = command.context.actors_to_track();
        if actors.is_empty() {
            // The command can be finished immediately.
            return Some(command);
        }

        let ddl_epoch = command.context.curr_epoch;
        for &actor in &actors {
            self.actor_map.insert(actor, ddl_epoch);
        }

        let progress = Progress::new(actors);
        let old = self.progress_map.insert(ddl_epoch, (progress, command));
        assert!(old.is_none());
        None
    }

    /// Update the progress of `actor` according to the Prost struct.
    ///
    /// If all actors in this MV have finished, returns the command.
    pub fn update(&mut self, progress: &CreateMviewProgress) -> Option<TrackingCommand<S>> {
        let actor = progress.chain_actor_id;
        let Some(epoch) = self.actor_map.get(&actor).copied() else {
            panic!("no tracked progress for actor {}, is it already finished?", actor);
        };

        let new_state = if progress.done {
            ChainState::Done
        } else {
            ChainState::ConsumingUpstream(progress.consumed_epoch.into())
        };

        match self.progress_map.entry(epoch) {
            Entry::Occupied(mut o) => {
                let progress = &mut o.get_mut().0;
                progress.update(actor, new_state);

                if progress.is_done() {
                    tracing::debug!("all actors done for creating mview with epoch {}!", epoch);

                    // Clean-up the mapping from actors to DDL epoch.
                    for actor in o.get().0.actors() {
                        self.actor_map.remove(&actor);
                    }
                    Some(o.remove().1)
                } else {
                    None
                }
            }
            Entry::Vacant(_) => {
                tracing::warn!(
                    "update the progress of an inexistent create-mview DDL: {progress:?}"
                );
                None
            }
        }
    }
}
