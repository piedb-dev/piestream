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

use prost::Message;
use risingwave_hummock_sdk::HummockEpoch;
use risingwave_pb::hummock::{HummockContextRefId, HummockPinnedSnapshot};

use crate::model::MetadataModel;

/// Column family name for hummock pinned snapshot
/// `cf(hummock_pinned_snapshot)`: `HummockContextRefId` -> `HummockPinnedSnapshot`
const HUMMOCK_PINNED_SNAPSHOT_CF_NAME: &str = "cf/hummock_pinned_snapshot";

/// `HummockPinnedSnapshot` tracks pinned snapshots by given context id.
impl MetadataModel for HummockPinnedSnapshot {
    type KeyType = HummockContextRefId;
    type ProstType = HummockPinnedSnapshot;

    fn cf_name() -> String {
        String::from(HUMMOCK_PINNED_SNAPSHOT_CF_NAME)
    }

    fn to_protobuf(&self) -> Self::ProstType {
        self.clone()
    }

    fn to_protobuf_encoded_vec(&self) -> Vec<u8> {
        self.encode_to_vec()
    }

    fn from_protobuf(prost: Self::ProstType) -> Self {
        prost
    }

    fn key(&self) -> risingwave_common::error::Result<Self::KeyType> {
        Ok(HummockContextRefId {
            id: self.context_id,
        })
    }
}

pub trait HummockPinnedSnapshotExt {
    fn pin_snapshot(&mut self, new_snapshot_id: HummockEpoch);

    fn unpin_snapshot(&mut self, pinned_snapshot_id: HummockEpoch);
}

impl HummockPinnedSnapshotExt for HummockPinnedSnapshot {
    fn pin_snapshot(&mut self, epoch: HummockEpoch) {
        let found = self.snapshot_id.iter().position(|&v| v == epoch);
        if found.is_none() {
            self.snapshot_id.push(epoch);
        }
    }

    fn unpin_snapshot(&mut self, epoch: HummockEpoch) {
        let found = self.snapshot_id.iter().position(|&v| v == epoch);
        if let Some(pos) = found {
            self.snapshot_id.remove(pos);
        }
    }
}
