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

use piestream_pb::user::UserInfo;

use crate::model::MetadataModel;

/// Column family name for user info.
const USER_INFO_CF_NAME: &str = "cf/user_info";

/// `UserInfo` stores the user information.
impl MetadataModel for UserInfo {
    type KeyType = String;
    type ProstType = UserInfo;

    fn cf_name() -> String {
        USER_INFO_CF_NAME.to_string()
    }

    fn to_protobuf(&self) -> Self::ProstType {
        self.clone()
    }

    fn from_protobuf(prost: Self::ProstType) -> Self {
        prost
    }

    fn key(&self) -> piestream_common::error::Result<Self::KeyType> {
        Ok(self.name.clone())
    }
}
