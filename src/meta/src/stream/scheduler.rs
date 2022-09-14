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

use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicUsize, Ordering};

use piestream_common::buffer::BitmapBuilder;
use piestream_common::error::ErrorCode::InternalError;
use piestream_common::error::{internal_error, Result};
use piestream_common::types::VIRTUAL_NODE_COUNT;
use piestream_common::util::compress::compress_data;
use piestream_pb::common::{ActorInfo, ParallelUnit, ParallelUnitMapping, ParallelUnitType};
use piestream_pb::meta::table_fragments::fragment::FragmentDistributionType;
use piestream_pb::meta::table_fragments::Fragment;

use super::record_table_vnode_mappings;
use crate::cluster::{ClusterManagerRef, WorkerId, WorkerLocations};
use crate::manager::HashMappingManagerRef;
use crate::model::ActorId;
use crate::storage::MetaStore;

/// [`Scheduler`] defines schedule logic for mv actors.
pub struct Scheduler<S: MetaStore> {
    cluster_manager: ClusterManagerRef<S>,
    /// Maintains vnode mappings of all scheduled fragments.
    hash_mapping_manager: HashMappingManagerRef,
    /// Round robin counter for singleton fragments
    single_rr: AtomicUsize,
}
/// [`ScheduledLocations`] represents the location of scheduled result.
pub struct ScheduledLocations {
    /// actor location map.
    pub actor_locations: BTreeMap<ActorId, ParallelUnit>,
    /// worker location map.
    pub node_locations: WorkerLocations,
}

impl ScheduledLocations {
    pub fn new() -> Self {
        Self {
            actor_locations: BTreeMap::new(),
            node_locations: HashMap::new(),
        }
    }

    /// [`Self::node_actors`] returns all actors for every node.
    pub fn node_actors(&self) -> HashMap<WorkerId, Vec<ActorId>> {
        let mut node_actors = HashMap::new();
        self.actor_locations
            .iter()
            .for_each(|(actor_id, parallel_unit)| {
                node_actors
                    .entry(parallel_unit.worker_node_id)
                    .or_insert_with(Vec::new)
                    .push(*actor_id);
            });

        node_actors
    }

    /// [`Self::actor_info_map`] returns the `ActorInfo` map for every actor.
    pub fn actor_info_map(&self) -> HashMap<ActorId, ActorInfo> {
        self.actor_locations
            .iter()
            .map(|(actor_id, parallel_unit)| {
                (
                    *actor_id,
                    ActorInfo {
                        actor_id: *actor_id,
                        host: self.node_locations[&parallel_unit.worker_node_id]
                            .host
                            .clone(),
                    },
                )
            })
            .collect::<HashMap<_, _>>()
    }

    /// [`Self::actor_infos`] returns the `ActorInfo` slice.
    pub fn actor_infos(&self) -> Vec<ActorInfo> {
        self.actor_locations
            .iter()
            .map(|(actor_id, parallel_unit)| ActorInfo {
                actor_id: *actor_id,
                host: self.node_locations[&parallel_unit.worker_node_id]
                    .host
                    .clone(),
            })
            .collect::<Vec<_>>()
    }

    /// Find a placement location that is on the same node of given actor ids.
    pub fn schedule_colocate_with(&self, actor_ids: &[ActorId]) -> Result<ParallelUnit> {
        let mut result_location = None;
        for actor_id in actor_ids {
            let location = self.actor_locations.get(actor_id);
            if let Some(location) = location {
                if result_location.is_none() {
                    result_location = Some(location.clone());
                } else if location != result_location.as_ref().unwrap() {
                    return Err(internal_error(format!(
                        "cannot satisfy placement rule: {} is at {:?}, while others are on {:?}",
                        actor_id,
                        location,
                        result_location.as_ref().unwrap()
                    )));
                }
            } else {
                return Err(internal_error(format!(
                    "actor location not found: {}",
                    actor_id
                )));
            }
        }
        Ok(result_location.unwrap())
    }
}

