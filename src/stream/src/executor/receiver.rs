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
use std::sync::Arc;

use futures::StreamExt;
use risingwave_common::catalog::Schema;
use tokio::sync::mpsc::Receiver;
use tokio_stream::wrappers::ReceiverStream;

use super::{ActorContextRef, OperatorInfoStatus};
use crate::executor::monitor::StreamingMetrics;
use crate::executor::{
    BoxedMessageStream, Executor, ExecutorInfo, Message, PkIndices, PkIndicesRef,
};
use crate::task::ActorId;
/// `ReceiverExecutor` is used along with a channel. After creating a mpsc channel,
/// there should be a `ReceiverExecutor` running in the background, so as to push
/// messages down to the executors.
pub struct ReceiverExecutor {
    receiver: Receiver<Message>,

    /// Logical Operator Info
    info: ExecutorInfo,

    /// Actor operator context
    status: OperatorInfoStatus,

    // Actor id,
    actor_id: ActorId,

    /// Metrics
    metrics: Arc<StreamingMetrics>,
}

impl std::fmt::Debug for ReceiverExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReceiverExecutor")
            .field("schema", &self.info.schema)
            .field("pk_indices", &self.info.pk_indices)
            .finish()
    }
}

impl ReceiverExecutor {
    pub fn new(
        schema: Schema,
        pk_indices: PkIndices,
        receiver: Receiver<Message>,
        actor_context: ActorContextRef,
        receiver_id: u64,
        actor_id: ActorId,
        metrics: Arc<StreamingMetrics>,
    ) -> Self {
        Self {
            receiver,
            info: ExecutorInfo {
                schema,
                pk_indices,
                identity: "ReceiverExecutor".to_string(),
            },
            status: OperatorInfoStatus::new(actor_context, receiver_id),
            actor_id,
            metrics,
        }
    }
}

impl Executor for ReceiverExecutor {
    fn execute(self: Box<Self>) -> BoxedMessageStream {
        let mut status = self.status;
        let metrics = self.metrics.clone();
        let actor_id_str = self.actor_id.to_string();
        ReceiverStream::new(self.receiver)
            .map(move |msg| {
                match &msg {
                    Message::Chunk(chunk) => {
                        metrics
                            .actor_in_record_cnt
                            .with_label_values(&[&actor_id_str])
                            .inc_by(chunk.cardinality() as _);
                    }
                    Message::Barrier(_) => {}
                };
                status.next_message(&msg);
                Ok(msg)
            })
            .boxed()
    }

    fn schema(&self) -> &Schema {
        &self.info.schema
    }

    fn pk_indices(&self) -> PkIndicesRef {
        &self.info.pk_indices
    }

    fn identity(&self) -> &str {
        &self.info.identity
    }
}
