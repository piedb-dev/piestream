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
use tokio::sync::mpsc::{Sender, Receiver, UnboundedSender, UnboundedReceiver};

use amqp::{Basic, Session, Channel, Table, protocol};
//use std::default::Default;
use std::thread;
use std::collections::VecDeque;
use core::time::Duration;
use std::borrow::BorrowMut;
use std::sync::{Arc,RwLock};
//use std::cell::RefCell;
//use tokio::sync;

use piestream_common::try_match_expand;
use crate::source::rabbitmq::source::message::RabbitMQMessage;
use crate::source::rabbitmq::{RabbitMQProperties, RabbitMQSplit};
use crate::source::{
    BoxSourceStream, Column, ConnectorState, SourceMessage, SplitImpl, SplitReader,
};
pub struct SendError<T>(pub T);


#[derive(Debug)]
pub struct RabbitMQSplitReader {
    split: RabbitMQSplit,
    receiver: UnboundedReceiver<RabbitMQMessage>,
}

#[derive(Debug, Clone)]
struct Message{
    deliveries_number: u64,
    queue: String,
    body: Vec<u8>,
}

#[derive(Debug, Clone)]
struct MyConsumer {
    deliveries_number: u64,
    queue_name:String,
    sender: UnboundedSender<RabbitMQMessage>,
}

impl amqp::Consumer for MyConsumer {
    fn handle_delivery(&mut self, channel: &mut Channel, deliver: protocol::basic::Deliver, headers: protocol::basic::BasicProperties, body: Vec<u8>){
        //println!("handle_delivery [struct] Content body(as string): {:?}", String::from_utf8(body.clone()));
        //println!("handle_delivery [struct] Content body(as string): {:?}", String::from_utf8(body.clone()));
        println!("handle_delivery [struct] Content body(as string): {:?}", String::from_utf8(body.clone()));
        // DO SOME JOB:
        self.deliveries_number += 1;

        let msg=RabbitMQMessage {
                split_id: self.queue_name.clone().into(),
                offset:deliver.delivery_tag.to_string(),
                payload:body.into(),
        };
        /*let msg=Message{
            deliveries_number:self.deliveries_number ,
            queue: "".to_string(),
            body: body,
        };*/
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
        println!("*****#### new 95");
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

        //let (sender,  receiver) = tokio::sync::mpsc::channel(1);
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        let  my_consumer = MyConsumer { 
                deliveries_number: 0, 
                queue_name:properties.queue_name, 
                sender:sender
            };
        println!("******#### {:?}",&receiver);

        let consumer = channel.basic_consume(my_consumer, queue_name, "".to_string(), false, false, false, false, Table::new());
        println!("*****####  118");
        let mut reader=Self {
            split:split,
            receiver:receiver,
            //sync_call_rx:Arc::new(Mutex::new(sync_call_rx)),
        };
        tokio::time::sleep(Duration::from_secs(1)).await;
        thread::spawn(move || {
            channel.start_consuming();
            println!("*****#### 127");

        });
        Ok(reader)
    }

    fn into_stream(self) -> BoxSourceStream {
        println!("*****#### into_stream 132");
        self.into_stream()
    }

   
}

impl RabbitMQSplitReader {
    #[try_stream(boxed, ok = Vec<SourceMessage>, error = anyhow::Error)]
    pub async fn into_stream(mut self) {
        println!("*****#### into_stream 141");
        let mut interval =tokio::time::interval(Duration::from_millis(10));
        println!("*****#### into_stream 147");
        loop {  
               match  self.receiver.borrow_mut().recv().await  {
                    Some(msg)=>{    
                        println!("*****#### into_stream 151");
                        let mut res = Vec::new();
                        //let m=msg.clone();
                        res.push(SourceMessage::from(msg));
                        println!("*****#### {:?}",&res);
                        yield res;
                    }
                    None =>{
                        // println!("*****#### into_stream None");
                        interval.tick().await;
                        //println!("run interval.tick");
                    }
                }           
        }
        
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_rabbitmq_reader() -> Result<()> {
        let properties=RabbitMQProperties{
            queue_name:"test5".to_string(),
            service_url:"amqp://admin:123456@39.105.209.227//".to_string(),
            auto_ack: Some("false".to_string()),
            consumer_tag: Some("tag".to_string())
        };

        let mut vec=vec![];
        let split=RabbitMQSplit{
            queue_name:"test5".to_string(),
            start_offset:None,
        };
        vec.push(split);
        let v=vec.into_iter().map(SplitImpl::RabbitMQ).collect();
    

        let mut reader=RabbitMQSplitReader::new(properties, Some(v), None)
        .await?
        .into_stream();
        loop {  
            println!("*****#### loop");
            match  reader.next().await{  
                 Some(msg)=>{    
                    let vec=msg?;
                    println!("test {:?}",vec[0].offset);
                 }
                 None =>{
                    tokio::time::sleep(Duration::from_secs(1)).await;
                     //println!("run interval.tick");
                 }
             }           
        }
        //let v=reader.next().await.unwrap()?;
        //println!("v={:?}", v);

        //tokio::time::sleep(Duration::from_secs(300)).await;
       // println!("*****************end.");
        Ok(())

    }
}