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

use std::sync::Arc;
use std::time::Instant;

use anyhow::Context;
use enum_as_inner::EnumAsInner;
use futures::future::{select, Either};
use futures::StreamExt;
use futures_async_stream::try_stream;
use piestream_common::bail;

use super::error::StreamExecutorError;
use super::{Barrier, BoxedMessageStream, Message, StreamChunk, StreamExecutorResult};
use crate::executor::monitor::StreamingMetrics;
use crate::task::ActorId;

pub type AlignedMessageStreamItem = StreamExecutorResult<AlignedMessage>;
pub trait AlignedMessageStream = futures::Stream<Item = AlignedMessageStreamItem> + Send;

#[derive(Debug, EnumAsInner, PartialEq)]
pub enum AlignedMessage {
    Barrier(Barrier),
    Left(StreamChunk),
    Right(StreamChunk),
}

#[try_stream(ok = AlignedMessage, error = StreamExecutorError)]
pub async fn barrier_align(
    mut left: BoxedMessageStream,
    mut right: BoxedMessageStream,
    actor_id: ActorId,
    metrics: Arc<StreamingMetrics>,
) {
    let actor_id = actor_id.to_string();
    loop {
        let prefer_left: bool = rand::random();
        let select_result = if prefer_left {
            select(left.next(), right.next()).await
        } else {
            match select(right.next(), left.next()).await {
                Either::Left(x) => Either::Right(x),
                Either::Right(x) => Either::Left(x),
            }
        };
        match select_result {
            Either::Left((None, _)) => {
                // left stream end, passthrough right chunks
                while let Some(msg) = right.next().await {
                    match msg? {
                        Message::Chunk(chunk) => yield AlignedMessage::Right(chunk),
                        Message::Barrier(_) => {
                            bail!("right barrier received while left stream end");
                        }
                    }
                }
                break;
            }
            Either::Right((None, _)) => {
                // right stream end, passthrough left chunks
                while let Some(msg) = left.next().await {
                    match msg? {
                        Message::Chunk(chunk) => yield AlignedMessage::Left(chunk),
                        Message::Barrier(_) => {
                            bail!("left barrier received while right stream end");
                        }
                    }
                }
                break;
            }
            Either::Left((Some(msg), _)) => match msg? {
                Message::Chunk(chunk) => yield AlignedMessage::Left(chunk),
                Message::Barrier(_) => loop {
                    let start_time = Instant::now();
                    // received left barrier, waiting for right barrier
                    match right
                        .next()
                        .await
                        .context("failed to poll right message, stream closed unexpectedly")??
                    {
                        Message::Chunk(chunk) => yield AlignedMessage::Right(chunk),
                        Message::Barrier(barrier) => {
                            yield AlignedMessage::Barrier(barrier);
                            metrics
                                .join_barrier_align_duration
                                .with_label_values(&[&actor_id, "right"])
                                .observe(start_time.elapsed().as_secs_f64());
                            break;
                        }
                    }
                },
            },
            Either::Right((Some(msg), _)) => match msg? {
                Message::Chunk(chunk) => yield AlignedMessage::Right(chunk),
                Message::Barrier(_) => loop {
                    let start_time = Instant::now();
                    // received right barrier, waiting for left barrier
                    match left
                        .next()
                        .await
                        .context("failed to poll left message, stream closed unexpectedly")??
                    {
                        Message::Chunk(chunk) => yield AlignedMessage::Left(chunk),
                        Message::Barrier(barrier) => {
                            yield AlignedMessage::Barrier(barrier);
                            metrics
                                .join_barrier_align_duration
                                .with_label_values(&[&actor_id, "left"])
                                .observe(start_time.elapsed().as_secs_f64());
                            break;
                        }
                    }
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use async_stream::try_stream;
    use futures::{Stream, TryStreamExt};
    use piestream_common::array::stream_chunk::StreamChunkTestExt;
    use tokio::time::sleep;

    use super::*;

    fn barrier_align_for_test(
        left: BoxedMessageStream,
        right: BoxedMessageStream,
    ) -> impl Stream<Item = Result<AlignedMessage, StreamExecutorError>> {
        barrier_align(left, right, 0, Arc::new(StreamingMetrics::unused()))
    }

    #[tokio::test]
    async fn test_barrier_align() {
        let left = try_stream! {
            yield Message::Chunk(StreamChunk::from_pretty("I\n + 1"));
            yield Message::Barrier(Barrier::new_test_barrier(1));
            yield Message::Chunk(StreamChunk::from_pretty("I\n + 2"));
            yield Message::Barrier(Barrier::new_test_barrier(2));
        }
        .boxed();
        let right = try_stream! {
            sleep(Duration::from_millis(1)).await;
            yield Message::Chunk(StreamChunk::from_pretty("I\n + 1"));
            yield Message::Barrier(Barrier::new_test_barrier(1));
            yield Message::Barrier(Barrier::new_test_barrier(2));
            yield Message::Chunk(StreamChunk::from_pretty("I\n + 3"));
        }
        .boxed();
        let output: Vec<_> = barrier_align_for_test(left, right)
            .try_collect()
            .await
            .unwrap();
        assert_eq!(
            output,
            vec![
                AlignedMessage::Left(StreamChunk::from_pretty("I\n + 1")),
                AlignedMessage::Right(StreamChunk::from_pretty("I\n + 1")),
                AlignedMessage::Barrier(Barrier::new_test_barrier(1)),
                AlignedMessage::Left(StreamChunk::from_pretty("I\n + 2")),
                AlignedMessage::Barrier(Barrier::new_test_barrier(2)),
                AlignedMessage::Right(StreamChunk::from_pretty("I\n + 3")),
            ]
        );
    }

    #[tokio::test]
    #[should_panic]
    async fn left_barrier_right_end_1() {
        let left = try_stream! {
            sleep(Duration::from_millis(1)).await;
            yield Message::Chunk(StreamChunk::from_pretty("I\n + 1"));
            yield Message::Barrier(Barrier::new_test_barrier(1));
        }
        .boxed();
        let right = try_stream! {
            yield Message::Chunk(StreamChunk::from_pretty("I\n + 1"));
        }
        .boxed();
        let _output: Vec<_> = barrier_align_for_test(left, right)
            .try_collect()
            .await
            .unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn left_barrier_right_end_2() {
        let left = try_stream! {
            yield Message::Chunk(StreamChunk::from_pretty("I\n + 1"));
            yield Message::Barrier(Barrier::new_test_barrier(1));
        }
        .boxed();
        let right = try_stream! {
            sleep(Duration::from_millis(1)).await;
            yield Message::Chunk(StreamChunk::from_pretty("I\n + 1"));
        }
        .boxed();
        let _output: Vec<_> = barrier_align_for_test(left, right)
            .try_collect()
            .await
            .unwrap();
    }
}
