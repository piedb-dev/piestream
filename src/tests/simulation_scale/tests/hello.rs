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

#![cfg(madsim)]

use anyhow::Result;
use piestream_simulation_scale::cluster::Cluster;
use piestream_simulation_scale::utils::AssertResult;

#[madsim::test]
async fn test_hello() -> Result<()> {
    let mut cluster = Cluster::start().await?;
    cluster
        .run("select concat_ws(', ', 'hello', 'world');")
        .await?
        .assert_result_eq("hello, world");

    Ok(())
}
