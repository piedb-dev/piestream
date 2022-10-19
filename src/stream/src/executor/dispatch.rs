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

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::future::Future;
use std::iter::repeat_with;
use std::sync::Arc;

use async_stack_trace::StackTrace;
use futures::Stream;
use futures_async_stream::try_stream;
use itertools::Itertools;
use piestream_common::array::{Op, StreamChunk};
use piestream_common::buffer::BitmapBuilder;
use piestream_common::util::compress::decompress_data;
use piestream_common::util::hash_util::Crc32FastBuilder;
use piestream_pb::stream_plan::update_mutation::DispatcherUpdate as ProstDispatcherUpdate;
use piestream_pb::stream_plan::Dispatcher as ProstDispatcher;
use smallvec::{smallvec, SmallVec};
use tracing::event;

use super::exchange::output::{new_output, BoxedOutput};
use crate::error::StreamResult;
use crate::executor::monitor::StreamingMetrics;
use crate::executor::{Barrier, BoxedExecutor, Message, Mutation, StreamConsumer};
use crate::task::{ActorId, DispatcherId, SharedContext};

/// [`DispatchExecutor`] consumes messages and send them into downstream actors. Usually,
/// data chunks will be dispatched with some specified policy, while control message
/// such as barriers will be distributed to all receivers.
pub struct DispatchExecutor {
    input: BoxedExecutor,
    inner: DispatchExecutorInner,
}

struct DispatchExecutorInner {
    dispatchers: Vec<DispatcherImpl>,
    actor_id: u32,
    actor_id_str: String,
    context: Arc<SharedContext>,
    metrics: Arc<StreamingMetrics>,
}

impl DispatchExecutorInner {
    fn single_inner_mut(&mut self) -> &mut DispatcherImpl {
        assert_eq!(
            self.dispatchers.len(),
            1,
            "only support mutation on one-dispatcher actors"
        );
        &mut self.dispatchers[0]
    }

    async fn dispatch(&mut self, msg: Message) -> StreamResult<()> {
        match msg {
            Message::Chunk(chunk) => {
                self.metrics
                    .actor_out_record_cnt
                    .with_label_values(&[&self.actor_id_str])
                    .inc_by(chunk.cardinality() as _);
                let start_time = minstant::Instant::now();
                if self.dispatchers.len() == 1 {
                    // special clone optimization when there is only one downstream dispatcher
                    self.single_inner_mut().dispatch_data(chunk).await?;
                } else {
                    for dispatcher in &mut self.dispatchers {
                        dispatcher.dispatch_data(chunk.clone()).await?;
                    }
                }
                self.metrics
                    .actor_output_buffer_blocking_duration_ns
                    .with_label_values(&[&self.actor_id_str])
                    .inc_by(start_time.elapsed().as_nanos() as u64);
            }
            Message::Barrier(barrier) => {
                let start_time = minstant::Instant::now();
                let mutation = barrier.mutation.clone();
                self.pre_mutate_dispatchers(&mutation)?;
                for dispatcher in &mut self.dispatchers {
                    dispatcher.dispatch_barrier(barrier.clone()).await?;
                }
                self.post_mutate_dispatchers(&mutation)?;
                self.metrics
                    .actor_output_buffer_blocking_duration_ns
                    .with_label_values(&[&self.actor_id_str])
                    .inc_by(start_time.elapsed().as_nanos() as u64);
            }
        };
        Ok(())
    }

