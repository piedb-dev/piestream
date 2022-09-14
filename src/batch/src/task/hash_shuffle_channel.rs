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
use std::ops::BitAnd;
use std::option::Option;

use piestream_common::array::DataChunk;
use piestream_common::buffer::Bitmap;
use piestream_common::error::ErrorCode::InternalError;
use piestream_common::error::{Result, ToRwResult};
use piestream_common::util::hash_util::CRC32FastBuilder;
use piestream_pb::batch_plan::exchange_info::HashInfo;
use piestream_pb::batch_plan::*;
use tokio::sync::mpsc;

use crate::task::channel::{ChanReceiver, ChanReceiverImpl, ChanSender, ChanSenderImpl};
use crate::task::data_chunk_in_channel::DataChunkInChannel;
use crate::task::BOUNDED_BUFFER_SIZE;

pub struct HashShuffleSender {
    senders: Vec<mpsc::Sender<Option<DataChunkInChannel>>>,
    hash_info: exchange_info::HashInfo,
}

pub struct HashShuffleReceiver {
    receiver: mpsc::Receiver<Option<DataChunkInChannel>>,
}

//根据hash_info.key计算每行的hashvalue
fn generate_hash_values(chunk: &DataChunk, hash_info: &HashInfo) -> Result<Vec<usize>> {
    let output_count = hash_info.output_count as usize;

    let hasher_builder = CRC32FastBuilder {};

    //chunk计算一个hash值列表（根据pk字段） 返回是output_count取模后的结果
    let hash_values = chunk
        .get_hash_values(
            &hash_info
                .keys
                .iter()
                .map(|key| *key as usize)
                .collect::<Vec<_>>(),
            hasher_builder,
        )
        .map_err(|e| InternalError(format!("get_hash_values:{}", e)))?
        .iter_mut()
        .map(|hash_value| hash_value.hash_code() as usize % output_count)
        .collect::<Vec<_>>();
    Ok(hash_values)
}

/// The returned chunks must have cardinality > 0.
/// 产生output_count个新chunk,每个chunk内容一致，visibility有差异
/// 
fn generate_new_data_chunks(
    chunk: &DataChunk,
    hash_info: &exchange_info::HashInfo,
    hash_values: &[usize],
) -> Vec<DataChunk> {
    let output_count = hash_info.output_count as usize;
    let mut vis_maps = vec![vec![]; output_count];
    hash_values.iter().for_each(|hash| {
        //每个vis_map长度都等于hash_values长度，即chunk记录行数
        for (sink_id, vis_map) in vis_maps.iter_mut().enumerate() {
            //hash（理解为行标识）等于sink_id，设置为true
            if *hash == sink_id {
                vis_map.push(true);
            } else {
                vis_map.push(false);
            }
        }
    });
    let mut res = Vec::with_capacity(output_count);
    for (sink_id, vis_map_vec) in vis_maps.into_iter().enumerate() {
        //映射成bitmap
        let vis_map: Bitmap = vis_map_vec.into_iter().collect();

        let vis_map = if let Some(visibility) = chunk.get_visibility_ref() {
            //两个bitmap做and操作（标识删除状态的数据bitmap）
            vis_map.bitand(visibility)
        } else {
            vis_map
        };
        //根据新的vis_map产生新的chunk
        let new_data_chunk = chunk.with_visibility(vis_map);
        trace!(
            "send to sink:{}, cardinality:{}",
            sink_id,
            new_data_chunk.cardinality()
        );
        res.push(new_data_chunk);
    }
    res
}

impl ChanSender for HashShuffleSender {
    type SendFuture<'a> = impl Future<Output = Result<()>>;

    fn send(&mut self, chunk: Option<DataChunk>) -> Self::SendFuture<'_> {
        async move {
            match chunk {
                Some(c) => self.send_chunk(c).await,
                None => self.send_done().await,
            }
        }
    }
}

impl HashShuffleSender {
    async fn send_chunk(&mut self, chunk: DataChunk) -> Result<()> {
        //计算chunk每行hashvalue根据 self.hash_info.keys
        let hash_values = generate_hash_values(&chunk, &self.hash_info)?;
        //根据hash_info.output_count  产生一组新的chunk
        let new_data_chunks = generate_new_data_chunks(&chunk, &self.hash_info, &hash_values);
        //每组的chunk内容都一致，等于重复了hash_info.output_count次性能是否合适???
        for (sink_id, new_data_chunk) in new_data_chunks.into_iter().enumerate() {
            trace!(
                "send to sink:{}, cardinality:{}",
                sink_id,
                new_data_chunk.cardinality()
            );
            // The reason we need to add this filter only in HashShuffleSender is that
            // `generate_new_data_chunks` may generate an empty chunk.
            if new_data_chunk.cardinality() > 0 {
                self.senders[sink_id]
                    .send(Some(DataChunkInChannel::new(new_data_chunk)))
                    .await
                    .to_rw_result_with(|| "HashShuffleSender::send".into())?;
            }
        }
        Ok(())
    }

    async fn send_done(&mut self) -> Result<()> {
        //发送结束
        for sender in &self.senders {
            sender
                .send(None)
                .await
                .to_rw_result_with(|| "HashShuffleSender::send".into())?;
        }

        Ok(())
    }
}

impl ChanReceiver for HashShuffleReceiver {
    type RecvFuture<'a> = impl Future<Output = Result<Option<DataChunkInChannel>>>;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        async move {
            match self.receiver.recv().await {
                Some(data_chunk) => Ok(data_chunk),
                // Early close should be treated as error.
                None => Err(InternalError("broken hash_shuffle_channel".to_string()).into()),
            }
        }
    }
}

pub fn new_hash_shuffle_channel(shuffle: &ExchangeInfo) -> (ChanSenderImpl, Vec<ChanReceiverImpl>) {
    let hash_info = match shuffle.distribution {
        Some(exchange_info::Distribution::HashInfo(ref v)) => v.clone(),
        _ => exchange_info::HashInfo::default(),
    };

    let output_count = hash_info.output_count as usize;
    let mut senders = Vec::with_capacity(output_count);
    let mut receivers = Vec::with_capacity(output_count);
    //构建output_count个下发通道
    for _ in 0..output_count {
        let (s, r) = mpsc::channel(BOUNDED_BUFFER_SIZE);
        senders.push(s);
        receivers.push(r);
    }
    let channel_sender = ChanSenderImpl::HashShuffle(HashShuffleSender { senders, hash_info });
    let channel_receivers = receivers
        .into_iter()
        .map(|receiver| ChanReceiverImpl::HashShuffle(HashShuffleReceiver { receiver }))
        .collect::<Vec<_>>();
    (channel_sender, channel_receivers)
}
