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

use bytes::Bytes;

use crate::source::{SourceMessage, SplitId};

#[derive(Clone, Debug)]
pub struct RabbitMQMessage {
    pub split_id: SplitId,
    pub offset: String,
    pub payload: Bytes,
}


impl From<RabbitMQMessage> for SourceMessage {
    fn from(msg: RabbitMQMessage) -> Self {
        SourceMessage {
            payload: Some(msg.payload),
            offset: msg.offset.clone(),
            split_id: msg.split_id,
        }
    }
}

impl RabbitMQMessage {
    pub fn new(split_id: SplitId, reply: String, data: Vec<u8>) -> Self {
        RabbitMQMessage {
            split_id,
            offset: reply,
            payload: data.into(),
        }
    }
}