    /// Add new dispatchers to the executor. Will check whether their ids are unique.
    fn add_dispatchers<'a>(
        &mut self,
        new_dispatchers: impl IntoIterator<Item = &'a ProstDispatcher>,
    ) -> StreamResult<()> {
        let new_dispatchers: Vec<_> = new_dispatchers
            .into_iter()
            .map(|d| DispatcherImpl::new(&self.context, self.actor_id, d))
            .try_collect()?;

        self.dispatchers.extend(new_dispatchers);

        assert!(
            self.dispatchers
                .iter()
                .map(|d| d.dispatcher_id())
                .all_unique(),
            "dispatcher ids must be unique: {:?}",
            self.dispatchers
        );

        Ok(())
    }

    fn find_dispatcher(&mut self, dispatcher_id: DispatcherId) -> &mut DispatcherImpl {
        self.dispatchers
            .iter_mut()
            .find(|d| d.dispatcher_id() == dispatcher_id)
            .unwrap_or_else(|| panic!("dispatcher {}:{} not found", self.actor_id, dispatcher_id))
    }

    /// Update the dispatcher BEFORE we actually dispatch this barrier. We'll only add the new
    /// outputs.
    fn pre_update_dispatcher(&mut self, update: &ProstDispatcherUpdate) -> StreamResult<()> {
        let outputs: Vec<_> = update
            .added_downstream_actor_id
            .iter()
            .map(|&id| new_output(&self.context, self.actor_id, id))
            .try_collect()?;

        let dispatcher = self.find_dispatcher(update.dispatcher_id);
        dispatcher.add_outputs(outputs);

        Ok(())
    }

    /// Update the dispatcher AFTER we dispatch this barrier. We'll remove some outputs and finally
    /// update the hash mapping.
    fn post_update_dispatcher(&mut self, update: &ProstDispatcherUpdate) -> StreamResult<()> {
        let ids = update.removed_downstream_actor_id.iter().copied().collect();

        let dispatcher = self.find_dispatcher(update.dispatcher_id);
        dispatcher.remove_outputs(&ids);

        match dispatcher {
            // The hash mapping is only used by the hash dispatcher.
            DispatcherImpl::Hash(dispatcher) => {
                dispatcher.hash_mapping = {
                    let compressed_mapping = update.get_hash_mapping()?;
                    decompress_data(
                        &compressed_mapping.original_indices,
                        &compressed_mapping.data,
                    )
                }
            }
            _ => assert!(update.hash_mapping.is_none()),
        }

        Ok(())
    }

    /// For `Add` and `Update`, update the dispatchers before we dispatch the barrier.
    fn pre_mutate_dispatchers(&mut self, mutation: &Option<Arc<Mutation>>) -> StreamResult<()> {
        let Some(mutation) = mutation.as_deref() else {
            return Ok(())
        };

        match mutation {
            Mutation::Add { adds, .. } => {
                if let Some(new_dispatchers) = adds.get(&self.actor_id) {
                    self.add_dispatchers(new_dispatchers)?;
                }
            }
            Mutation::Update { dispatchers, .. } => {
                if let Some(updates) = dispatchers.get(&self.actor_id) {
                    for update in updates {
                        self.pre_update_dispatcher(update)?;
                    }
                }
            }
            _ => {}
        };

        Ok(())
    }

    /// For `Stop` and `Update`, update the dispatchers after we dispatch the barrier.
    fn post_mutate_dispatchers(&mut self, mutation: &Option<Arc<Mutation>>) -> StreamResult<()> {
        let Some(mutation) = mutation.as_deref() else {
            return Ok(())
        };

        match mutation {
            Mutation::Stop(stops) => {
                // Remove outputs only if this actor itself is not to be stopped.
                if !stops.contains(&self.actor_id) {
                    for dispatcher in &mut self.dispatchers {
                        dispatcher.remove_outputs(stops);
                    }
                }
            }
            Mutation::Update { dispatchers, .. } => {
                if let Some(updates) = dispatchers.get(&self.actor_id) {
                    for update in updates {
                        self.post_update_dispatcher(update)?;
                    }
                }
            }

            _ => {}
        };

        // After stopping the downstream mview, the outputs of some dispatcher might be empty and we
        // should clean up them.
        self.dispatchers.drain_filter(|d| d.is_empty());

        Ok(())
    }
}

impl DispatchExecutor {
    pub fn new(
        input: BoxedExecutor,
        dispatchers: Vec<DispatcherImpl>,
        actor_id: u32,
        context: Arc<SharedContext>,
        metrics: Arc<StreamingMetrics>,
    ) -> Self {
        Self {
            input,
            inner: DispatchExecutorInner {
                dispatchers,
                actor_id,
                actor_id_str: actor_id.to_string(),
                context,
                metrics,
            },
        }
    }
}

impl StreamConsumer for DispatchExecutor {
    type BarrierStream = impl Stream<Item = StreamResult<Barrier>> + Send;

