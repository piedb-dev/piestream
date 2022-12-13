use async_trait::async_trait;
use anyhow::{Result};
use crate::source::SplitEnumerator;
use crate::source::rabbitmq::split::RabbitMQSplit;
use crate::source::rabbitmq::RabbitMQProperties;

pub struct RabbitMQSplitEnumerator {
    service_url: String,
    queue_name: String,
    consumer_tag: String
}

impl RabbitMQSplitEnumerator {}

#[async_trait]
impl SplitEnumerator for RabbitMQSplitEnumerator {
    type Properties = RabbitMQProperties;
    type Split = RabbitMQSplit;
    async fn new(properties: RabbitMQProperties) -> Result<RabbitMQSplitEnumerator> {
        Ok(RabbitMQSplitEnumerator {
            service_url: "123456@39.105.209.227".to_string(),
            queue_name: "test".to_string(),
            consumer_tag: '1'.to_string()
        })
    }

    async fn list_splits(&mut self) -> anyhow::Result<Vec<RabbitMQSplit>> {
        let k = vec![RabbitMQSplit {
            queue_name: "test".to_string(),
            start_offset: Some(1)
        }];
        Ok(k)
    }
}

