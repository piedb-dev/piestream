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

#![cfg_attr(coverage, feature(no_coverage))]

use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
//前端节点启动入口
#[cfg_attr(coverage, no_coverage)]
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    use clap::StructOpt;
    //设置数据库启动环境，
    //piestream_frontend = { path = "../frontend" }
    //piestream_rt = { path = "../utils/runtime" }
    //在每个toml文件中，依赖的定义，可以基于本地项目的绝对路径或者相对路径
    let opts = piestream_frontend::FrontendOpts::parse();

    piestream_rt::oneshot_common();
    //初始化日志
    piestream_rt::init_piestream_logger(piestream_rt::LoggerSettings::new_default());
    //程序启动 
    piestream_frontend::start(opts).await
}
