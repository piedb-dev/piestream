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

use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{Stream, StreamExt};

pub struct MergeStream<S: Stream + Unpin> {
    sources: Vec<S>,
    last_base: usize,
}

impl<S: Stream + Unpin> MergeStream<S> {
    fn new(sources: Vec<S>) -> Self {
        Self {
            sources,
            last_base: 0,
        }
    }
}

impl<S: Stream + Unpin> Stream for MergeStream<S> {
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        println!("poll_next**************************************");
        let mut poll_count = 0;
        while poll_count < self.sources.len() {
            let idx = (poll_count + self.last_base) % self.sources.len();
            match self.sources[idx].poll_next_unpin(cx) {
                Poll::Ready(Some(item)) => {
                    //已经读到数据，立刻返回，所以并没有顺序
                    self.last_base = (idx + 1) % self.sources.len();
                    return Poll::Ready(Some(item));
                }
                Poll::Ready(None) => {
                    //idx和列表最后一个元素交换，并从sources列表中删除最后一个节点并返回
                    let _ = self.sources.swap_remove(idx);
                    // read from the front or we may miss the stream just moved from the back.
                    //设置为0从新扫描，就是怕删除swap节点遍历不到
                    poll_count = 0;
                    continue;
                }
                Poll::Pending => {
                    poll_count += 1;
                    continue;
                }
            }
        }
        //都在Pending状态
        if !self.sources.is_empty() {
            Poll::Pending
        } else {
            //全部都读完
            Poll::Ready(None)
        }
    }
}

pub fn select_all<S: Stream + Unpin>(streams: Vec<S>) -> MergeStream<S> {
    MergeStream::new(streams)
}
