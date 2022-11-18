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

use anyhow::anyhow;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::source::{SplitId, SplitMetaData};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Hash)]
pub struct RabbitMQSplit {
    pub(crate) queue_name: String,
    pub start_offset: Option<u64>,
}

impl SplitMetaData for RabbitMQSplit {
    fn id(&self) -> SplitId {
        // TODO: should avoid constructing a string every time
        self.queue_name.to_string().into()
    }

    fn encode_to_bytes(&self) -> Bytes {
        Bytes::from(serde_json::to_string(self).unwrap())
    }

    fn restore_from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        serde_json::from_slice(bytes).map_err(|e| anyhow!(e))
    }
}

impl RabbitMQSplit {
    pub fn new(queue_name: String) -> RabbitMQSplit {
        RabbitMQSplit {
            queue_name: queue_name.trim().to_string(),
            start_offset: None,
        }
    }
    pub fn copy_with_offset(&self, start_offset: String) -> Self {
        Self {
            queue_name: self.queue_name.clone(),
            start_offset: Some(start_offset.as_str().parse::<u64>().unwrap()),
        }
    }
}
