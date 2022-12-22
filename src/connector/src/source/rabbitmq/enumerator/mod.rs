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
use anyhow::{Context};
use crate::source::rabbitmq::{RabbitMQProperties, RabbitMQSplit};
use crate::source::SplitEnumerator;
use amqp::{Basic, Session, Channel, protocol};
use amqp::ConsumeBuilder;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RabbitMQSplitEnumerator {
    queue_name: String,
    service_url: String
}


#[async_trait]
impl SplitEnumerator for RabbitMQSplitEnumerator {
    type Properties = RabbitMQProperties;
    type Split = RabbitMQSplit;

    async fn new(properties: RabbitMQProperties) -> anyhow::Result<RabbitMQSplitEnumerator> {
       
        let queue_name = &properties.queue_name;
        let service_url = &properties.service_url;
        //ensure!(splits.len() == 1, "only support single split");
        assert!(!queue_name.is_empty(), "rabbitmq queue is empty.");
        assert!(!queue_name.trim().len()>0, "rabbitmq queue name len is zero.");
        Ok(Self { queue_name: queue_name.clone(),service_url: service_url.clone() })
    }

    async fn list_splits(&mut self) -> anyhow::Result<Vec<RabbitMQSplit>> {
        
        let _queue_name = self.fetch_queue_name().await.with_context(|| {
            format!(
                "failed to fetch metadata from rabbitmq (queue not exists)"            )
        })?;
        let mut splits = vec![];
        splits.push(RabbitMQSplit {
            queue_name: self.queue_name.clone(),
            start_offset: None,
        });
        Ok(splits)
    }
}



struct MyConsumer {
    deliveries_number: u64
}
impl amqp::Consumer for MyConsumer {
    fn handle_delivery(&mut self, channel: &mut Channel, deliver: protocol::basic::Deliver, _headers: protocol::basic::BasicProperties, _body: Vec<u8>){
        println!("[struct] Got a delivery # {}", self.deliveries_number);
        self.deliveries_number += 1;
        channel.basic_ack(deliver.delivery_tag, false).unwrap();
    }
}
fn consumer_function(channel: &mut Channel, deliver: protocol::basic::Deliver, _headers: protocol::basic::BasicProperties, _body: Vec<u8>){
    println!("[function] Got a delivery:");
    channel.basic_ack(deliver.delivery_tag, false).unwrap();
}


impl RabbitMQSplitEnumerator {

    async fn fetch_queue_name(&self) -> anyhow::Result<Vec<i32>> {
        let amqp_url = &self.service_url;
        let queue_name = &self.queue_name;
        let mut session = match Session::open_url(amqp_url) {
            Ok(session) => session,
            Err(error) =>{
                panic!("Can't create session: {:?}", error)
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

}
