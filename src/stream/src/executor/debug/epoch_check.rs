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

use futures_async_stream::try_stream;

use crate::executor::error::StreamExecutorError;
use crate::executor::{ExecutorInfo, Message, MessageStream};

/// Streams wrapped by `epoch_check` will check whether the first message received is a barrier, and
/// the epoch in the barriers are monotonically increasing.
#[try_stream(ok = Message, error = StreamExecutorError)]
pub async fn epoch_check(info: Arc<ExecutorInfo>, input: impl MessageStream) {
    // Epoch number recorded from last barrier message.
    let mut last_epoch = None;

    #[for_await]
    for message in input {
        //println!("************");
        let message = message?;

        if let Message::Barrier(b) = &message {
            let new_epoch = b.epoch.curr;
            //new_epoch必须大于等于last_epoch
            let stale = last_epoch
                .map(|last_epoch| last_epoch > new_epoch)
                .unwrap_or(false);

            if stale {
                panic!(
                    "epoch check failed on {}: last epoch is {:?}, while the epoch of incoming barrier is {}.\nstale barrier: {:?}",
                    info.identity,
                    last_epoch,
                    new_epoch,
                    b
                );
            }
            last_epoch = Some(new_epoch);
        } else if last_epoch.is_none() && !info.identity.contains("BatchQuery") {
            panic!(
                "epoch check failed on {}: the first message must be a barrier",
                info.identity
            )
        }

        yield message;
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use futures::{pin_mut, StreamExt};
    use piestream_common::array::StreamChunk;

    use super::*;
    use crate::executor::test_utils::MockSource;
    use crate::executor::Executor;

    #[tokio::test]
    async fn test_epoch_ok() {
        let (mut tx, source) = MockSource::channel(Default::default(), vec![]);
        tx.push_barrier(100, false);
        tx.push_chunk(StreamChunk::default());
        tx.push_barrier(114, false);
        tx.push_barrier(114, false);
        tx.push_barrier(514, false);

        let checked = epoch_check(source.info().into(), source.boxed().execute());
        pin_mut!(checked);

        assert_matches!(checked.next().await.unwrap().unwrap(), Message::Barrier(b) if b.epoch.curr == 100);
        assert_matches!(checked.next().await.unwrap().unwrap(), Message::Chunk(_));
        assert_matches!(checked.next().await.unwrap().unwrap(), Message::Barrier(b) if b.epoch.curr == 114);
        assert_matches!(checked.next().await.unwrap().unwrap(), Message::Barrier(b) if b.epoch.curr == 114);
        assert_matches!(checked.next().await.unwrap().unwrap(), Message::Barrier(b) if b.epoch.curr == 514);
    }

    #[should_panic]
    #[tokio::test]
    async fn test_epoch_bad() {
        let (mut tx, source) = MockSource::channel(Default::default(), vec![]);
        tx.push_barrier(100, false);
        tx.push_chunk(StreamChunk::default());
        tx.push_barrier(514, false);
        tx.push_barrier(514, false);
        tx.push_barrier(114, false);

        let checked = epoch_check(source.info().into(), source.boxed().execute());
        pin_mut!(checked);

        assert_matches!(checked.next().await.unwrap().unwrap(), Message::Barrier(b) if b.epoch.curr == 100);
        assert_matches!(checked.next().await.unwrap().unwrap(), Message::Chunk(_));
        assert_matches!(checked.next().await.unwrap().unwrap(), Message::Barrier(b) if b.epoch.curr == 514);
        assert_matches!(checked.next().await.unwrap().unwrap(), Message::Barrier(b) if b.epoch.curr == 514);

        checked.next().await.unwrap().unwrap(); // should panic
    }

    #[should_panic]
    #[tokio::test]
    async fn test_epoch_first_not_barrier() {
        let (mut tx, source) = MockSource::channel(Default::default(), vec![]);
        tx.push_chunk(StreamChunk::default());
        tx.push_barrier(114, false);

        let checked = epoch_check(source.info().into(), source.boxed().execute());
        pin_mut!(checked);

        checked.next().await.unwrap().unwrap(); // should panic
    }

    #[tokio::test]
    async fn test_empty() {
        let (_, mut source) = MockSource::channel(Default::default(), vec![]);
        source = source.stop_on_finish(false);
        let checked = epoch_check(source.info().into(), source.boxed().execute());
        pin_mut!(checked);
        assert!(checked.next().await.transpose().unwrap().is_none());
    }
}
