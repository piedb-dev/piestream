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

use async_trait::async_trait;

use crate::base::SplitEnumerator;
use crate::datagen::{DatagenProperties, DatagenSplit};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DatagenSplitEnumerator {}

#[async_trait]
impl SplitEnumerator for DatagenSplitEnumerator {
    type Properties = DatagenProperties;
    type Split = DatagenSplit;

    async fn new(_properties: DatagenProperties) -> anyhow::Result<DatagenSplitEnumerator> {
        Ok(DatagenSplitEnumerator {})
    }

    async fn list_splits(&mut self) -> anyhow::Result<Vec<DatagenSplit>> {
        let split = vec![DatagenSplit::default()];
        Ok(split)
    }
}
