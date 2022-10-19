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

use bytes::Bytes;
use piestream_hummock_sdk::HummockReadEpoch;
use piestream_meta::hummock::test_utils::setup_compute_env;
use piestream_meta::hummock::MockHummockMetaClient;
use piestream_rpc_client::HummockMetaClient;
use piestream_storage::hummock::iterator::test_utils::mock_sstable_store;
use piestream_storage::hummock::test_utils::{count_iter, default_config_for_test};
use piestream_storage::hummock::HummockStorage;
use piestream_storage::storage_value::StorageValue;
use piestream_storage::store::{ReadOptions, WriteOptions};
use piestream_storage::StateStore;

use crate::test_utils::get_test_notification_client;

#[tokio::test]
#[ignore]
#[cfg(all(test, feature = "failpoints"))]
async fn test_failpoints_state_store_read_upload() {
    let mem_upload_err = "mem_upload_err";
    let mem_read_err = "mem_read_err";
    let sstable_store = mock_sstable_store();
    let hummock_options = Arc::new(default_config_for_test());
    let (env, hummock_manager_ref, _cluster_manager_ref, worker_node) =
        setup_compute_env(8080).await;
    let meta_client = Arc::new(MockHummockMetaClient::new(
        hummock_manager_ref.clone(),
        worker_node.id,
    ));

    let hummock_storage = HummockStorage::for_test(
        hummock_options.clone(),
        sstable_store.clone(),
        meta_client.clone(),
        get_test_notification_client(env, hummock_manager_ref, worker_node),
    )
    .await
    .unwrap();
    let local_version_manager = hummock_storage.local_version_manager();

    let anchor = Bytes::from("aa");
    let mut batch1 = vec![
        (anchor.clone(), StorageValue::new_put("111")),
        (Bytes::from("cc"), StorageValue::new_put("222")),
    ];
    batch1.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

    let mut batch2 = vec![
        (Bytes::from("cc"), StorageValue::new_put("333")),
        (anchor.clone(), StorageValue::new_delete()),
    ];
    // Make sure the batch is sorted.
    batch2.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
    hummock_storage
        .ingest_batch(
            batch1,
            WriteOptions {
                epoch: 1,
                table_id: Default::default(),
            },
        )
        .await
        .unwrap();

    // Get the value after flushing to remote.
    let value = hummock_storage
        .get(
            &anchor,
            true,
            ReadOptions {
                epoch: 1,
                table_id: Default::default(),
                retention_seconds: None,
            },
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(value, Bytes::from("111"));
    // // Write second batch.
    hummock_storage
        .ingest_batch(
            batch2,
            WriteOptions {
                epoch: 3,
                table_id: Default::default(),
            },
        )
        .await
        .unwrap();

    // sync epoch1 test the read_error
    let ssts = hummock_storage
        .seal_and_sync_epoch(1)
        .await
        .unwrap()
        .uncommitted_ssts;
    meta_client.commit_epoch(1, ssts).await.unwrap();
    local_version_manager
        .try_wait_epoch(HummockReadEpoch::Committed(1))
        .await
        .unwrap();
    // clear block cache
    sstable_store.clear_block_cache();
    sstable_store.clear_meta_cache();
    fail::cfg(mem_read_err, "return").unwrap();

    let result = hummock_storage
        .get(
            &anchor,
            true,
            ReadOptions {
                epoch: 2,
                table_id: Default::default(),
                retention_seconds: None,
            },
        )
        .await;
    assert!(result.is_err());
    let result = hummock_storage
        .iter(
            None,
            ..=b"ee".to_vec(),
            ReadOptions {
                epoch: 2,
                table_id: Default::default(),
                retention_seconds: None,
            },
        )
        .await;
    assert!(result.is_err());

    let value = hummock_storage
        .get(
            b"ee".as_ref(),
            true,
            ReadOptions {
                epoch: 2,
                table_id: Default::default(),
                retention_seconds: None,
            },
        )
        .await
        .unwrap();
    assert!(value.is_none());
    fail::remove(mem_read_err);
    // test the upload_error
    fail::cfg(mem_upload_err, "return").unwrap();

    let result = hummock_storage.seal_and_sync_epoch(3).await;
    assert!(result.is_err());
    fail::remove(mem_upload_err);

    let ssts = hummock_storage
        .seal_and_sync_epoch(3)
        .await
        .unwrap()
        .uncommitted_ssts;
    meta_client.commit_epoch(3, ssts).await.unwrap();
    local_version_manager
        .try_wait_epoch(HummockReadEpoch::Committed(3))
        .await
        .unwrap();

    let value = hummock_storage
        .get(
            &anchor,
            true,
            ReadOptions {
                epoch: 5,
                table_id: Default::default(),
                retention_seconds: None,
            },
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(value, Bytes::from("111"));
    let mut iters = hummock_storage
        .iter(
            None,
            ..=b"ee".to_vec(),
            ReadOptions {
                epoch: 5,
                table_id: Default::default(),
                retention_seconds: None,
            },
        )
        .await
        .unwrap();
    let len = count_iter(&mut iters).await;
    assert_eq!(len, 2);
}
