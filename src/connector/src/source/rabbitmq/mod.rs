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

pub mod enumerator;
pub mod source;
pub mod split;

pub use enumerator::*;
use serde::Deserialize;
pub use split::*;

pub const RABBITMQ_CONNECTOR: &str = "rabbitmq";

#[derive(Clone, Debug, Deserialize)]
pub struct RabbitMQProperties {
    #[serde(rename = "queue.name", alias = "rabbitmq.queue.name")]
    pub queue_name: String,

    #[serde(rename = "service.url", alias = "rabbitmq.service.url")]
    pub service_url: String,

    #[serde(rename = "auto.ack", alias = "rabbitmq.auto.ack")]
    pub auto_ack: Option<bool>,

    #[serde(rename = "consumer.tag", alias = "rabbitmq.consumer.tag")]
    pub consumer_tag: String,
    
    // #[serde(rename = "auto.ack", alias = "rabbitmq.auto.ack")]
    // pub auto_ack: Option<bool>,

    // #[serde(rename = "consumer.tag", alias = "rabbitmq.consumer.tag")]
    // pub consumer_tag: String,
}