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

use async_trait::async_trait;

use crate::source::rabbitmq::{RabbitMQProperties, RabbitMQSplit};
use crate::source::SplitEnumerator;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RabbitMQSplitEnumerator {
    queue_name: String,
}

#[async_trait]
impl SplitEnumerator for RabbitMQSplitEnumerator {
    type Properties = RabbitMQProperties;
    type Split = RabbitMQSplit;

    async fn new(properties: RabbitMQProperties) -> anyhow::Result<RabbitMQSplitEnumerator> {
       
        let queue_name = &properties.queue_name;
        //ensure!(splits.len() == 1, "only support single split");
        assert!(!queue_name.is_empty(), "rabbitmq queue is empty.");
        assert!(!queue_name.trim().len()>0, "rabbitmq queue name len is zero.");
        Ok(Self { queue_name:"queue_name".to_string() })
    }

    async fn list_splits(&mut self) -> anyhow::Result<Vec<RabbitMQSplit>> {
        let mut splits = vec![];
        splits.push(RabbitMQSplit {
            queue_name: self.queue_name.clone(),
        });
        Ok(splits)
    }
}
