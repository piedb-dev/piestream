use async_trait::async_trait;
use anyhow::{anyhow, bail, Result};
use futures::future::ok;
use crate::source::SplitEnumerator;
use crate::source::rabbitmq::split::RabbitMQSplit;
use crate::source::rabbitmq::RabbitMQProperties;

pub struct RabbitMQSplitEnumerator {
    broker_address: String,
}

impl RabbitMQSplitEnumerator {}

#[async_trait]
impl SplitEnumerator for RabbitMQSplitEnumerator {
    type Properties = RabbitMQProperties;
    type Split = RabbitMQSplit;
    async fn new(properties: RabbitMQProperties) -> Result<RabbitMQSplitEnumerator> {
        Ok(RabbitMQSplitEnumerator {
            broker_address: "127.0.0.1:1592".to_string(),
        })
    }

    async fn list_splits(&mut self) -> anyhow::Result<Vec<RabbitMQSplit>> {
        let k = vec![RabbitMQSplit {
            queue_name : "hello".to_string(),
        }];
        Ok(k)
    }
}

