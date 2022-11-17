use serde_with::formats::Strict;
use anyhow::{anyhow, ensure, Result};
use async_trait::async_trait;
use std::thread;
use core::time::Duration;
use futures_async_stream::try_stream;
use amqp::{Basic, Session, Channel, Table, protocol};
use piestream_common::try_match_expand;
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver};
use crate::source::base::{SourceMessage, SplitReader, MAX_CHUNK_SIZE};
use crate::source::{BoxSourceStream, Column, ConnectorState, SplitId, SplitImpl};
use crate::source::rabbitmq::split::RabbitmqSplit;
use crate::source::rabbitmq::RabbitMqProperties;
pub struct RabbitMQSplitReader {
    split: RabbitmqSplit,
    sync_call_tx: UnboundedSender<Message>,
    sync_call_rx: UnboundedReceiver<Message>,
}


#[derive(Debug, Clone)]
struct Message{
    deliveries_number: u64,
    queue: String,
    body: Vec<u8>,
}

struct MyConsumer {
    deliveries_number: u64,
    sender: UnboundedSender<Message>,
}


impl amqp::Consumer for MyConsumer {
    fn handle_delivery(&mut self, channel: &mut Channel, deliver: protocol::basic::Deliver, headers: protocol::basic::BasicProperties, body: Vec<u8>){
        self.deliveries_number += 1;
        let msg=Message{
            deliveries_number:self.deliveries_number ,
            queue: "".to_string(),
            body: body,
        };
        self.sender.send(msg).unwrap();
        channel.basic_ack(deliver.delivery_tag, false).unwrap();
    }
}



#[async_trait]
impl SplitReader for RabbitMQSplitReader {
    type Properties = RabbitMqProperties;
    async fn new(
        properties: RabbitMqProperties,
        state: ConnectorState,
        _columns: Option<Vec<Column>>,
    ) -> Result<Self> {
        let splits = state.ok_or_else(|| anyhow!("no default state for reader"))?;
        ensure!(splits.len() == 1, "only support single split");
        let split = try_match_expand!(splits.into_iter().next().unwrap(), SplitImpl::Rabbitmq)?;

        let amqp_url = &properties.service_url;
        let queue_name = split.queue_name.to_string();

        tracing::debug!("creating consumer for rabbitmq split queue {}", queue_name,);
        println!("amqp_url ====== {:?}",&amqp_url);
        let mut session = match Session::open_url(amqp_url) {
            Ok(session) => session,
            Err(error) => panic!("Can't create session: {:?}", error)
        };

        let mut channel = session.open_channel(1).ok().expect("Can't open channel");
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        let  my_consumer = MyConsumer { deliveries_number: 0, sender:sender };
        let consumer = channel.basic_consume(my_consumer, queue_name, "".to_string(), false, false, false, false, Table::new());

        thread::spawn(move || {
            channel.start_consuming();
        });
        tokio::time::sleep(Duration::from_secs(1)).await;
        //tokio::spawn( async move { run(receiver).await;} );

        let (sync_call_tx, sync_call_rx) = tokio::sync::mpsc::unbounded_channel();
        Ok(Self {
            //queue_size:QUEUE_SIZE,
            //vec_msg:vec_msg,
            //consumer:channel,
            split:split,
            sync_call_tx:sync_call_tx,
            sync_call_rx: sync_call_rx,
        })
        
    }

    fn into_stream(self) -> BoxSourceStream {
        self.into_stream()
    }
}


impl RabbitMQSplitReader {
    #[try_stream(boxed, ok = Vec<SourceMessage>, error = anyhow::Error)]
    pub async fn into_stream(self) {
        let mut res = Vec::new();
        let msg=SourceMessage{
            payload:None,
            offset:"0".to_string(),
            split_id: "1".into()
        };
        res.push(msg);
        yield res;
    }
}

