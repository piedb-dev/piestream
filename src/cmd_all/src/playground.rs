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

async fn load_risedev_config(
    profile: &str,
) -> Result<(Vec<String>, HashMap<String, ServiceConfig>)> {
    let risedev_config = {
        let mut content = String::new();
        File::open("risedev.yml")
            .await?
            .read_to_string(&mut content)
            .await?;
        content
    };
    let risedev_config = ConfigExpander::expand(&risedev_config, profile)?;
    let (steps, services) = ConfigExpander::select(&risedev_config, profile)?;

    Ok((steps, services))
}

pub enum PiestreamService {
    Compute(Vec<OsString>),
    Meta(Vec<OsString>),
    Frontend(Vec<OsString>),
    Compactor(Vec<OsString>),
}

pub async fn playground() -> Result<()> {
    eprintln!("launching playground");

    piestream_rt::oneshot_common();
    piestream_rt::init_piestream_logger(piestream_rt::LoggerSettings::new_default());

    // Enable tokio console for `./risedev p` by replacing the above statement to:
    // piestream_rt::init_piestream_logger(piestream_rt::LoggerSettings::new(false, true));

    let profile = if let Ok(profile) = std::env::var("PLAYGROUND_PROFILE") {
        profile.to_string()
    } else {
        "playground".to_string()
    };

    let services = match load_risedev_config(&profile).await {
        Ok((steps, services)) => {
            tracing::info!(
                "Launching services from risedev config playground using profile: {}",
                profile
            );
            tracing::info!("steps: {:?}", steps);
            let mut rw_services = vec![];
            for step in steps {
                match services.get(&step).expect("service not found") {
                    ServiceConfig::ComputeNode(c) => {
                        let mut command = Command::new("compute-node");
                        ComputeNodeService::apply_command_args(&mut command, c)?;
                        rw_services.push(PiestreamService::Compute(
                            command.get_args().map(ToOwned::to_owned).collect(),
                        ));
                    }
                    ServiceConfig::MetaNode(c) => {
                        let mut command = Command::new("meta-node");
                        MetaNodeService::apply_command_args(&mut command, c)?;
                        rw_services.push(PiestreamService::Meta(
                            command.get_args().map(ToOwned::to_owned).collect(),
                        ));
                    }
                    ServiceConfig::FrontendV2(c) => {
                        let mut command = Command::new("frontend-node");
                        FrontendService::apply_command_args(&mut command, c)?;
                        rw_services.push(PiestreamService::Frontend(
                            command.get_args().map(ToOwned::to_owned).collect(),
                        ));
                    }
                    ServiceConfig::Compactor(c) => {
                        let mut command = Command::new("compactor");
                        CompactorService::apply_command_args(&mut command, c)?;
                        rw_services.push(PiestreamService::Compactor(
                            command.get_args().map(ToOwned::to_owned).collect(),
                        ));
                    }
                    _ => {
                        return Err(anyhow!("unsupported service: {}", step));
                    }
                }
            }
            rw_services
        }
        Err(e) => {
            tracing::warn!("Failed to load risedev config. All components will be started using the default command line options.\n{}", e);
            vec![
                PiestreamService::Meta(vec!["--backend".into(), "mem".into()]),
                PiestreamService::Compute(vec!["--state-store".into(), "hummock+memory".into()]),
                PiestreamService::Frontend(vec![]),
            ]
        }
    };

    for service in services {
        match service {
            PiestreamService::Meta(mut opts) => {
                opts.insert(0, "meta-node".into());
                tracing::info!("starting meta-node thread with cli args: {:?}", opts);
                let opts = piestream_meta::MetaNodeOpts::parse_from(opts);
                tracing::info!("opts: {:#?}", opts);
                let _meta_handle = tokio::spawn(async move {
                    piestream_meta::start(opts).await;
                    tracing::info!("meta is stopped, shutdown all nodes");
                    // As a playground, it's fine to just kill everything.
                    std::process::exit(0);
                });
                // wait for the service to be ready
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            PiestreamService::Compute(mut opts) => {
                opts.insert(0, "compute-node".into());
                tracing::info!("starting compute-node thread with cli args: {:?}", opts);
                let opts = piestream_compute::ComputeNodeOpts::parse_from(opts);
                tracing::info!("opts: {:#?}", opts);
                let _compute_handle =
                    tokio::spawn(async move { piestream_compute::start(opts).await });
            }
            PiestreamService::Frontend(mut opts) => {
                opts.insert(0, "frontend-node".into());
                tracing::info!("starting frontend-node thread with cli args: {:?}", opts);
                let opts = piestream_frontend::FrontendOpts::parse_from(opts);
                tracing::info!("opts: {:#?}", opts);
                let _frontend_handle =
                    tokio::spawn(async move { piestream_frontend::start(opts).await });
            }
            PiestreamService::Compactor(mut opts) => {
                opts.insert(0, "compactor".into());
                tracing::info!("starting compactor thread with cli args: {:?}", opts);
                let opts = piestream_compactor::CompactorOpts::parse_from(opts);
                tracing::info!("opts: {:#?}", opts);
                let _compactor_handle =
                    tokio::spawn(async move { piestream_compactor::start(opts).await });
            }
        }
    }

    // TODO: should we join all handles?
    // Currently, not all services can be shutdown gracefully, just quit on Ctrl-C now.
    signal::ctrl_c().await.unwrap();
    tracing::info!("Ctrl+C received, now exiting");

    Ok(())
}
