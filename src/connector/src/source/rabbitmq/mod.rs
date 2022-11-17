pub mod enumerator;
pub mod source;
pub mod split;

pub use enumerator::*;
use serde::Deserialize;
pub use split::*;

pub const RABBITMQ_CONNECTOR: &str = "rabbitmq";

#[derive(Clone, Debug, Deserialize)]
pub struct RabbitMqProperties {
    #[serde(rename = "queue.name", alias = "rabbitmq.queue.name")]
    pub queue_name: String,

    #[serde(rename = "service.url", alias = "rabbitmq.service.url")]
    pub service_url: String,

    // #[serde(rename = "auto.ack", alias = "rabbitmq.auto.ack")]
    // pub auto_ack: Option<bool>,

    // #[serde(rename = "consumer.tag", alias = "rabbitmq.consumer.tag")]
    // pub consumer_tag: String,
}
