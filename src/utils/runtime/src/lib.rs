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

//! Configures the piestream binary, including logging, locks, panic handler, etc.

#![feature(let_chains)]

mod trace_runtime;

use std::time::Duration;

use tracing::Level;
use tracing_subscriber::filter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;

/// Configure log targets for all `piestream` crates. When new crates are added and TRACE level
/// logs are needed, add them here.
fn configure_piestream_targets_jaeger(targets: filter::Targets) -> filter::Targets {
    targets
        // enable trace for most modules
        .with_target("piestream_stream", Level::TRACE)
        .with_target("piestream_batch", Level::TRACE)
        .with_target("piestream_storage", Level::TRACE)
        .with_target("piestream_sqlparser", Level::INFO)
        // disable events that are too verbose
        // if you want to enable any of them, find the target name and set it to `TRACE`
        // .with_target("events::stream::mview::scan", Level::TRACE)
        .with_target("events", Level::ERROR)
}

/// Configure log targets for all `piestream` crates. When new crates are added and TRACE level
/// logs are needed, add them here.
fn configure_piestream_targets_fmt(targets: filter::Targets) -> filter::Targets {
    targets
        // enable trace for most modules
        .with_target("piestream_stream", Level::DEBUG)
        .with_target("piestream_batch", Level::DEBUG)
        .with_target("piestream_storage", Level::DEBUG)
        .with_target("piestream_sqlparser", Level::INFO)
        .with_target("piestream_source", Level::INFO)
        .with_target("piestream_connector", Level::INFO)
        .with_target("piestream_frontend", Level::INFO)
        .with_target("piestream_meta", Level::INFO)
        .with_target("pgwire", Level::ERROR)
        // disable events that are too verbose
        // if you want to enable any of them, find the target name and set it to `TRACE`
        // .with_target("events::stream::mview::scan", Level::TRACE)
        .with_target("events", Level::ERROR)

    // if env_var_is_true("RW_CI") {
    //     targets.with_target("events::meta::server_heartbeat", Level::TRACE)
    // } else {
    //     targets
    // }
}

pub struct LoggerSettings {
    /// Enable Jaeger tracing.
    enable_jaeger_tracing: bool,
    /// Enable tokio console output.
    enable_tokio_console: bool,
    /// Enable colorful output in console.
    colorful: bool,
}

impl LoggerSettings {
    pub fn new_default() -> Self {
        Self::new(false, false)
    }

    pub fn new(enable_jaeger_tracing: bool, enable_tokio_console: bool) -> Self {
        Self {
            enable_jaeger_tracing,
            enable_tokio_console,
            colorful: console::colors_enabled_stderr(),
        }
    }
}

/// Set panic hook to abort the process (without losing debug info and stack trace).
pub fn set_panic_abort() {
    use std::panic;

    let default_hook = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
        default_hook(info);
        std::process::abort();
    }));
}

/// Init logger for piestream binaries.
pub fn init_piestream_logger(settings: LoggerSettings) {
    use isahc::config::Configurable;

    let fmt_layer = {
        // Configure log output to stdout
        let fmt_layer = tracing_subscriber::fmt::layer()
            .compact()
            .with_ansi(settings.colorful);

        let filter = filter::Targets::new()
            // Only enable WARN and ERROR for 3rd-party crates
            .with_target("aws_endpoint", Level::WARN)
            .with_target("hyper", Level::WARN)
            .with_target("h2", Level::WARN)
            .with_target("tower", Level::WARN)
            .with_target("isahc", Level::WARN)
            .with_target("console_subscriber", Level::WARN);

        // Configure piestream's own crates to log at TRACE level, uncomment the following line if
        // needed.

        let filter = configure_piestream_targets_fmt(filter);

        // Enable DEBUG level for all other crates
        // TODO: remove this in release mode
        let filter = filter.with_default(Level::DEBUG);

        fmt_layer.with_filter(filter)
    };

    let opentelemetry_layer = if settings.enable_jaeger_tracing {
        // With Jaeger tracing enabled, we should configure opentelemetry endpoints.

        opentelemetry::global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());

        let tracer = opentelemetry_jaeger::new_pipeline()
            // TODO: use UDP tracing in production environment
            .with_collector_endpoint("http://127.0.0.1:14268/api/traces")
            // TODO: change service name to compute-{port}
            .with_service_name("compute")
            // disable proxy
            .with_http_client(isahc::HttpClient::builder().proxy(None).build().unwrap())
            .install_batch(trace_runtime::RwTokio)
            .unwrap();

        let opentelemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        // Configure piestream's own crates to log at TRACE level, and ignore all third-party
        // crates
        let filter = filter::Targets::new();
        let filter = configure_piestream_targets_jaeger(filter);

        Some(opentelemetry_layer.with_filter(filter))
    } else {
        None
    };

    let tokio_console_layer = if settings.enable_tokio_console {
        let (console_layer, server) = console_subscriber::ConsoleLayer::builder()
            .with_default_env()
            .build();
        let console_layer = console_layer.with_filter(
            filter::Targets::new()
                .with_target("tokio", Level::TRACE)
                .with_target("runtime", Level::TRACE),
        );
        Some((console_layer, server))
    } else {
        None
    };

    match (opentelemetry_layer, tokio_console_layer) {
        (Some(_), Some(_)) => {
            // tracing_subscriber::registry()
            //     .with(fmt_layer)
            //     .with(opentelemetry_layer)
            //     .with(tokio_console_layer)
            //     .init();
            // Strange compiler bug is preventing us from enabling both of them...
            panic!("cannot enable opentelemetry layer and tokio console layer at the same time");
        }
        (Some(opentelemetry_layer), None) => {
            tracing_subscriber::registry()
                .with(fmt_layer)
                .with(opentelemetry_layer)
                .init();
        }
        (None, Some((tokio_console_layer, server))) => {
            tracing_subscriber::registry()
                .with(fmt_layer)
                .with(tokio_console_layer)
                .init();
            tokio::spawn(async move {
                tracing::info!("serving console subscriber");
                server.serve().await.unwrap();
            });
        }
        (None, None) => {
            tracing_subscriber::registry().with(fmt_layer).init();
        }
    }

    // TODO: add file-appender tracing subscriber in the future
}

/// Enable parking lot's deadlock detection.
pub fn enable_parking_lot_deadlock_detection() {
    // TODO: deadlock detection as a feature instead of always enabling
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(3));
        let deadlocks = parking_lot::deadlock::check_deadlock();
        if deadlocks.is_empty() {
            continue;
        }

        println!("{} deadlocks detected", deadlocks.len());
        for (i, threads) in deadlocks.iter().enumerate() {
            println!("Deadlock #{}", i);
            for t in threads {
                println!("Thread Id {:#?}", t.thread_id());
                println!("{:#?}", t.backtrace());
            }
        }
    });
}

/// Common set-up for all piestream binaries. Currently, this includes:
///
/// * Set panic hook to abort the whole process.
pub fn oneshot_common() {
    set_panic_abort();

    if cfg!(debug_assertion) {
        enable_parking_lot_deadlock_detection();
    }
}
