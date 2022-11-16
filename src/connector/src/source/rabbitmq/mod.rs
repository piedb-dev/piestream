pub mod enumerator;
pub mod source;
pub mod split;

pub use enumerator::*;
use serde::Deserialize;
pub use split::*;

pub const RABBITMQ_CONNECTOR: &str = "rabbitmq";

#[derive(Clone, Debug, Deserialize)]
pub struct RabbitMqProperties {

    #[serde(rename = "service.url", alias = "rabbitmq.service.url")]
    pub service_url: String,

    // #[serde(rename = "scan.startup.mode", alias = "pulsar.scan.startup.mode")]
    // pub scan_startup_mode: Option<String>,

    // #[serde(rename = "scan.startup.timestamp_millis", alias = "pulsar.time.offset")]
    // pub time_offset: Option<String>,
}
