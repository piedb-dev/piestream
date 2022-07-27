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
#![feature(map_try_insert)]
#![feature(let_chains)]
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

#[macro_use]
pub mod catalog;
pub mod binder;
pub mod expr;
pub mod handler;
pub mod observer;
pub mod optimizer;
pub mod planner;
#[expect(dead_code)]
pub mod scheduler;
pub mod session;
pub mod stream_fragmenter;
pub mod utils;
extern crate log;
mod meta_client;
pub mod test_utils;
extern crate core;
extern crate piestream_common;

pub mod user;

use std::ffi::OsString;
use std::iter;
use std::sync::Arc;

use clap::Parser;
use pgwire::pg_server::pg_serve;
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
}

impl Default for FrontendOpts {
    fn default() -> Self {
        FrontendOpts::parse_from(iter::empty::<OsString>())
    }
}

use std::future::Future;
use std::pin::Pin;

/// Start frontend
pub fn start(opts: FrontendOpts) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    // WARNING: don't change the function signature. Making it `async fn` will cause
    // slow compile in release mode.
    Box::pin(async move {
        //初始化记录环境变量等基础参数的session
        let session_mgr = Arc::new(SessionManagerImpl::new(&opts).await.unwrap());
        //启动pgwire的pg_serve服务
        //use pgwire::pg_server::pg_serve;
        pg_serve(&opts.host, session_mgr).await.unwrap();
    })
}
