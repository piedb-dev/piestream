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

use std::env;

use anyhow::{bail, Result};
use risingwave_pb::common::WorkerType;
use risingwave_rpc_client::MetaClient;

pub struct MetaServiceOpts {
    pub meta_addr: String,
}

impl MetaServiceOpts {
    /// Recover meta service options from env variable
    ///
    /// Currently, we will read these variables for meta:
    ///
    /// * `RW_META_ADDR`: meta service address
    pub fn from_env() -> Result<Self> {
        let meta_addr = match env::var("RW_META_ADDR") {
            Ok(url) => {
                tracing::info!("using meta addr from `RW_META_ADDR`: {}", url);
                url
            }
            Err(_) => {
                bail!("env variable `RW_META_ADDR` not found, please do one of the following:\n* use `./risedev ctl` to start risectl.\n* `source .risingwave/config/risectl-env` or `source ~/risingwave-deploy/risectl-env` before running risectl.\n* manually set `RW_META_ADDR` in env variable.\nrisectl requires a full persistent cluster to operate, so please also remember to add `use: minio` to risedev config.");
            }
        };
        Ok(Self { meta_addr })
    }

    /// Create meta client from options, and register as rise-ctl worker
    pub async fn create_meta_client(&self) -> Result<MetaClient> {
        let mut client = MetaClient::new(&self.meta_addr).await?;
        // FIXME: don't use 127.0.0.1 for ctl
        let worker_id = client
            .register(&"127.0.0.1:2333".parse().unwrap(), WorkerType::RiseCtl)
            .await?;
        tracing::info!("registered as RiseCtl worker, worker_id = {}", worker_id);
        // TODO: remove worker node
        client.set_worker_id(worker_id);
        Ok(client)
    }
}
