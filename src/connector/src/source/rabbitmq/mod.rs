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
use amqp::{Basic, Session, Channel, protocol};
use amqp::ConsumeBuilder;
use tokio::{runtime::Runtime, time};
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
    pub auto_ack: Option<String>,

    #[serde(rename = "consumer.tag", alias = "rabbitmq.consumer.tag")]
    pub consumer_tag: Option<String>,
    
    // #[serde(rename = "auto.ack", alias = "rabbitmq.auto.ack")]
    // pub auto_ack: Option<bool>,

    // #[serde(rename = "consumer.tag", alias = "rabbitmq.consumer.tag")]
    // pub consumer_tag: String,
}
impl RabbitMQProperties {

    async fn check_queue_name(&self,queue_name: String,amqp_url: String) -> anyhow::Result<Vec<i32>> {
        let mut session = match Session::open_url(&amqp_url) {
            Ok(session) => session,
            Err(e) => {
                return Err(anyhow::Error::from(e));
            }
        };
        let mut channel = session.open_channel(1).unwrap();
        let consume_builder = ConsumeBuilder::new(consumer_function, queue_name);
        match consume_builder.basic_consume(&mut channel) {
            Ok(_consumer) => {
                return Ok(vec![1]);
            },
            Err(e) =>{ 
                return Err(anyhow::Error::from(e));
            }
        }
    }

    async fn check_url_api(&self,queue_name: String,amqp_url: String) -> anyhow::Result<Vec<i32>> {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let res = time::timeout(time::Duration::from_secs(2), async {
                self.open_url_api(queue_name,amqp_url).await;
                time::sleep(time::Duration::from_secs(6)).await;
                33
            });
            match res.await {
                Err(_) => {
                    return Ok(vec![1]);
                }
                Ok(data) => {
                    return Ok(vec![1]);
                }
            };
        })
    }
    async fn open_url_api(&self,queue_name: String,amqp_url: String) -> () {
        // Session::open_url(&amqp_url);
        return ();
    }

}

struct MyConsumer {
    deliveries_number: u64
}
impl amqp::Consumer for MyConsumer {
    fn handle_delivery(&mut self, channel: &mut Channel, deliver: protocol::basic::Deliver, _headers: protocol::basic::BasicProperties, _body: Vec<u8>){
        self.deliveries_number += 1;
        channel.basic_ack(deliver.delivery_tag, false).unwrap();
    }
}
fn consumer_function(channel: &mut Channel, deliver: protocol::basic::Deliver, _headers: protocol::basic::BasicProperties, _body: Vec<u8>){
    channel.basic_ack(deliver.delivery_tag, false).unwrap();
}