impl<S> Scheduler<S>
where
    S: MetaStore,
{
    pub fn new(
        cluster_manager: ClusterManagerRef<S>,
        hash_mapping_manager: HashMappingManagerRef,
    ) -> Self {
        Self {
            cluster_manager,
            hash_mapping_manager,
            single_rr: AtomicUsize::new(0),
        }
    }

    /// [`Self::schedule`] schedules input fragments to different parallel units (workers).
    /// The schedule procedure is two-fold:
    /// (1) For singleton fragments, we apply the round robin strategy. One single parallel unit in
    /// the cluster is assigned to a singleton fragment once, and all the single parallel units take
    /// turns.
    /// (2) For normal fragments, we schedule them to all the hash parallel units in the cluster.
    pub async fn schedule(
        &self,
        fragment: &mut Fragment,
        locations: &mut ScheduledLocations,
    ) -> Result<()> {
        if fragment.actors.is_empty() {
            return Err(InternalError("fragment has no actor".to_string()).into());
        }

        self.hash_mapping_manager.set_need_consolidation(true);

        if fragment.distribution_type == FragmentDistributionType::Single as i32 {
            // Singleton fragment
            let actor = &fragment.actors[0];

            //same_worker_node_as_upstrea在delta节点里赋值，后续还需要研究
            if actor.same_worker_node_as_upstream && !actor.upstream_actor_id.is_empty() {
                // Schedule the fragment to the same parallel unit as upstream.
                let parallel_unit = locations.schedule_colocate_with(&actor.upstream_actor_id)?;

                // Build vnode mapping.
                self.set_fragment_vnode_mapping(fragment, &[parallel_unit.clone()])?;

                // Record actor locations.
                locations
                    .actor_locations
                    .insert(fragment.actors[0].actor_id, parallel_unit);
            } else {
                // Choose one parallel unit to schedule from single parallel units.
                let single_parallel_units = self
                    .cluster_manager
                    .list_parallel_units(Some(ParallelUnitType::Single))
                    .await;
                let single_idx = self
                    .single_rr
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |idx| {
                        Some((idx + 1) % single_parallel_units.len())
                    })
                    .map_err(|_| internal_error("failed to round robin id"))?;

                // Build vnode mapping.
                self.set_fragment_vnode_mapping(
                    fragment,
                    &[single_parallel_units[single_idx].clone()],
                )?;

                // Record actor locations.
                locations.actor_locations.insert(
                    fragment.actors[0].actor_id,
                    single_parallel_units[single_idx].clone(),
                );
            }
        } else {
            // Normal fragment

            // Find out all the hash parallel units in the cluster.
            let mut parallel_units = self
                .cluster_manager
                .list_parallel_units(Some(ParallelUnitType::Hash))
                .await;
            // FIXME(Kexiang): select appropriate parallel_units, currently only support
            // `parallel_degree < parallel_units.size()`
            parallel_units.truncate(fragment.actors.len());

            // Build vnode mapping according to the parallel units.
            self.set_fragment_vnode_mapping(fragment, &parallel_units)?;

            // Find out the vnodes that a parallel unit owns.
            let vnode_mapping = self
                .hash_mapping_manager
                .get_fragment_hash_mapping(&fragment.fragment_id)
                .unwrap();

            let mut vnode_bitmaps = HashMap::new();
            vnode_mapping
                .iter()
                .enumerate()
                .for_each(|(vnode, parallel_unit)| {
                    vnode_bitmaps
                        .entry(*parallel_unit)
                        .or_insert_with(|| BitmapBuilder::zeroed(VIRTUAL_NODE_COUNT))
                        .set(vnode, true);
                });
            let vnode_bitmaps = vnode_bitmaps
                .into_iter()
                .map(|(u, b)| (u, b.finish()))
                .collect::<HashMap<_, _>>();

            // Record actor locations and set vnodes into the actors.
            for (idx, actor) in fragment.actors.iter_mut().enumerate() {
                if actor.same_worker_node_as_upstream && !actor.upstream_actor_id.is_empty() {
                    let parallel_unit =
                        locations.schedule_colocate_with(&actor.upstream_actor_id)?;
                    actor.vnode_bitmap =
                        Some(vnode_bitmaps.get(&parallel_unit.id).unwrap().to_protobuf());
                    locations
                        .actor_locations
                        .insert(actor.actor_id, parallel_unit);
                } else {
                    actor.vnode_bitmap = Some(
                        vnode_bitmaps
                            .get(&parallel_units[idx % parallel_units.len()].id)
                            .unwrap()
                            .to_protobuf(),
                    );
                    locations.actor_locations.insert(
                        actor.actor_id,
                        parallel_units[idx % parallel_units.len()].clone(),
                    );
                }
            }
        }

        Ok(())
    }

    /// `set_fragment_vnode_mapping` works by following steps:
    /// 1. Build a vnode mapping according to parallel units where the fragment is scheduled.
    /// 2. Set the vnode mapping into the fragment.
    /// 3. Record the relationship between state tables and vnode mappings.
    fn set_fragment_vnode_mapping(
        &self,
        fragment: &mut Fragment,
        parallel_units: &[ParallelUnit],
    ) -> Result<()> {
        let vnode_mapping = self
            .hash_mapping_manager
            .build_fragment_hash_mapping(fragment.fragment_id, parallel_units);
        //压缩vnode_mapping，original_indices是位置，data实际就是ParallelUnitId
        let (original_indices, data) = compress_data(&vnode_mapping);
        fragment.vnode_mapping = Some(ParallelUnitMapping {
            original_indices,
            data,
            ..Default::default()
        });
        // Looking at the first actor is enough, since all actors in one fragment have identical
        // state table id.
        let actor = fragment.actors.first().unwrap();
        let stream_node = actor.get_nodes()?;
        record_table_vnode_mappings(
            &self.hash_mapping_manager,
            stream_node,
            fragment.fragment_id,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::time::Duration;

    use itertools::Itertools;
    use piestream_common::buffer::Bitmap;
    use piestream_common::types::VIRTUAL_NODE_COUNT;
    use piestream_pb::common::{HostAddress, WorkerType};
    use piestream_pb::meta::table_fragments::fragment::FragmentDistributionType;
    use piestream_pb::plan_common::TableRefId;
    use piestream_pb::stream_plan::stream_node::NodeBody;
    use piestream_pb::stream_plan::{MaterializeNode, StreamActor, StreamNode, TopNNode};

    use super::*;
    use crate::cluster::{ClusterManager, DEFAULT_WORK_NODE_PARALLEL_DEGREE};
    use crate::manager::MetaSrvEnv;

    #[tokio::test]
    async fn test_schedule() -> Result<()> {
        let env = MetaSrvEnv::for_test().await;
        let cluster_manager =
            Arc::new(ClusterManager::new(env.clone(), Duration::from_secs(3600)).await?);

        let node_count = 4;
        for i in 0..node_count {
            let host = HostAddress {
                host: "127.0.0.1".to_string(),
                port: i as i32,
            };
            cluster_manager
                .add_worker_node(host.clone(), WorkerType::ComputeNode)
                .await?;
            cluster_manager.activate_worker_node(host).await?;
        }

        let scheduler = Scheduler::new(cluster_manager, env.hash_mapping_manager_ref());
        let mut locations = ScheduledLocations::new();

        let mut actor_id = 1u32;
        let mut single_fragments = (1..6u32)
            .map(|id| {
                let fragment = Fragment {
                    fragment_id: id,
                    fragment_type: 0,
                    distribution_type: FragmentDistributionType::Single as i32,
                    actors: vec![StreamActor {
                        actor_id,
                        fragment_id: id,
                        nodes: Some(StreamNode {
                            node_body: Some(NodeBody::TopN(TopNNode {
                                ..Default::default()
                            })),
                            ..Default::default()
                        }),
                        dispatcher: vec![],
                        upstream_actor_id: vec![],
                        same_worker_node_as_upstream: false,
                        vnode_bitmap: None,
                    }],
                    vnode_mapping: None,
                };
                actor_id += 1;
                fragment
            })
            .collect_vec();

        let parallel_degree = DEFAULT_WORK_NODE_PARALLEL_DEGREE - 1;
        let mut normal_fragments = (6..8u32)
            .map(|fragment_id| {
                let actors = (actor_id..actor_id + node_count * parallel_degree as u32)
                    .map(|id| StreamActor {
                        actor_id: id,
                        fragment_id,
                        nodes: Some(StreamNode {
                            node_body: Some(NodeBody::Materialize(MaterializeNode {
                                table_ref_id: Some(TableRefId {
                                    table_id: fragment_id as i32,
                                    ..Default::default()
                                }),
                                ..Default::default()
                            })),
                            ..Default::default()
                        }),
                        dispatcher: vec![],
                        upstream_actor_id: vec![],
                        same_worker_node_as_upstream: false,
                        vnode_bitmap: None,
                    })
                    .collect_vec();
                actor_id += node_count * 7;
                Fragment {
                    fragment_id,
                    fragment_type: 0,
                    distribution_type: FragmentDistributionType::Hash as i32,
                    actors,
                    vnode_mapping: None,
                }
            })
            .collect_vec();

        // Test round robin schedule for singleton fragments
        for fragment in &mut single_fragments {
            scheduler.schedule(fragment, &mut locations).await.unwrap();
        }
        assert_eq!(locations.actor_locations.get(&1).unwrap().id, 0);
        assert_eq!(
            locations.actor_locations.get(&1),
            locations.actor_locations.get(&5)
        );
        for fragment in single_fragments {
            assert_ne!(
                env.hash_mapping_manager()
                    .get_fragment_hash_mapping(&fragment.fragment_id),
                None
            );
            // We use fragment id as table id here.
            assert_eq!(
                env.hash_mapping_manager()
                    .get_table_hash_mapping(&fragment.fragment_id),
                None
            );
            for actor in fragment.actors {
                assert!(actor.vnode_bitmap.is_none());
            }
        }

        // Test normal schedule for other fragments
        for fragment in &mut normal_fragments {
            scheduler.schedule(fragment, &mut locations).await.unwrap();
        }
        assert_eq!(
            locations
                .actor_locations
                .iter()
                .filter(|(actor_id, _)| {
                    normal_fragments[1]
                        .actors
                        .iter()
                        .map(|actor| actor.actor_id)
                        .contains(actor_id)
                })
                .count(),
            node_count as usize * parallel_degree
        );
        for fragment in normal_fragments {
            assert_ne!(
                env.hash_mapping_manager()
                    .get_fragment_hash_mapping(&fragment.fragment_id),
                None
            );
            // We use fragment id as table id here.
            assert_ne!(
                env.hash_mapping_manager()
                    .get_table_hash_mapping(&fragment.fragment_id),
                None
            );
            let mut vnode_sum = 0;
            for actor in fragment.actors {
                vnode_sum += Bitmap::try_from(actor.get_vnode_bitmap()?)?.num_high_bits();
            }
            assert_eq!(vnode_sum as usize, VIRTUAL_NODE_COUNT);
        }

        Ok(())
    }
}
