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

#![feature(let_else)]

mod compactor_observer;
mod rpc;
mod server;

use clap::Parser;
use piestream_common::config::{ServerConfig, StorageConfig};
use serde::{Deserialize, Serialize};

use crate::server::compactor_serve;

/// Command-line arguments for compute-node.
#[derive(Parser, Debug)]
pub struct CompactorOpts {
    // TODO: rename to listen_address and separate out the port.
    #[clap(long, default_value = "127.0.0.1:5509")]
    pub host: String,

    // Optional, we will use listen_address if not specified.
    #[clap(long)]
    pub client_address: Option<String>,

    // TODO: This is currently unused.
    #[clap(long)]
    pub port: Option<u16>,

    #[clap(long, default_value = "hummock+minio://hummockadmin:hummockadmin@127.0.0.1:9308/hummock001")]
    pub state_store: String,

    #[clap(long, default_value = "127.0.0.1:1260")]
    pub prometheus_listener_addr: String,

    #[clap(long, default_value = "0")]
    pub metrics_level: u32,

    #[clap(long, default_value = "http://127.0.0.1:5507")]
    pub meta_address: String,

    /// No given `config_path` means to use default config.
    #[clap(long, default_value = "")]
    pub config_path: String,

    /// It's a hint used by meta node.
    #[clap(long, default_value = "16")]
    pub max_concurrent_task_number: u64,

    #[clap(long)]
    pub compaction_worker_threads_number: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CompactorConfig {
    // For connection
    #[serde(default)]
    pub server: ServerConfig,

    // Below for Hummock.
    #[serde(default)]
    pub storage: StorageConfig,
}

use std::future::Future;
use std::pin::Pin;

pub fn start(opts: CompactorOpts) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    // WARNING: don't change the function signature. Making it `async fn` will cause
    // slow compile in release mode.
    Box::pin(async move {
        tracing::info!("meta address: {}", opts.meta_address.clone());

        let listen_address = opts.host.parse().unwrap();
        tracing::info!("Server Listening at {}", listen_address);

        let client_address = opts
            .client_address
            .as_ref()
            .unwrap_or_else(|| {
                tracing::warn!("Client address is not specified, defaulting to host address");
                &opts.host
            })
            .parse()
            .unwrap();
        tracing::info!("Client address is {}", client_address);

        let (join_handle, observer_join_handle, _shutdown_sender) =
            compactor_serve(listen_address, client_address, opts).await;

        join_handle.await.unwrap();
        observer_join_handle.await.unwrap();
    })
}
