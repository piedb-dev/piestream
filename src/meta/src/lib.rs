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

#![allow(clippy::derive_partial_eq_without_eq)]
#![warn(clippy::dbg_macro)]
#![warn(clippy::disallowed_methods)]
#![warn(clippy::doc_markdown)]
#![warn(clippy::explicit_into_iter_loop)]
#![warn(clippy::explicit_iter_loop)]
#![warn(clippy::inconsistent_struct_constructor)]
#![warn(clippy::unused_async)]
#![warn(clippy::map_flatten)]
#![warn(clippy::no_effect_underscore_binding)]
#![warn(clippy::await_holding_lock)]
#![deny(unused_must_use)]
#![deny(rustdoc::broken_intra_doc_links)]
#![feature(trait_alias)]
#![feature(generic_associated_types)]
#![feature(binary_heap_drain_sorted)]
#![feature(option_result_contains)]
#![feature(let_chains)]
#![feature(let_else)]
#![feature(type_alias_impl_trait)]
#![feature(map_first_last)]
#![feature(drain_filter)]
#![feature(custom_test_frameworks)]
#![feature(lint_reasons)]
#![cfg_attr(coverage, feature(no_coverage))]
#![test_runner(piestream_test_runner::test_runner::run_failpont_tests)]

extern crate core;

mod barrier;
pub mod cluster;
mod dashboard;
pub mod hummock;
pub mod manager;
mod model;
pub mod rpc;
pub mod storage;
mod stream;
pub mod test_utils;

use std::time::Duration;

use clap::{ArgEnum, Parser};
use piestream_common::config::ComputeNodeConfig;

use crate::manager::MetaOpts;
use crate::rpc::server::{rpc_serve, AddressInfo, MetaStoreBackend};

#[derive(Copy, Clone, Debug, ArgEnum)]
enum Backend {
    Mem,
    Etcd,
}

#[derive(Debug, Parser)]
pub struct MetaNodeOpts {
    // TODO: rename to listen_address and separate out the port.
    #[clap(long, default_value = "127.0.0.1:5690")]
    listen_addr: String,

    #[clap(long)]
    host: Option<String>,

    #[clap(long)]
    dashboard_host: Option<String>,

    #[clap(long)]
    prometheus_host: Option<String>,

    #[clap(long, arg_enum, default_value_t = Backend::Mem)]
    backend: Backend,

    #[clap(long, default_value_t = String::from(""))]
    etcd_endpoints: String,

    /// Maximum allowed heartbeat interval in ms.
    #[clap(long, default_value = "60000")]
    max_heartbeat_interval: u32,

    #[clap(long)]
    dashboard_ui_path: Option<String>,

    /// No given `config_path` means to use default config.
    #[clap(long, default_value = "")]
    pub config_path: String,

    /// Whether to enable fail-on-recovery. If not set, default to enable. Should only be used in
    /// e2e tests.
    #[clap(long)]
    disable_recovery: bool,

    #[clap(long, default_value = "10")]
    meta_leader_lease_secs: u64,

    /// After specified seconds of idle (no mview or flush), the process will be exited.
    /// It is mainly useful for playgrounds.
    #[clap(long)]
    dangerous_max_idle_secs: Option<u64>,
}

fn load_config(opts: &MetaNodeOpts) -> ComputeNodeConfig {
    piestream_common::config::load_config(&opts.config_path)
}

use std::future::Future;
use std::pin::Pin;

/// Start meta node

pub fn start(opts: MetaNodeOpts) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    // WARNING: don't change the function signature. Making it `async fn` will cause
    // slow compile in release mode.
    Box::pin(async move {
        let compute_config = load_config(&opts);
        let meta_addr = opts.host.unwrap_or_else(|| opts.listen_addr.clone());
        let listen_addr = opts.listen_addr.parse().unwrap();
        let dashboard_addr = opts.dashboard_host.map(|x| x.parse().unwrap());
        let prometheus_addr = opts.prometheus_host.map(|x| x.parse().unwrap());
        let backend = match opts.backend {
            Backend::Etcd => MetaStoreBackend::Etcd {
                endpoints: opts
                    .etcd_endpoints
                    .split(',')
                    .map(|x| x.to_string())
                    .collect(),
            },
            Backend::Mem => MetaStoreBackend::Mem,
        };
        let max_heartbeat_interval = Duration::from_millis(opts.max_heartbeat_interval as u64);
        let checkpoint_interval =
            Duration::from_millis(compute_config.streaming.checkpoint_interval_ms as u64);
        let max_idle_ms = opts.dangerous_max_idle_secs.unwrap_or(0) * 1000;
        let in_flight_barrier_nums = compute_config.streaming.in_flight_barrier_nums as usize;

        tracing::info!("Meta server listening at {}", listen_addr);
        let add_info = AddressInfo {
            addr: meta_addr,
            listen_addr,
            prometheus_addr,
            dashboard_addr,
            ui_path: opts.dashboard_ui_path,
        };
        let (join_handle, _shutdown_send) = rpc_serve(
            add_info,
            backend,
            max_heartbeat_interval,
            opts.meta_leader_lease_secs,
            MetaOpts {
                enable_recovery: !opts.disable_recovery,
                checkpoint_interval,
                max_idle_ms,
                in_flight_barrier_nums,
            },
        )
        .await
        .unwrap();
        join_handle.await.unwrap();
        tracing::info!("Meta server is stopped");
    })
}
