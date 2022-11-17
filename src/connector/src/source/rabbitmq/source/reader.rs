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
//pub const QUEUE_SIZE: i32 = 1024;



use anyhow::{anyhow, ensure, Result};
use async_trait::async_trait;
use futures::StreamExt;
use futures_async_stream::try_stream;
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver};

use amqp::{Basic, Session, Channel, Table, protocol};
//use std::default::Default;
use std::thread;
//use std::collections::VecDeque;
use core::time::Duration;

use piestream_common::try_match_expand;
use crate::source::rabbitmq::{RabbitMQProperties, RabbitMQSplit};
use crate::source::{
    BoxSourceStream, Column, ConnectorState, SourceMessage, SplitImpl, SplitReader,
};
use std::sync::Arc;

#[derive(Debug)]
pub struct RabbitMQSplitReader {
    //queue_size: i32,
    //vec_msg: VecDeque<Message>,
    //consumer: Channel,
    split: RabbitMQSplit,
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
    //fn handle_delivery(&mut self, channel: &mut Channel, deliver: protocol::basic::Deliver, headers: protocol::basic::BasicProperties, body: Vec<u8>){
        //println!("[struct] Got a delivery # {}", self.deliveries_number);
        //println!("[struct] Deliver info: {:?}", deliver);
        //println!("[struct] Content headers: {:?}", headers);
        //println!("[struct] Content body: {:?}", String::from_utf8_lossy(body.as_slice()));
        //println!("[struct] Content body(as string): {:?}", String::from_utf8(body));
        // DO SOME JOB:
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
    type Properties = RabbitMQProperties;

    async fn new(
        properties: RabbitMQProperties,
        state: ConnectorState,
        _columns: Option<Vec<Column>>,
    ) -> Result<Self> {
       
        let splits = state.ok_or_else(|| anyhow!("no default state for reader"))?;
        ensure!(splits.len() == 1, "only support single split");
        let split = try_match_expand!(splits.into_iter().next().unwrap(), SplitImpl::RabbitMQ)?;

        let amqp_url = &properties.service_url;
        let queue_name = split.queue_name.to_string();

        tracing::debug!("creating consumer for rabbitmq split queue {}", queue_name,);
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

        /*#[for_await]
        for msg in self.sync_call_rx.borrow_mut().recv() {
            yield msg;
        }*/
    }

    async  fn run(
        self,
        receiver: UnboundedReceiver<Message>,
   ){
       println!("into run.");
       /*let mut interval =tokio::time::interval(Duration::from_millis(10));
       loop {  
           tokio::select! {
               Some(msg) = receiver.borrow_mut().recv() => {
                    self.sync_call_tx.send(msg).unwrap();;
                /*if self.vec_msg.len()<QUEUE_SIZE{
                        v.push_back(msg);
                }*/
                   println!("[struct] Content body(as string): {:?}", msg.deliveries_number);
                   //println!("[struct] Content body(as string): {:?}", String::from_utf8(msg.body))
                   println!("[struct] Content body(as string): {:?}", String::from_utf8(msg.body));
                }
               _ = interval.tick() => {
                   //println!("interval.tick");
               }
           }           
       }*/
   }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_rabbitmq_reader() -> Result<()> {
    
        Ok(())
    }
}
