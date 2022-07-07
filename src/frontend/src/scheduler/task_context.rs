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

use std::sync::Arc;

use piestream_batch::executor::BatchMetrics;
use piestream_batch::task::{BatchTaskContext, TaskOutput, TaskOutputId};
use piestream_common::catalog::SysCatalogReaderRef;
use piestream_common::error::Result;
use piestream_common::util::addr::{is_local_address, HostAddr};
use piestream_source::SourceManagerRef;

use crate::catalog::pg_catalog::SysCatalogReaderImpl;
use crate::session::{AuthContext, FrontendEnv};

/// Batch task execution context in frontend.
#[derive(Clone)]
pub struct FrontendBatchTaskContext {
    env: FrontendEnv,
    auth_context: Arc<AuthContext>,
}

impl FrontendBatchTaskContext {
    pub fn new(env: FrontendEnv, auth_context: Arc<AuthContext>) -> Self {
        Self { env, auth_context }
    }
}

impl BatchTaskContext for FrontendBatchTaskContext {
    fn get_task_output(&self, _task_output_id: TaskOutputId) -> Result<TaskOutput> {
        todo!()
    }

    fn catalog_reader_ref(&self) -> Option<SysCatalogReaderRef> {
        Some(Arc::new(SysCatalogReaderImpl::new(
            self.env.catalog_reader().clone(),
            self.env.user_info_reader().clone(),
            self.env.worker_node_manager_ref(),
            self.env.meta_client_ref(),
            self.auth_context.clone(),
        )))
    }

    fn is_local_addr(&self, peer_addr: &HostAddr) -> bool {
        is_local_address(self.env.server_address(), peer_addr)
    }

    fn source_manager_ref(&self) -> Option<SourceManagerRef> {
        todo!()
    }

    fn state_store(&self) -> Option<piestream_storage::store_impl::StateStoreImpl> {
        todo!()
    }

    fn stats(&self) -> Arc<BatchMetrics> {
        todo!()
    }
}