    fn execute(mut self: Box<Self>) -> Self::BarrierStream {
        #[try_stream]
        async move {
            let input = self.input.execute();

            #[for_await]
            for msg in input {
                let msg: Message = msg?;
                let barrier = msg.as_barrier().cloned();
                self.inner
                    .dispatch(msg)
                    .stack_trace(if barrier.is_some() {
                        "dispatch_barrier"
                    } else {
                        "dispatch_chunk"
                    })
                    .await?;
                if let Some(barrier) = barrier {
                    yield barrier;
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum DispatcherImpl {
    Hash(HashDataDispatcher),
    Broadcast(BroadcastDispatcher),
    Simple(SimpleDispatcher),
    RoundRobin(RoundRobinDataDispatcher),
}

impl DispatcherImpl {
    pub fn new(
        context: &SharedContext,
        actor_id: ActorId,
        dispatcher: &ProstDispatcher,
    ) -> StreamResult<Self> {
        let outputs = dispatcher
            .downstream_actor_id
            .iter()
            .map(|&down_id| new_output(context, actor_id, down_id))
            .collect::<StreamResult<Vec<_>>>()?;

        use piestream_pb::stream_plan::DispatcherType::*;
        let dispatcher_impl = match dispatcher.get_type()? {
            Hash => {
                assert!(!outputs.is_empty());
                let column_indices = dispatcher
                    .column_indices
                    .iter()
                    .map(|i| *i as usize)
                    .collect();

                let hash_mapping = {
                    let compressed_mapping = dispatcher.get_hash_mapping()?;
                    decompress_data(
                        &compressed_mapping.original_indices,
                        &compressed_mapping.data,
                    )
                };

                DispatcherImpl::Hash(HashDataDispatcher::new(
                    outputs,
                    column_indices,
                    hash_mapping,
                    dispatcher.dispatcher_id,
                ))
            }
            Broadcast => DispatcherImpl::Broadcast(BroadcastDispatcher::new(
                outputs,
                dispatcher.dispatcher_id,
            )),
            Simple | NoShuffle => {
                let [output]: [_; 1] = outputs.try_into().unwrap();
                DispatcherImpl::Simple(SimpleDispatcher::new(output, dispatcher.dispatcher_id))
            }
            Unspecified => unreachable!(),
        };

        Ok(dispatcher_impl)
    }
}

macro_rules! impl_dispatcher {
    ($( { $variant_name:ident } ),*) => {
        impl DispatcherImpl {
            pub async fn dispatch_data(&mut self, chunk: StreamChunk) -> StreamResult<()> {
                match self {
                    $( Self::$variant_name(inner) => inner.dispatch_data(chunk).await, )*
                }
            }

            pub async fn dispatch_barrier(&mut self, barrier: Barrier) -> StreamResult<()> {
                match self {
                    $( Self::$variant_name(inner) => inner.dispatch_barrier(barrier).await, )*
                }
            }

            pub fn add_outputs(&mut self, outputs: impl IntoIterator<Item = BoxedOutput>) {
                match self {
                    $(Self::$variant_name(inner) => inner.add_outputs(outputs), )*
                }
            }

            pub fn remove_outputs(&mut self, actor_ids: &HashSet<ActorId>) {
                match self {
                    $(Self::$variant_name(inner) => inner.remove_outputs(actor_ids), )*
                }
            }

            pub fn dispatcher_id(&self) -> DispatcherId {
                match self {
                    $(Self::$variant_name(inner) => inner.dispatcher_id(), )*
                }
            }

            pub fn is_empty(&self) -> bool {
                match self {
                    $(Self::$variant_name(inner) => inner.is_empty(), )*
                }
            }
        }
    }
}

macro_rules! for_all_dispatcher_variants {
    ($macro:ident) => {
        $macro! {
            { Hash },
            { Broadcast },
            { Simple },
            { RoundRobin }
        }
    };
}

for_all_dispatcher_variants! { impl_dispatcher }

macro_rules! define_dispatcher_associated_types {
    () => {
        type DataFuture<'a> = impl DispatchFuture<'a>;
        type BarrierFuture<'a> = impl DispatchFuture<'a>;
    };
}

pub trait DispatchFuture<'a> = Future<Output = StreamResult<()>> + Send;

pub trait Dispatcher: Debug + 'static {
    type DataFuture<'a>: DispatchFuture<'a>;
    type BarrierFuture<'a>: DispatchFuture<'a>;

    /// Dispatch a data chunk to downstream actors.
    fn dispatch_data(&mut self, chunk: StreamChunk) -> Self::DataFuture<'_>;
    /// Dispatch a barrier to downstream actors, generally by broadcasting it.
    fn dispatch_barrier(&mut self, barrier: Barrier) -> Self::BarrierFuture<'_>;

    /// Add new outputs to the dispatcher.
    fn add_outputs(&mut self, outputs: impl IntoIterator<Item = BoxedOutput>);
    /// Remove outputs to `actor_ids` from the dispatcher.
    fn remove_outputs(&mut self, actor_ids: &HashSet<ActorId>);

    /// The ID of the dispatcher. A [`DispatchExecutor`] may have multiple dispatchers with
    /// different IDs.
    fn dispatcher_id(&self) -> DispatcherId;

    /// Whether the dispatcher has no outputs. If so, it'll be cleaned up from the
    /// [`DispatchExecutor`].
    fn is_empty(&self) -> bool;
}

#[derive(Debug)]
pub struct RoundRobinDataDispatcher {
    outputs: Vec<BoxedOutput>,
    cur: usize,
    dispatcher_id: DispatcherId,
}

impl RoundRobinDataDispatcher {
    pub fn new(outputs: Vec<BoxedOutput>, dispatcher_id: DispatcherId) -> Self {
        Self {
            outputs,
            cur: 0,
            dispatcher_id,
        }
    }
}

impl Dispatcher for RoundRobinDataDispatcher {
    define_dispatcher_associated_types!();

    fn dispatch_data(&mut self, chunk: StreamChunk) -> Self::DataFuture<'_> {
        async move {
            self.outputs[self.cur].send(Message::Chunk(chunk)).await?;
            self.cur += 1;
            self.cur %= self.outputs.len();
            Ok(())
        }
    }

    fn dispatch_barrier(&mut self, barrier: Barrier) -> Self::BarrierFuture<'_> {
        async move {
            // always broadcast barrier
            for output in &mut self.outputs {
                output.send(Message::Barrier(barrier.clone())).await?;
            }
            Ok(())
        }
    }

    fn add_outputs(&mut self, outputs: impl IntoIterator<Item = BoxedOutput>) {
        self.outputs.extend(outputs.into_iter());
    }

    fn remove_outputs(&mut self, actor_ids: &HashSet<ActorId>) {
        self.outputs
            .drain_filter(|output| actor_ids.contains(&output.actor_id()))
            .count();
        self.cur = self.cur.min(self.outputs.len() - 1);
    }

    fn dispatcher_id(&self) -> DispatcherId {
        self.dispatcher_id
    }

    fn is_empty(&self) -> bool {
        self.outputs.is_empty()
    }
}

pub struct HashDataDispatcher {
    outputs: Vec<BoxedOutput>,
    keys: Vec<usize>,
    /// Mapping from virtual node to actor id, used for hash data dispatcher to dispatch tasks to
    /// different downstream actors.
    hash_mapping: Vec<ActorId>,
    dispatcher_id: DispatcherId,
}

impl Debug for HashDataDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HashDataDispatcher")
            .field("outputs", &self.outputs)
            .field("keys", &self.keys)
            .field("dispatcher_id", &self.dispatcher_id)
            .finish_non_exhaustive()
    }
}

