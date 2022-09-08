// Copyright 2022 PieDb Data
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

use std::future::Future;

use piestream_common::array::DataChunk;
use piestream_common::error::Result;
use piestream_pb::batch_plan::exchange_info::DistributionMode as ShuffleDistributionMode;
use piestream_pb::batch_plan::ExchangeInfo;

use crate::task::broadcast_channel::{new_broadcast_channel, BroadcastReceiver, BroadcastSender};
use crate::task::data_chunk_in_channel::DataChunkInChannel;
use crate::task::fifo_channel::{new_fifo_channel, FifoReceiver, FifoSender};
use crate::task::hash_shuffle_channel::{
    new_hash_shuffle_channel, HashShuffleReceiver, HashShuffleSender,
};

pub(super) trait ChanSender: Send {
    type SendFuture<'a>: Future<Output = Result<()>> + Send
    where
        Self: 'a;
    /// This function will block until there's enough resource to process the chunk.
    /// Currently, it will only be called from single thread.
    /// `None` is sent as a mark of the ending of channel.
    fn send(&mut self, chunk: Option<DataChunk>) -> Self::SendFuture<'_>;
}

pub enum ChanSenderImpl {
    HashShuffle(HashShuffleSender),
    Fifo(FifoSender),
    Broadcast(BroadcastSender),
}

impl ChanSenderImpl {
    pub(super) async fn send(&mut self, chunk: Option<DataChunk>) -> Result<()> {
        match self {
            Self::HashShuffle(sender) => sender.send(chunk).await,
            Self::Fifo(sender) => sender.send(chunk).await,
            Self::Broadcast(sender) => sender.send(chunk).await,
        }
    }
}

pub(super) trait ChanReceiver: Send {
    type RecvFuture<'a>: Future<Output = Result<Option<DataChunkInChannel>>> + Send
    where
        Self: 'a;
    /// Returns `None` if there's no more data to read.
    /// Otherwise it will wait until there's data.
    fn recv(&mut self) -> Self::RecvFuture<'_>;
}

pub enum ChanReceiverImpl {
    HashShuffle(HashShuffleReceiver),
    Fifo(FifoReceiver),
    Broadcast(BroadcastReceiver),
}

impl ChanReceiverImpl {
    pub(super) async fn recv(&mut self) -> Result<Option<DataChunkInChannel>> {
        match self {
            Self::HashShuffle(receiver) => receiver.recv().await,
            Self::Broadcast(receiver) => receiver.recv().await,
            Self::Fifo(receiver) => receiver.recv().await,
        }
    }
}

/// Output-channel is a synchronous, bounded single-producer-multiple-consumer queue.
/// The producer is the local task executor, the consumer is
/// [`ExchangeService`](piestream_pb::task_service::exchange_service_server::ExchangeService).
/// The implementation depends on the shuffling strategy.
pub fn create_output_channel(
    shuffle: &ExchangeInfo,
) -> Result<(ChanSenderImpl, Vec<ChanReceiverImpl>)> {
    //下发的模式
    match shuffle.get_mode()? {
        ShuffleDistributionMode::Single => Ok(new_fifo_channel()),
        ShuffleDistributionMode::Hash => Ok(new_hash_shuffle_channel(shuffle)),
        ShuffleDistributionMode::Broadcast => Ok(new_broadcast_channel(shuffle)),
    }
}
