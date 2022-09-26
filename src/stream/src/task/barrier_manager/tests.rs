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

use std::iter::once;

use itertools::Itertools;
use tokio::sync::mpsc::unbounded_channel;

use super::*;

#[tokio::test]
async fn test_managed_barrier_collection() -> Result<()> {
    let mut manager = LocalBarrierManager::new();
    assert!(!manager.is_local_mode());
    println!("!manager.is_local_mode()={:?}", !manager.is_local_mode());

    //注册actor->sender,返回actor_id->receiver
    let register_sender = |actor_id: u32| {
        let (barrier_tx, barrier_rx) = unbounded_channel();
        manager.register_sender(actor_id, barrier_tx);
        (actor_id, barrier_rx)
    };

    // Register actors
    let actor_ids = vec![233, 234, 235];
    let count = actor_ids.len();
    let mut rxs = actor_ids
        .clone()
        .into_iter()
        .map(register_sender)
        .collect_vec();

    // Send a barrier to all actors
    let epoch = 114514;

    //构建barrier
    let barrier = Barrier::new_test_barrier(epoch);

    //发送barrier,输入两个参数一个是发送actor_id列表，一个是接收actor_id列表
    manager
        .send_barrier(&barrier, actor_ids.clone(), actor_ids)
        .unwrap();
    //拿到一个collect_rx
    let mut collect_rx = manager.remove_collect_rx(barrier.epoch.prev);

    // Collect barriers from actors,接收到send_barrier函数发送的barrier
    let collected_barriers = rxs
        .iter_mut()
        .map(|(actor_id, rx)| {
            let barrier = rx.try_recv().unwrap();
            assert_eq!(barrier.epoch.curr, epoch);
            (*actor_id, barrier)
        })
        .collect_vec();

    println!("collected_barriers={:?}", collected_barriers);
    // Report to local barrier manager
    for (i, (actor_id, barrier)) in collected_barriers.into_iter().enumerate() {
        manager.collect(actor_id, &barrier).unwrap();
        let notified = collect_rx.try_recv().is_ok();
        println!("notified={:?}", notified);
        assert_eq!(notified, i == count - 1);
    }

    Ok(())
}

#[tokio::test]
async fn test_managed_barrier_collection_before_send_request() -> Result<()> {
    let mut manager = LocalBarrierManager::new();
    assert!(!manager.is_local_mode());

    let register_sender = |actor_id: u32| {
        let (barrier_tx, barrier_rx) = unbounded_channel();
        manager.register_sender(actor_id, barrier_tx);
        (actor_id, barrier_rx)
    };

    let actor_ids_to_send = vec![233, 234, 235];
    let extra_actor_id = 666;
    let actor_ids_to_collect = actor_ids_to_send
        .iter()
        .cloned()
        .chain(once(extra_actor_id))
        .collect_vec();

    println!("actor_ids_to_collect={:?} actor_ids_to_send={:?}", actor_ids_to_collect, actor_ids_to_send);
    // Register actors
    let count = actor_ids_to_send.len();
    //注册三个actor_id
    let mut rxs = actor_ids_to_send
        .clone()
        .into_iter()
        .map(register_sender)
        .collect_vec();

    // Prepare the barrier
    let epoch = 114514;
    let barrier = Barrier::new_test_barrier(epoch);

    // Collect a barrer before sending 
    // 发送前就已经接收到barrier,send_barrier函数处理会忽略该actor_id
    manager.collect(extra_actor_id, &barrier).unwrap();

    // Send the barrier to all actors
    manager
        .send_barrier(&barrier, actor_ids_to_send, actor_ids_to_collect)
        .unwrap();
    let mut collect_rx = manager.remove_collect_rx(barrier.epoch.prev);

    // Collect barriers from actors
    let collected_barriers = rxs
        .iter_mut()
        .map(|(actor_id, rx)| {
            let barrier = rx.try_recv().unwrap();
            assert_eq!(barrier.epoch.curr, epoch);
            (*actor_id, barrier)
        })
        .collect_vec();

   
    // Report to local barrier manager
    for (i, (actor_id, barrier)) in collected_barriers.into_iter().enumerate() {
        println!("****************************{:?}", i);
        manager.collect(actor_id, &barrier).unwrap();
        let notified = collect_rx.try_recv().is_ok();
        assert_eq!(notified, i == count - 1);
    }

    Ok(())
}