impl HashDataDispatcher {
    pub fn new(
        outputs: Vec<BoxedOutput>,
        keys: Vec<usize>,
        hash_mapping: Vec<ActorId>,
        dispatcher_id: DispatcherId,
    ) -> Self {
        Self {
            outputs,
            keys,
            hash_mapping,
            dispatcher_id,
        }
    }
}

impl Dispatcher for HashDataDispatcher {
    define_dispatcher_associated_types!();

    fn add_outputs(&mut self, outputs: impl IntoIterator<Item = BoxedOutput>) {
        self.outputs.extend(outputs.into_iter());
    }

    fn dispatch_barrier(&mut self, barrier: Barrier) -> Self::BarrierFuture<'_> {
        async move {
            // always broadcast barrier
            for output in &mut self.outputs {
                output.send(Message::Barrier(barrier.clone())).await?;
            }
            Ok(())
        }
    }

    fn dispatch_data(&mut self, chunk: StreamChunk) -> Self::DataFuture<'_> {
        async move {
            // A chunk can be shuffled into multiple output chunks that to be sent to downstreams.
            // In these output chunks, the only difference are visibility map, which is calculated
            // by the hash value of each line in the input chunk.
            let num_outputs = self.outputs.len();

            // get hash value of every line by its key
            let hash_builder = Crc32FastBuilder {};
            let vnodes = chunk
                .data_chunk()
                .get_hash_values(&self.keys, hash_builder)
                .into_iter()
                .map(|hash| hash.to_vnode())
                .collect_vec();

            tracing::trace!(target: "events::stream::dispatch::hash", "\n{}\n keys {:?} => {:?}", chunk.to_pretty_string(), self.keys, vnodes);

            let mut vis_maps = repeat_with(|| BitmapBuilder::with_capacity(chunk.capacity()))
                .take(num_outputs)
                .collect_vec();
            let mut last_vnode_when_update_delete = 0;
            let mut new_ops: Vec<Op> = Vec::with_capacity(chunk.capacity());

            let (ops, columns, visibility) = chunk.into_inner();

            match visibility {
                None => {
                    vnodes.iter().zip_eq(ops).for_each(|(vnode, op)| {
                        // get visibility map for every output chunk
                        for (output, vis_map) in self.outputs.iter().zip_eq(vis_maps.iter_mut()) {
                            vis_map.append(self.hash_mapping[*vnode as usize] == output.actor_id());
                        }
                        // The 'update' message, noted by an UpdateDelete and a successive
                        // UpdateInsert, need to be rewritten to common
                        // Delete and Insert if they were dispatched to
                        // different actors.
                        if op == Op::UpdateDelete {
                            last_vnode_when_update_delete = *vnode;
                        } else if op == Op::UpdateInsert {
                            if *vnode != last_vnode_when_update_delete {
                                new_ops.push(Op::Delete);
                                new_ops.push(Op::Insert);
                            } else {
                                new_ops.push(Op::UpdateDelete);
                                new_ops.push(Op::UpdateInsert);
                            }
                        } else {
                            new_ops.push(op);
                        }
                    });
                }
                Some(visibility) => {
                    vnodes
                        .iter()
                        .zip_eq(visibility.iter())
                        .zip_eq(ops)
                        .for_each(|((vnode, visible), op)| {
                            for (output, vis_map) in self.outputs.iter().zip_eq(vis_maps.iter_mut())
                            {
                                vis_map.append(
                                    visible
                                        && self.hash_mapping[*vnode as usize] == output.actor_id(),
                                );
                            }
                            if !visible {
                                new_ops.push(op);
                                return;
                            }
                            if op == Op::UpdateDelete {
                                last_vnode_when_update_delete = *vnode;
                            } else if op == Op::UpdateInsert {
                                if *vnode != last_vnode_when_update_delete {
                                    new_ops.push(Op::Delete);
                                    new_ops.push(Op::Insert);
                                } else {
                                    new_ops.push(Op::UpdateDelete);
                                    new_ops.push(Op::UpdateInsert);
                                }
                            } else {
                                new_ops.push(op);
                            }
                        });
                }
            }

            let ops = new_ops;

            // individually output StreamChunk integrated with vis_map
            for (vis_map, output) in vis_maps.into_iter().zip_eq(self.outputs.iter_mut()) {
                let vis_map = vis_map.finish();
                // columns is not changed in this function
                let new_stream_chunk =
                    StreamChunk::new(ops.clone(), columns.clone(), Some(vis_map));
                if new_stream_chunk.cardinality() > 0 {
                    event!(
                        tracing::Level::TRACE,
                        msg = "chunk",
                        downstream = output.actor_id(),
                        "send = \n{:#?}",
                        new_stream_chunk
                    );
                    output.send(Message::Chunk(new_stream_chunk)).await?;
                }
            }
            Ok(())
        }
    }

    fn remove_outputs(&mut self, actor_ids: &HashSet<ActorId>) {
        self.outputs
            .drain_filter(|output| actor_ids.contains(&output.actor_id()))
            .count();
    }

    fn dispatcher_id(&self) -> DispatcherId {
        self.dispatcher_id
    }

    fn is_empty(&self) -> bool {
        self.outputs.is_empty()
    }
}

