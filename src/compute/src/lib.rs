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

#![feature(trait_alias)]
#![feature(binary_heap_drain_sorted)]
#![feature(generic_associated_types)]
#![feature(let_else)]
#![feature(generators)]
#![feature(type_alias_impl_trait)]
#![cfg_attr(coverage, feature(no_coverage))]

#[macro_use]
extern crate tracing;

pub mod rpc;
pub mod server;

use clap::Parser;
use serde::{Deserialize, Serialize};

/// Command-line arguments for compute-node.
#[derive(Parser, Debug)]
pub struct ComputeNodeOpts {
    // TODO: rename to listen_address and separate out the port.
    #[clap(long, default_value = "127.0.0.1:5688")]
    pub host: String,

    // Optional, we will use listen_address if not specified.
    #[clap(long)]
    pub client_address: Option<String>,

    #[clap(long, default_value = "hummock+memory")]
    pub state_store: String,

    #[clap(long, default_value = "127.0.0.1:1222")]
    pub prometheus_listener_addr: String,

    /// Used for control the metrics level, similar to log level.
    /// 0 = close metrics
    /// >0 = open metrics
    #[clap(long, default_value = "0")]
    pub metrics_level: u32,

    #[clap(long, default_value = "http://127.0.0.1:5690")]
    pub meta_address: String,

    /// No given `config_path` means to use default config.
    #[clap(long, default_value = "")]
    pub config_path: String,

    /// Enable reporting tracing information to jaeger.
    #[clap(long)]
    pub enable_jaeger_tracing: bool,

    /// Enable async stack tracing for risectl.
    #[clap(long)]
    pub enable_async_stack_trace: bool,

    /// Path to file cache data directory.
    /// Left empty to disable file cache.
    #[clap(long, default_value = "")]
    pub file_cache_dir: String,

    /// Enable managed lru cache, or use local lru cache.
    #[clap(long)]
    pub enable_managed_cache: bool,
}

use std::future::Future;
use std::pin::Pin;

use piestream_common::config::{BatchConfig, ServerConfig, StorageConfig, StreamingConfig};

use crate::server::compute_node_serve;

/// Start compute node
pub fn start(opts: ComputeNodeOpts) -> Pin<Box<dyn Future<Output = ()> + Send>> {
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

        let (join_handle_vec, _shutdown_send) =
            compute_node_serve(listen_address, client_address, opts).await;

        for join_handle in join_handle_vec {
            join_handle.await.unwrap();
        }
    })
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ComputeNodeConfig {
    // For connection
    #[serde(default)]
    pub server: ServerConfig,

    // Below for batch query.
    #[serde(default)]
    pub batch: BatchConfig,

    // Below for streaming.
    #[serde(default)]
    pub streaming: StreamingConfig,

    // Below for Hummock.
    #[serde(default)]
    pub storage: StorageConfig,
}
