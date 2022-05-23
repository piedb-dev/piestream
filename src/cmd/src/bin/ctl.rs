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

#![cfg_attr(coverage, feature(no_coverage))]

use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[cfg_attr(coverage, no_coverage)]
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    use clap::StructOpt;

    let opts = risingwave_ctl::CliOpts::parse();

    risingwave_logging::oneshot_common();
    risingwave_logging::init_risingwave_logger(false, true);

    risingwave_ctl::start(opts).await
}
