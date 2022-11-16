use serde_with::formats::Strict;
use anyhow::{anyhow, ensure, Result};
use async_trait::async_trait;
use futures_async_stream::try_stream;
use crate::source::base::{SourceMessage, SplitReader, MAX_CHUNK_SIZE};
use crate::source::{BoxSourceStream, Column, ConnectorState, SplitId, SplitImpl};
use crate::source::rabbitmq::split::RabbitmqSplit;
use crate::source::rabbitmq::RabbitMqProperties;
pub struct RabbitMQSplitReader {
    server: String,
}

#[async_trait]
impl SplitReader for RabbitMQSplitReader {
    type Properties = RabbitMqProperties;
    async fn new(
        properties: RabbitMqProperties,
        state: ConnectorState,
        _columns: Option<Vec<Column>>,
    ) -> Result<Self> {
        
    }

    fn into_stream(self) -> BoxSourceStream {
        self.into_stream()
    }
}


impl RabbitMQSplitReader {
    #[try_stream(boxed, ok = Vec<SourceMessage>, error = anyhow::Error)]
    pub async fn into_stream(self) {
        #[for_await]
        for msgs in self.consumer.ready_chunks(MAX_CHUNK_SIZE) {
            let mut res = Vec::with_capacity(msgs.len());
            for msg in msgs {
                res.push(SourceMessage::from(msg?));
            }
            yield res;
        }
    }
}

