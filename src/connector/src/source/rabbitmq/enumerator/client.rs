use async_trait::async_trait;
use anyhow::{anyhow, bail, Result};
use futures::future::ok;
use crate::source::SplitEnumerator;
use crate::source::rabbitmq::RabbitMqProperties;
use crate::source::rabbitmq::split::RabbitmqSplit;


pub struct RabbitMQSplitEnumerator {
    broker_address: String,
}

impl RabbitMQSplitEnumerator {}

#[async_trait]
impl SplitEnumerator for RabbitMQSplitEnumerator {
    type Properties = RabbitMqProperties;
    type Split = RabbitmqSplit;
    async fn new(properties: RabbitMqProperties) -> Result<RabbitMQSplitEnumerator> {
        Ok(RabbitMQSplitEnumerator {
            broker_address: "127.0.0.1:1592".to_string(),
        })
    }

    async fn list_splits(&mut self) -> anyhow::Result<Vec<RabbitmqSplit>> {
        let k = vec![RabbitmqSplit {
            queue_name : "hello".to_string(),
        }];
        Ok(k)
    }
}









// extern crate amqp;
// use amqp::Session;

// #[test]
// fn main() {
//     let mut session = Session::open_url("amqp://localhost//").unwrap();
//     let mut channel = session.open_channel(1).unwrap();
// }

