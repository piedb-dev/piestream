// Copyright 2022 Singularity Data
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

use itertools::Itertools;
use piestream_pb::meta::list_table_fragments_response::{
    ActorInfo, FragmentInfo, TableFragmentInfo,
};
use piestream_pb::meta::stream_manager_service_server::StreamManagerService;
use piestream_pb::meta::*;
use tonic::{Request, Response, Status};

use crate::barrier::BarrierManagerRef;
use crate::manager::MetaSrvEnv;
use crate::storage::MetaStore;
use crate::stream::FragmentManagerRef;

pub type TonicResponse<T> = Result<Response<T>, Status>;

#[derive(Clone)]
pub struct StreamServiceImpl<S>
where
    S: MetaStore,
{
    env: MetaSrvEnv<S>,
    barrier_manager: BarrierManagerRef<S>,
    fragment_manager: FragmentManagerRef<S>,
}

impl<S> StreamServiceImpl<S>
where
    S: MetaStore,
{
    pub fn new(
        env: MetaSrvEnv<S>,
        barrier_manager: BarrierManagerRef<S>,
        fragment_manager: FragmentManagerRef<S>,
    ) -> Self {
        StreamServiceImpl {
            env,
            barrier_manager,
            fragment_manager,
        }
    }
}

#[async_trait::async_trait]
impl<S> StreamManagerService for StreamServiceImpl<S>
where
    S: MetaStore,
{
    #[cfg_attr(coverage, no_coverage)]
    async fn flush(&self, request: Request<FlushRequest>) -> TonicResponse<FlushResponse> {
        self.env.idle_manager().record_activity();
        let _req = request.into_inner();

        self.barrier_manager.flush().await?;
        Ok(Response::new(FlushResponse { status: None }))
    }

    #[cfg_attr(coverage, no_coverage)]
    async fn list_table_fragments(
        &self,
        request: Request<ListTableFragmentsRequest>,
    ) -> Result<Response<ListTableFragmentsResponse>, Status> {
        let req = request.into_inner();
        let table_ids = HashSet::<u32>::from_iter(req.table_ids);
        let table_fragments = self.fragment_manager.list_table_fragments().await?;
        let info = table_fragments
            .into_iter()
            .filter(|tf| table_ids.contains(&tf.table_id().table_id))
            .map(|tf| {
                (
                    tf.table_id().table_id,
                    TableFragmentInfo {
                        fragments: tf
                            .fragments
                            .into_iter()
                            .map(|(id, fragment)| FragmentInfo {
                                id,
                                actors: fragment
                                    .actors
                                    .into_iter()
                                    .map(|actor| ActorInfo {
                                        id: actor.actor_id,
                                        node: actor.nodes,
                                        dispatcher: actor.dispatcher,
                                    })
                                    .collect_vec(),
                            })
                            .collect_vec(),
                    },
                )
            })
            .collect::<HashMap<u32, TableFragmentInfo>>();

        Ok(Response::new(ListTableFragmentsResponse {
            table_fragments: info,
        }))
    }
}
