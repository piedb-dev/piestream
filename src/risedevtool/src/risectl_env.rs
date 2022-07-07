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

use std::collections::HashMap;
use std::process::Command;

use anyhow::Result;

use crate::{add_storage_backend, ServiceConfig};

pub fn compute_risectl_env(services: &HashMap<String, ServiceConfig>) -> Result<String> {
    // Pick one of the compute node and generate risectl config
    for item in services.values() {
        if let ServiceConfig::ComputeNode(c) = item {
            let mut cmd = Command::new("compute-node");
            add_storage_backend(
                "risectl",
                c.provide_minio.as_ref().unwrap(),
                c.provide_aws_s3.as_ref().unwrap(),
                false,
                &mut cmd,
            )?;
            let meta_node = &c.provide_meta_node.as_ref().unwrap()[0];
            return Ok(format!(
                "export RW_HUMMOCK_URL=\"{}\"\nexport RW_META_ADDR=\"http://{}:{}\"",
                cmd.get_args().nth(1).unwrap().to_str().unwrap(),
                meta_node.address,
                meta_node.port
            ));
        }
    }
    Ok("".into())
}
