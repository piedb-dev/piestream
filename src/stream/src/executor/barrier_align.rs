// Copyright 2022 Singularity Data
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

use futures::future::{select, Either};
use futures::StreamExt;
use futures_async_stream::try_stream;

use super::error::StreamExecutorError;
use super::{Barrier, BoxedMessageStream, Message, StreamChunk};

#[derive(Debug, PartialEq)]
pub enum AlignedMessage {
    Left(StreamChunk),
    Right(StreamChunk),
    Barrier(Barrier),
}

#[try_stream(ok = AlignedMessage, error = StreamExecutorError)]
pub async fn barrier_align(mut left: BoxedMessageStream, mut right: BoxedMessageStream) {
    // TODO: handle stream end
    loop {
        match select(left.next(), right.next()).await {
            Either::Left((None, _)) => {
                // left stream end, passthrough right chunks
                while let Some(msg) = right.next().await {
                    match msg? {
                        Message::Chunk(chunk) => yield AlignedMessage::Right(chunk),
                        Message::Barrier(_) => {
                            panic!("right barrier received while left stream end")
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
                            panic!("left barrier received while right stream end")
                        }
                    }
                }
                break;
            }
            Either::Left((Some(msg), _)) => match msg? {
                Message::Chunk(chunk) => yield AlignedMessage::Left(chunk),
                Message::Barrier(_) => loop {
                    // received left barrier, waiting for right barrier
                    match right.next().await.unwrap()? {
                        Message::Chunk(chunk) => yield AlignedMessage::Right(chunk),
                        Message::Barrier(barrier) => {
                            yield AlignedMessage::Barrier(barrier);
                            break;
                        }
                    }
                },
            },
            Either::Right((Some(msg), _)) => match msg? {
                Message::Chunk(chunk) => yield AlignedMessage::Right(chunk),
                Message::Barrier(_) => loop {
                    // received right barrier, waiting for left barrier
                    match left.next().await.unwrap()? {
                        Message::Chunk(chunk) => yield AlignedMessage::Left(chunk),
                        Message::Barrier(barrier) => {
                            yield AlignedMessage::Barrier(barrier);
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
    use futures::TryStreamExt;
    use madsim::time::sleep;
    use risingwave_common::array::stream_chunk::StreamChunkTestExt;

    use super::*;

    #[madsim::test]
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
        let output: Vec<_> = barrier_align(left, right).try_collect().await.unwrap();
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

    #[madsim::test]
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
        let _output: Vec<_> = barrier_align(left, right).try_collect().await.unwrap();
    }

    #[madsim::test]
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
        let _output: Vec<_> = barrier_align(left, right).try_collect().await.unwrap();
    }
}
