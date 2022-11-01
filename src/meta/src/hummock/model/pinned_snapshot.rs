// Copyright 2022 Piedb Data
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
use piestream_hummock_sdk::HummockContextId;
use piestream_pb::hummock::HummockPinnedSnapshot;

use crate::model::{MetadataModel, MetadataModelResult};

/// Column family name for hummock pinned snapshot
/// `cf(hummock_pinned_snapshot)`: `HummockContextId` -> `HummockPinnedSnapshot`
const HUMMOCK_PINNED_SNAPSHOT_CF_NAME: &str = "cf/hummock_pinned_snapshot";

/// `HummockPinnedSnapshot` tracks pinned snapshots by given context id.
impl MetadataModel for HummockPinnedSnapshot {
    type KeyType = HummockContextId;
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

    fn key(&self) -> MetadataModelResult<Self::KeyType> {
        Ok(self.context_id)
    }
}