/// `BroadcastDispatcher` dispatches message to all outputs.
#[derive(Debug)]
pub struct BroadcastDispatcher {
    outputs: HashMap<ActorId, BoxedOutput>,
    dispatcher_id: DispatcherId,
}

impl BroadcastDispatcher {
    pub fn new(
        outputs: impl IntoIterator<Item = BoxedOutput>,
        dispatcher_id: DispatcherId,
    ) -> Self {
        Self {
            outputs: Self::into_pairs(outputs).collect(),
            dispatcher_id,
        }
    }

    fn into_pairs(
        outputs: impl IntoIterator<Item = BoxedOutput>,
    ) -> impl Iterator<Item = (ActorId, BoxedOutput)> {
        outputs
            .into_iter()
            .map(|output| (output.actor_id(), output))
    }
}

impl Dispatcher for BroadcastDispatcher {
    define_dispatcher_associated_types!();

    fn dispatch_data(&mut self, chunk: StreamChunk) -> Self::DataFuture<'_> {
        async move {
            for output in self.outputs.values_mut() {
                output.send(Message::Chunk(chunk.clone())).await?;
            }
            Ok(())
        }
    }

    fn dispatch_barrier(&mut self, barrier: Barrier) -> Self::BarrierFuture<'_> {
        async move {
            for output in self.outputs.values_mut() {
                output.send(Message::Barrier(barrier.clone())).await?;
            }
            Ok(())
        }
    }

    fn add_outputs(&mut self, outputs: impl IntoIterator<Item = BoxedOutput>) {
        self.outputs.extend(Self::into_pairs(outputs));
    }

    fn remove_outputs(&mut self, actor_ids: &HashSet<ActorId>) {
        self.outputs
            .drain_filter(|actor_id, _| actor_ids.contains(actor_id))
            .count();
    }

    fn dispatcher_id(&self) -> DispatcherId {
        self.dispatcher_id
    }

    fn is_empty(&self) -> bool {
        self.outputs.is_empty()
    }
}

/// `SimpleDispatcher` dispatches message to a single output.
#[derive(Debug)]
pub struct SimpleDispatcher {
    /// In most cases, there is exactly one output. However, in some cases of configuration change,
    /// the field needs to be temporarily set to 0 or 2 outputs.
    ///
    /// - When dropping a materialized view, the output will be removed and this field becomes
    ///   empty. The [`DispatchExecutor`] will immediately clean-up this empty dispatcher before
    ///   finishing processing the current mutation.
    /// - When migrating a singleton fragment, the new output will be temporarily added in `pre`
    ///   stage and this field becomes multiple, which is for broadcasting this configuration
    ///   change barrier to both old and new downstream actors. In `post` stage, the old output
    ///   will be removed and this field becomes single again.
    ///
    /// Therefore, when dispatching data, we assert that there's exactly one output by
    /// `Self::output`.
    output: SmallVec<[BoxedOutput; 2]>,
    dispatcher_id: DispatcherId,
}

impl SimpleDispatcher {
    pub fn new(output: BoxedOutput, dispatcher_id: DispatcherId) -> Self {
        Self {
            output: smallvec![output],
            dispatcher_id,
        }
    }
}

impl Dispatcher for SimpleDispatcher {
    define_dispatcher_associated_types!();

    fn add_outputs(&mut self, outputs: impl IntoIterator<Item = BoxedOutput>) {
        self.output.extend(outputs);
        assert!(self.output.len() <= 2);
    }

