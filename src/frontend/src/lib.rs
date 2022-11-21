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

#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(rustdoc::private_intra_doc_links)]
#![feature(map_try_insert)]
#![feature(negative_impls)]
#![feature(generators)]
#![feature(proc_macro_hygiene, stmt_expr_attributes)]
#![feature(let_else)]
#![feature(trait_alias)]
#![feature(drain_filter)]
#![feature(if_let_guard)]
#![feature(assert_matches)]
#![feature(map_first_last)]
#![feature(lint_reasons)]
#![feature(box_patterns)]
#![feature(once_cell)]
#![recursion_limit = "256"]

#[macro_use]
mod catalog;
pub use catalog::TableCatalog;
mod binder;
pub use binder::{bind_data_type, Binder};
pub mod expr;
pub mod handler;
pub use handler::PgResponseStream;
mod observer;
mod optimizer;
pub use optimizer::PlanRef;
mod planner;
pub use planner::Planner;
#[expect(dead_code)]
mod scheduler;
pub mod session;
mod stream_fragmenter;
pub use stream_fragmenter::build_graph;
mod utils;
pub use utils::{explain_stream_graph, WithOptions};
pub mod mysql_session;
mod meta_client;
pub mod test_utils;
mod user;

mod monitor;

use std::ffi::OsString;
use std::iter;
use std::sync::Arc;
use std::thread;
use core::time::Duration;

use clap::Parser;
use pgwire::pg_server::pg_serve;
use serde::{Deserialize, Serialize};
use mysql_session::mysql_server;
use session::SessionManagerImpl;

#[derive(Parser, Clone, Debug)]
pub struct FrontendOpts {
    // TODO: rename to listen_address and separate out the port.
    #[clap(long, default_value = "127.0.0.1:4566")]
    pub host: String,

    // Optional, we will use listen_address if not specified.
    #[clap(long)]
    pub client_address: Option<String>,

    // TODO: This is currently unused.
    #[clap(long)]
    pub port: Option<u16>,

    #[clap(long, default_value = "http://127.0.0.1:5690")]
    pub meta_addr: String,

    /// No given `config_path` means to use default config.
    #[clap(long, default_value = "")]
    pub config_path: String,

    #[clap(long, default_value = "127.0.0.1:2222")]
    pub prometheus_listener_addr: String,

    /// Used for control the metrics level, similar to log level.
    /// 0 = close metrics
    /// >0 = open metrics
    #[clap(long, default_value = "0")]
    pub metrics_level: u32,
}

impl Default for FrontendOpts {
    fn default() -> Self {
        FrontendOpts::parse_from(iter::empty::<OsString>())
    }
}

use std::future::Future;
use std::pin::Pin;
use tokio::task;
use piestream_common::error::internal_error;
use piestream_common::config::ServerConfig;

/// Start frontend
pub fn start(opts: FrontendOpts) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    // WARNING: don't change the function signature. Making it `async fn` will cause
    // slow compile in release mode.
    Box::pin(async move {
        let session_mgr = Arc::new(SessionManagerImpl::new(&opts).await.unwrap());
        let a1=session_mgr.clone();
        let a2=session_mgr.clone();
        let addr1=opts.host.clone();
        let addr2 = "127.0.0.1:4567".to_string();


        let pg_server_join=task::spawn(pg_serve(addr1, a1));
        //tokio::spawn(move ||futures::executor::block_on(pg_serve(addr1, a1)).unwrap());
        //thread::spawn(move ||futures::executor::block_on(pg_serve(addr1, a1)).unwrap());
        //let handle=thread::spawn(async move {pg_serve(addr1, a1).await.unwrap();});
        //thread::spawn(|| async move {pg_serve(addr1, a1).await.unwrap()});
        tokio::time::sleep(Duration::from_secs(10)).await;
        //pg_serve(addr1, a1).await.unwrap();
        println!("addr ====== ,&pg_serve)");
        let mysql_server_join=task::spawn( mysql_server(addr2, a2));
        //mysql_server(&addr2, a2).await;

       // pg_server_join.await.ok_or_else(|| internal_error("Could not compute vnode for lookup join"));
        mysql_server_join.await.unwrap();
    })
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct FrontendConfig {
    // For connection
    #[serde(default)]
    pub server: ServerConfig,
}

pub fn mysql_start(_opts: FrontendOpts) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    Box::pin(async move {
        let addr = "127.0.0.1:4567".to_string();
        println!("addr ====== {:?}",&addr);
        let session_mgr = Arc::new(SessionManagerImpl::new(&_opts).await.unwrap());
        mysql_server(addr,session_mgr).await;
    })
}
