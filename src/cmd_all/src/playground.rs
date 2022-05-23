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

use std::collections::HashMap;
use std::ffi::OsString;
use std::process::Command;

use anyhow::{anyhow, Result};
use clap::StructOpt;
use risedev::{
    CompactorService, ComputeNodeService, ConfigExpander, FrontendService, MetaNodeService,
    ServiceConfig,
};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::signal;

async fn load_risedev_config() -> Result<(Vec<String>, HashMap<String, ServiceConfig>)> {
    let risedev_config = {
        let mut content = String::new();
        File::open("risedev.yml")
            .await?
            .read_to_string(&mut content)
            .await?;
        content
    };
    let risedev_config = ConfigExpander::expand(&risedev_config)?;
    let (steps, services) = ConfigExpander::select(&risedev_config, "playground")?;

    Ok((steps, services))
}

pub enum RisingWaveService {
    Compute(Vec<OsString>),
    Meta(Vec<OsString>),
    Frontend(Vec<OsString>),
    Compactor(Vec<OsString>),
}

pub async fn playground() -> Result<()> {
    eprintln!("launching playground");

    risingwave_logging::oneshot_common();
    risingwave_logging::init_risingwave_logger(false, true);

    let services = if let Ok((steps, services)) = load_risedev_config().await {
        tracing::info!(
            "Launching services from risedev config playground section: {:?}",
            steps
        );
        let mut rw_services = vec![];
        for step in steps {
            match services.get(&step).expect("service not found") {
                ServiceConfig::ComputeNode(c) => {
                    let mut command = Command::new("compute-node");
                    ComputeNodeService::apply_command_args(&mut command, c)?;
                    rw_services.push(RisingWaveService::Compute(
                        command.get_args().map(ToOwned::to_owned).collect(),
                    ));
                }
                ServiceConfig::MetaNode(c) => {
                    let mut command = Command::new("meta-node");
                    MetaNodeService::apply_command_args(&mut command, c)?;
                    rw_services.push(RisingWaveService::Meta(
                        command.get_args().map(ToOwned::to_owned).collect(),
                    ));
                }
                ServiceConfig::FrontendV2(c) => {
                    let mut command = Command::new("frontend-node");
                    FrontendService::apply_command_args(&mut command, c)?;
                    rw_services.push(RisingWaveService::Frontend(
                        command.get_args().map(ToOwned::to_owned).collect(),
                    ));
                }
                ServiceConfig::Compactor(c) => {
                    let mut command = Command::new("compactor");
                    CompactorService::apply_command_args(&mut command, c)?;
                    rw_services.push(RisingWaveService::Compactor(
                        command.get_args().map(ToOwned::to_owned).collect(),
                    ));
                }
                _ => {
                    return Err(anyhow!("unsupported service: {}", step));
                }
            }
        }
        rw_services
    } else {
        tracing::warn!("Failed to load risedev config. All components will be started using the default command line options.");
        vec![
            RisingWaveService::Meta(vec!["--backend".into(), "mem".into()]),
            RisingWaveService::Compute(vec!["--state-store".into(), "hummock+memory".into()]),
            RisingWaveService::Frontend(vec![]),
        ]
    };

    for service in services {
        match service {
            RisingWaveService::Meta(mut opts) => {
                opts.insert(0, "meta-node".into());
                tracing::info!("starting meta-node thread with cli args: {:?}", opts);
                let opts = risingwave_meta::MetaNodeOpts::parse_from(opts);
                tracing::info!("opts: {:#?}", opts);
                let _meta_handle = tokio::spawn(async move { risingwave_meta::start(opts).await });
            }
            RisingWaveService::Compute(mut opts) => {
                opts.insert(0, "compute-node".into());
                tracing::info!("starting compute-node thread with cli args: {:?}", opts);
                let opts = risingwave_compute::ComputeNodeOpts::parse_from(opts);
                tracing::info!("opts: {:#?}", opts);
                let _compute_handle =
                    tokio::spawn(async move { risingwave_compute::start(opts).await });
            }
            RisingWaveService::Frontend(mut opts) => {
                opts.insert(0, "frontend-node".into());
                tracing::info!("starting frontend-node thread with cli args: {:?}", opts);
                let opts = risingwave_frontend::FrontendOpts::parse_from(opts);
                tracing::info!("opts: {:#?}", opts);
                let _frontend_handle =
                    tokio::spawn(async move { risingwave_frontend::start(opts).await });
            }
            RisingWaveService::Compactor(mut opts) => {
                opts.insert(0, "compactor".into());
                tracing::info!("starting compactor thread with cli args: {:?}", opts);
                let opts = risingwave_compactor::CompactorOpts::parse_from(opts);
                tracing::info!("opts: {:#?}", opts);
                let _compactor_handle =
                    tokio::spawn(async move { risingwave_compactor::start(opts).await });
            }
        }
    }

    // TODO: should we join all handles?
    // Currently, not all services can be shutdown gracefully, just quit on Ctrl-C now.
    signal::ctrl_c().await.unwrap();
    tracing::info!("Ctrl+C received, now exiting");

    Ok(())
}