    fn dispatch_barrier(&mut self, barrier: Barrier) -> Self::BarrierFuture<'_> {
        async move {
            // Only barrier is allowed to be dispatched to multiple outputs during migration.
            for output in self.output.iter_mut() {
                output.send(Message::Barrier(barrier.clone())).await?;
            }
            Ok(())
        }
    }

    fn dispatch_data(&mut self, chunk: StreamChunk) -> Self::DataFuture<'_> {
        async move {
            let output = self
                .output
                .iter_mut()
                .exactly_one()
                .expect("expect exactly one output");

            output.send(Message::Chunk(chunk)).await
        }
    }

    fn remove_outputs(&mut self, actor_ids: &HashSet<ActorId>) {
        self.output
            .retain(|output| !actor_ids.contains(&output.actor_id()));
    }

    fn dispatcher_id(&self) -> DispatcherId {
        self.dispatcher_id
    }

    fn is_empty(&self) -> bool {
        self.output.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use std::hash::{BuildHasher, Hasher};
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use futures::{pin_mut, StreamExt};
    use itertools::Itertools;
    use piestream_common::array::stream_chunk::StreamChunkTestExt;
    use piestream_common::array::{Array, ArrayBuilder, I32ArrayBuilder, Op};
    use piestream_common::catalog::Schema;
    use piestream_common::types::VIRTUAL_NODE_COUNT;
    use piestream_pb::stream_plan::DispatcherType;
    use static_assertions::const_assert_eq;
    use tokio::sync::mpsc::channel;

    use super::*;
    use crate::executor::exchange::output::Output;
    use crate::executor::receiver::ReceiverExecutor;
    use crate::task::test_utils::{add_local_channels, helper_make_local_actor};

    #[derive(Debug)]
    pub struct MockOutput {
        actor_id: ActorId,
        data: Arc<Mutex<Vec<Message>>>,
    }

    impl MockOutput {
        pub fn new(actor_id: ActorId, data: Arc<Mutex<Vec<Message>>>) -> Self {
            Self { actor_id, data }
        }
    }

    #[async_trait]
    impl Output for MockOutput {
        async fn send(&mut self, message: Message) -> StreamResult<()> {
            self.data.lock().unwrap().push(message);
            Ok(())
        }

        fn actor_id(&self) -> ActorId {
            self.actor_id
        }
    }

    // TODO: this test contains update being shuffled to different partitions, which is not
    // supported for now.
    #[tokio::test]
    #[ignore]
    async fn test_hash_dispatcher_complex() {
        test_hash_dispatcher_complex_inner().await
    }

    async fn test_hash_dispatcher_complex_inner() {
        // This test only works when VIRTUAL_NODE_COUNT is 256.
        const_assert_eq!(VIRTUAL_NODE_COUNT, 256);

        let num_outputs = 2; // actor id ranges from 1 to 2
        let key_indices = &[0, 2];
        let output_data_vecs = (0..num_outputs)
            .map(|_| Arc::new(Mutex::new(Vec::new())))
            .collect::<Vec<_>>();
        let outputs = output_data_vecs
            .iter()
            .enumerate()
            .map(|(actor_id, data)| {
                Box::new(MockOutput::new(1 + actor_id as u32, data.clone())) as BoxedOutput
            })
            .collect::<Vec<_>>();
        let mut hash_mapping = (1..num_outputs + 1)
            .flat_map(|id| vec![id as ActorId; VIRTUAL_NODE_COUNT / num_outputs])
            .collect_vec();
        hash_mapping.resize(VIRTUAL_NODE_COUNT, num_outputs as u32);
        let mut hash_dispatcher =
            HashDataDispatcher::new(outputs, key_indices.to_vec(), hash_mapping, 0);

        let chunk = StreamChunk::from_pretty(
            "  I I I
            +  4 6 8
            +  5 7 9
            +  0 0 0
            -  1 1 1 D
            U- 2 0 2
            U+ 2 0 2
            U- 3 3 2
            U+ 3 3 4",
        );
        hash_dispatcher.dispatch_data(chunk).await.unwrap();

        assert_eq!(
            *output_data_vecs[0].lock().unwrap()[0].as_chunk().unwrap(),
            StreamChunk::from_pretty(
                "  I I I
                +  4 6 8
                +  5 7 9
                +  0 0 0
                -  1 1 1 D
                U- 2 0 2
                U+ 2 0 2
                -  3 3 2 D  // Should rewrite UpdateDelete to Delete
                +  3 3 4    // Should rewrite UpdateInsert to Insert",
            )
        );
        assert_eq!(
            *output_data_vecs[1].lock().unwrap()[0].as_chunk().unwrap(),
            StreamChunk::from_pretty(
                "  I I I
                +  4 6 8 D
                +  5 7 9 D
                +  0 0 0 D
                -  1 1 1 D  // Should keep original invisible mark
                U- 2 0 2 D  // Should keep UpdateDelete
                U+ 2 0 2 D  // Should keep UpdateInsert
                -  3 3 2    // Should rewrite UpdateDelete to Delete
                +  3 3 4 D  // Should rewrite UpdateInsert to Insert",
            )
        );
    }

    #[tokio::test]
    async fn test_configuration_change() {
        let _schema = Schema { fields: vec![] };
        let (tx, rx) = channel(16);
        let actor_id = 233;
        let input = Box::new(ReceiverExecutor::for_test(rx));
        let ctx = Arc::new(SharedContext::for_test());
        let metrics = Arc::new(StreamingMetrics::unused());

        let (untouched, old, new) = (234, 235, 238); // broadcast downstream actors
        let (old_simple, new_simple) = (114, 514); // simple downstream actors

        // 1. Register info and channels in context.
        {
            let mut actor_infos = ctx.actor_infos.write();

            for local_actor_id in [actor_id, untouched, old, new, old_simple, new_simple] {
                actor_infos.insert(local_actor_id, helper_make_local_actor(local_actor_id));
            }
        }
        add_local_channels(
            ctx.clone(),
            vec![
                (actor_id, untouched),
                (actor_id, old),
                (actor_id, new),
                (actor_id, old_simple),
                (actor_id, new_simple),
            ],
        );

        let broadcast_dispatcher_id = 666;
        let broadcast_dispatcher = DispatcherImpl::new(
            &ctx,
            actor_id,
            &ProstDispatcher {
                r#type: DispatcherType::Broadcast as _,
                dispatcher_id: broadcast_dispatcher_id,
                downstream_actor_id: vec![untouched, old],
                ..Default::default()
            },
        )
        .unwrap();

        let simple_dispatcher_id = 888;
        let simple_dispatcher = DispatcherImpl::new(
            &ctx,
            actor_id,
            &ProstDispatcher {
                r#type: DispatcherType::Simple as _,
                dispatcher_id: simple_dispatcher_id,
                downstream_actor_id: vec![old_simple],
                ..Default::default()
            },
        )
        .unwrap();

        let executor = Box::new(DispatchExecutor::new(
            input,
            vec![broadcast_dispatcher, simple_dispatcher],
            actor_id,
            ctx.clone(),
            metrics,
        ))
        .execute();
        pin_mut!(executor);

        // 2. Take downstream receivers.
        let mut rxs = [untouched, old, new, old_simple, new_simple]
            .into_iter()
            .map(|id| (id, ctx.take_receiver(&(actor_id, id)).unwrap()))
            .collect::<HashMap<_, _>>();
        macro_rules! try_recv {
            ($down_id:expr) => {
                rxs.get_mut(&$down_id).unwrap().try_recv()
            };
        }

        // 3. Send a chunk.
        tx.send(Message::Chunk(StreamChunk::default()))
            .await
            .unwrap();

        // 4. Send a configuration change barrier for broadcast dispatcher.
        let dispatcher_updates = maplit::hashmap! {
            actor_id => vec![ProstDispatcherUpdate {
                actor_id,
                dispatcher_id: broadcast_dispatcher_id,
                added_downstream_actor_id: vec![new],
                removed_downstream_actor_id: vec![old],
                hash_mapping: Default::default(),
            }]
        };
        let b1 = Barrier::new_test_barrier(1).with_mutation(Mutation::Update {
            dispatchers: dispatcher_updates,
            merges: Default::default(),
            vnode_bitmaps: Default::default(),
            dropped_actors: Default::default(),
            actor_splits: Default::default(),
        });
        tx.send(Message::Barrier(b1)).await.unwrap();
        executor.next().await.unwrap().unwrap();

        // 5. Check downstream.
        try_recv!(untouched).unwrap().as_chunk().unwrap();
        try_recv!(untouched).unwrap().as_barrier().unwrap();

        try_recv!(old).unwrap().as_chunk().unwrap();
        try_recv!(old).unwrap().as_barrier().unwrap(); // It should still receive the barrier even if it's to be removed.

        try_recv!(new).unwrap().as_barrier().unwrap(); // Since it's just added, it won't receive the chunk.

        try_recv!(old_simple).unwrap().as_chunk().unwrap();
        try_recv!(old_simple).unwrap().as_barrier().unwrap(); // Untouched.

        // 6. Send another barrier.
        tx.send(Message::Barrier(Barrier::new_test_barrier(2)))
            .await
            .unwrap();
        executor.next().await.unwrap().unwrap();

        // 7. Check downstream.
        try_recv!(untouched).unwrap().as_barrier().unwrap();
        try_recv!(old).unwrap_err(); // Since it's stopped, we can't receive the new messages.
        try_recv!(new).unwrap().as_barrier().unwrap();

        try_recv!(old_simple).unwrap().as_barrier().unwrap(); // Untouched.
        try_recv!(new_simple).unwrap_err(); // Untouched.

        // 8. Send another chunk.
        tx.send(Message::Chunk(StreamChunk::default()))
            .await
            .unwrap();

        // 9. Send a configuration change barrier for simple dispatcher.
        let dispatcher_updates = maplit::hashmap! {
            actor_id => vec![ProstDispatcherUpdate {
                actor_id,
                dispatcher_id: simple_dispatcher_id,
                added_downstream_actor_id: vec![new_simple],
                removed_downstream_actor_id: vec![old_simple],
                hash_mapping: Default::default(),
            }]
        };
        let b3 = Barrier::new_test_barrier(3).with_mutation(Mutation::Update {
            dispatchers: dispatcher_updates,
            merges: Default::default(),
            vnode_bitmaps: Default::default(),
            dropped_actors: Default::default(),
            actor_splits: Default::default(),
        });
        tx.send(Message::Barrier(b3)).await.unwrap();
        executor.next().await.unwrap().unwrap();

        // 10. Check downstream.
        try_recv!(old_simple).unwrap().as_chunk().unwrap();
        try_recv!(old_simple).unwrap().as_barrier().unwrap(); // It should still receive the barrier even if it's to be removed.

        try_recv!(new_simple).unwrap().as_barrier().unwrap(); // Since it's just added, it won't receive the chunk.

        // 11. Send another barrier.
        tx.send(Message::Barrier(Barrier::new_test_barrier(4)))
            .await
            .unwrap();
        executor.next().await.unwrap().unwrap();

        // 12. Check downstream.
        try_recv!(old_simple).unwrap_err(); // Since it's stopped, we can't receive the new messages.
        try_recv!(new_simple).unwrap().as_barrier().unwrap();
    }

    #[tokio::test]
    async fn test_hash_dispatcher() {
        let num_outputs = 5; // actor id ranges from 1 to 5
        let cardinality = 10;
        let dimension = 4;
        let key_indices = &[0, 2];
        let output_data_vecs = (0..num_outputs)
            .map(|_| Arc::new(Mutex::new(Vec::new())))
            .collect::<Vec<_>>();
        let outputs = output_data_vecs
            .iter()
            .enumerate()
            .map(|(actor_id, data)| {
                Box::new(MockOutput::new(1 + actor_id as u32, data.clone())) as BoxedOutput
            })
            .collect::<Vec<_>>();
        let mut hash_mapping = (1..num_outputs + 1)
            .flat_map(|id| vec![id as ActorId; VIRTUAL_NODE_COUNT / num_outputs])
            .collect_vec();
        hash_mapping.resize(VIRTUAL_NODE_COUNT, num_outputs as u32);
        let mut hash_dispatcher =
            HashDataDispatcher::new(outputs, key_indices.to_vec(), hash_mapping.clone(), 0);

        let mut ops = Vec::new();
        for idx in 0..cardinality {
            if idx % 2 == 0 {
                ops.push(Op::Insert);
            } else {
                ops.push(Op::Delete);
            }
        }

        let mut start = 19260817i32..;
        let mut builders = (0..dimension)
            .map(|_| I32ArrayBuilder::new(cardinality))
            .collect_vec();
        let mut output_cols = vec![vec![vec![]; dimension]; num_outputs];
        let mut output_ops = vec![vec![]; num_outputs];
        for op in &ops {
            let hash_builder = Crc32FastBuilder {};
            let mut hasher = hash_builder.build_hasher();
            let one_row = (0..dimension).map(|_| start.next().unwrap()).collect_vec();
            for key_idx in key_indices.iter() {
                let val = one_row[*key_idx];
                let bytes = val.to_le_bytes();
                hasher.update(&bytes);
            }
            let output_idx =
                hash_mapping[hasher.finish() as usize % VIRTUAL_NODE_COUNT] as usize - 1;
            for (builder, val) in builders.iter_mut().zip_eq(one_row.iter()) {
                builder.append(Some(*val));
            }
            output_cols[output_idx]
                .iter_mut()
                .zip_eq(one_row.iter())
                .for_each(|(each_column, val)| each_column.push(*val));
            output_ops[output_idx].push(op);
        }

        let columns = builders
            .into_iter()
            .map(|builder| {
                let array = builder.finish();
                array.into()
            })
            .collect::<Vec<_>>();

        let chunk = StreamChunk::new(ops, columns, None);
        hash_dispatcher.dispatch_data(chunk).await.unwrap();

        for (output_idx, output) in output_data_vecs.into_iter().enumerate() {
            let guard = output.lock().unwrap();
            // It is possible that there is no chunks, as a key doesn't belong to any hash bucket.
            assert!(guard.len() <= 1);
            if guard.is_empty() {
                assert!(output_cols[output_idx].iter().all(|x| { x.is_empty() }));
            } else {
                let message = guard.get(0).unwrap();
                let real_chunk = match message {
                    Message::Chunk(chunk) => chunk,
                    _ => panic!(),
                };
                real_chunk
                    .columns()
                    .iter()
                    .zip_eq(output_cols[output_idx].iter())
                    .for_each(|(real_col, expect_col)| {
                        let real_vals = real_chunk
                            .visibility()
                            .as_ref()
                            .unwrap()
                            .iter()
                            .enumerate()
                            .filter(|(_, vis)| *vis)
                            .map(|(row_idx, _)| {
                                real_col.array_ref().as_int32().value_at(row_idx).unwrap()
                            })
                            .collect::<Vec<_>>();
                        assert_eq!(real_vals.len(), expect_col.len());
                        assert_eq!(real_vals, *expect_col);
                    });
            }
        }
    }
}
