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

#![cfg_attr(not(madsim), allow(dead_code))]
#![feature(once_cell)]

use std::path::Path;
use std::sync::LazyLock;
use std::time::Duration;

use clap::Parser;
use rand::{thread_rng, Rng};
use sqllogictest::ParallelTestError;

#[cfg(not(madsim))]
fn main() {
    println!("This binary is only available in simulation.");
}

/// Deterministic simulation end-to-end test runner.
///
/// ENVS:
///
///     RUST_LOG            Set the log level.
///
///     MADSIM_TEST_SEED    Random seed for this run.
///
///     MADSIM_TEST_NUM     The number of runs.
#[derive(Debug, Parser)]
pub struct Args {
    /// Glob of sqllogictest scripts.
    #[clap()]
    files: String,

    /// The number of frontend nodes.
    #[clap(long, default_value = "2")]
    frontend_nodes: usize,

    /// The number of compute nodes.
    #[clap(long, default_value = "3")]
    compute_nodes: usize,

    /// The number of compactor nodes.
    #[clap(long, default_value = "1")]
    compactor_nodes: usize,

    /// The number of CPU cores for each compute node.
    ///
    /// This determines worker_node_parallelism.
    #[clap(long, default_value = "2")]
    compute_node_cores: usize,

    /// The number of clients to run simultaneously.
    ///
    /// If this argument is set, the runner will implicitly create a database for each test file.
    #[clap(short, long)]
    jobs: Option<usize>,

    /// The probability of etcd request timeout.
    #[clap(long, default_value = "0.0")]
    etcd_timeout_rate: f32,

    /// Randomly kill the meta node after each query.
    ///
    /// Currently only available when `-j` is not set.
    #[clap(long)]
    kill_meta: bool,

    /// Randomly kill a frontend node after each query.
    ///
    /// Currently only available when `-j` is not set.
    #[clap(long)]
    kill_frontend: bool,

    /// Randomly kill a compute node after each query.
    ///
    /// Currently only available when `-j` is not set.
    #[clap(long)]
    kill_compute: bool,

    /// Randomly kill a compactor node after each query.
    ///
    /// Currently only available when `-j` is not set.
    #[clap(long)]
    kill_compactor: bool,

    /// The number of sqlsmith test cases to generate.
    ///
    /// If this argument is set, the `files` argument refers to a directory containing sqlsmith
    /// test data.
    #[clap(long)]
    sqlsmith: Option<usize>,
}

static ARGS: LazyLock<Args> = LazyLock::new(Args::parse);

#[cfg(madsim)]
#[madsim::main]
async fn main() {
    let args = &*ARGS;

    let handle = madsim::runtime::Handle::current();
    println!("seed = {}", handle.seed());
    println!("{:?}", args);

    // etcd node
    handle
        .create_node()
        .name("etcd")
        .ip("192.168.10.1".parse().unwrap())
        .init(|| async {
            let addr = "0.0.0.0:2388".parse().unwrap();
            etcd_client::SimServer::builder()
                .timeout_rate(args.etcd_timeout_rate)
                .serve(addr)
                .await
                .unwrap();
        })
        .build();

    // kafka broker
    handle
        .create_node()
        .name("kafka-broker")
        .ip("192.168.11.1".parse().unwrap())
        .init(move || async move {
            rdkafka::SimBroker::default()
                .serve("0.0.0.0:29092".parse().unwrap())
                .await
        })
        .build();

    // wait for the service to be ready
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // meta node
    handle
        .create_node()
        .name("meta")
        .ip("192.168.1.1".parse().unwrap())
        .init(|| async {
            let opts = piestream_meta::MetaNodeOpts::parse_from([
                "meta-node",
                // "--config-path",
                // "src/config/piestream.toml",
                "--listen-addr",
                "0.0.0.0:5690",
                "--backend",
                "etcd",
                "--etcd-endpoints",
                "192.168.10.1:2388",
            ]);
            piestream_meta::start(opts).await
        })
        .build();
    // wait for the service to be ready
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

    // frontend node
    let mut frontend_ip = vec![];
    for i in 1..=args.frontend_nodes {
        frontend_ip.push(format!("192.168.2.{i}"));
        handle
            .create_node()
            .name(format!("frontend-{i}"))
            .ip([192, 168, 2, i as u8].into())
            .init(move || async move {
                let opts = piestream_frontend::FrontendOpts::parse_from([
                    "frontend-node",
                    "--host",
                    "0.0.0.0:4566",
                    "--client-address",
                    &format!("192.168.2.{i}:4566"),
                    "--meta-addr",
                    "192.168.1.1:5690",
                ]);
                piestream_frontend::start(opts).await
            })
            .build();
    }

    // compute node
    for i in 1..=args.compute_nodes {
        let mut builder = handle
            .create_node()
            .name(format!("compute-{i}"))
            .ip([192, 168, 3, i as u8].into())
            .cores(args.compute_node_cores)
            .init(move || async move {
                let opts = piestream_compute::ComputeNodeOpts::parse_from([
                    "compute-node",
                    // "--config-path",
                    // "src/config/piestream.toml",
                    "--host",
                    "0.0.0.0:5688",
                    "--client-address",
                    &format!("192.168.3.{i}:5688"),
                    "--meta-address",
                    "192.168.1.1:5690",
                    "--state-store",
                    "hummock+memory-shared",
                ]);
                piestream_compute::start(opts).await
            });
        if args.kill_compute {
            builder = builder.restart_on_panic();
        }
        builder.build();
    }

    // compactor node
    for i in 1..=args.compactor_nodes {
        handle
            .create_node()
            .name(format!("compactor-{i}"))
            .ip([192, 168, 4, i as u8].into())
            .init(move || async move {
                let opts = piestream_compactor::CompactorOpts::parse_from([
                    "compactor-node",
                    // "--config-path",
                    // "src/config/piestream.toml",
                    "--host",
                    "0.0.0.0:6660",
                    "--client-address",
                    &format!("192.168.4.{i}:6660"),
                    "--meta-address",
                    "192.168.1.1:5690",
                    "--state-store",
                    "hummock+memory-shared",
                ]);
                piestream_compactor::start(opts).await
            })
            .build();
    }

    // prepare data for kafka
    handle
        .create_node()
        .name("kafka-producer")
        .ip("192.168.11.2".parse().unwrap())
        .build()
        .spawn(async move {
            use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
            use rdkafka::error::{KafkaError, RDKafkaErrorCode};
            use rdkafka::producer::{BaseProducer, BaseRecord};
            use rdkafka::ClientConfig;

            let admin = ClientConfig::new()
                .set("bootstrap.servers", "192.168.11.1:29092")
                .create::<AdminClient<_>>()
                .await
                .expect("failed to create kafka admin client");

            let producer = ClientConfig::new()
                .set("bootstrap.servers", "192.168.11.1:29092")
                .create::<BaseProducer>()
                .await
                .expect("failed to create kafka producer");

            for file in std::fs::read_dir("scripts/source/test_data").unwrap() {
                let file = file.unwrap();
                let name = file.file_name().into_string().unwrap();
                let (topic, partitions) = name.split_once('.').unwrap();
                admin
                    .create_topics(
                        &[NewTopic::new(
                            topic,
                            partitions.parse().unwrap(),
                            TopicReplication::Fixed(1),
                        )],
                        &AdminOptions::default(),
                    )
                    .await
                    .expect("failed to create topic");

                let content = std::fs::read(file.path()).unwrap();
                // binary message data, a file is a message
                if topic.ends_with("bin") {
                    loop {
                        let record = BaseRecord::<(), _>::to(topic).payload(&content);
                        match producer.send(record) {
                            Ok(_) => break,
                            Err((
                                KafkaError::MessageProduction(RDKafkaErrorCode::QueueFull),
                                _,
                            )) => {
                                producer.flush(None).await;
                            }
                            Err((e, _)) => panic!("failed to send message: {}", e),
                        }
                    }
                } else {
                    for line in content.split(|&b| b == b'\n') {
                        loop {
                            let record = BaseRecord::<(), _>::to(topic).payload(line);
                            match producer.send(record) {
                                Ok(_) => break,
                                Err((
                                    KafkaError::MessageProduction(RDKafkaErrorCode::QueueFull),
                                    _,
                                )) => {
                                    producer.flush(None).await;
                                }
                                Err((e, _)) => panic!("failed to send message: {}", e),
                            }
                        }
                    }
                }
                producer.flush(None).await;
            }
        });

    // wait for the service to be ready
    tokio::time::sleep(Duration::from_secs(30)).await;
    // client
    let client_node = handle
        .create_node()
        .name("client")
        .ip([192, 168, 100, 1].into())
        .build();

    if let Some(count) = args.sqlsmith {
        client_node
            .spawn(async move {
                let i = rand::thread_rng().gen_range(0..frontend_ip.len());
                let host = frontend_ip[i].clone();
                let rw = piestream::connect(host, "dev".into()).await.unwrap();
                piestream_sqlsmith::runner::run(&rw.client, &args.files, count).await;
            })
            .await
            .unwrap();
        return;
    }

    client_node
        .spawn(async move {
            let glob = &args.files;
            if let Some(jobs) = args.jobs {
                run_parallel_slt_task(glob, &frontend_ip, jobs)
                    .await
                    .unwrap();
            } else {
                let i = rand::thread_rng().gen_range(0..frontend_ip.len());
                run_slt_task(glob, &frontend_ip[i]).await;
            }
        })
        .await
        .unwrap();
}

#[cfg(madsim)]
async fn kill_node() {
    let mut nodes = vec![];
    if ARGS.kill_meta {
        nodes.push(format!("meta"));
    }
    if ARGS.kill_frontend {
        // FIXME: handle postgres connection error
        let i = rand::thread_rng().gen_range(1..=ARGS.frontend_nodes);
        nodes.push(format!("frontend-{}", i));
    }
    if ARGS.kill_compute {
        let i = rand::thread_rng().gen_range(1..=ARGS.compute_nodes);
        nodes.push(format!("compute-{}", i));
    }
    if ARGS.kill_compactor {
        let i = rand::thread_rng().gen_range(1..=ARGS.compactor_nodes);
        nodes.push(format!("compactor-{}", i));
    }
    if nodes.is_empty() {
        return;
    }
    for name in &nodes {
        tracing::info!("kill {name}");
        madsim::runtime::Handle::current().kill(&name);
    }
    tokio::time::sleep(Duration::from_secs(1)).await;
    for name in &nodes {
        tracing::info!("restart {name}");
        madsim::runtime::Handle::current().restart(&name);
    }
}

#[cfg(not(madsim))]
#[allow(clippy::unused_async)]
async fn kill_node() {}

async fn run_slt_task(glob: &str, host: &str) {
    let piestream = piestream::connect(host.to_string(), "dev".to_string())
        .await
        .unwrap();
    let kill = ARGS.kill_compute || ARGS.kill_meta || ARGS.kill_frontend || ARGS.kill_compactor;
    if ARGS.kill_compute || ARGS.kill_meta {
        piestream
            .client
            .simple_query("SET RW_IMPLICIT_FLUSH TO true;")
            .await
            .expect("failed to set");
    }
    piestream
        .client
        .simple_query("SET CREATE_COMPACTION_GROUP_FOR_MV TO true;")
        .await
        .expect("failed to set");
    let mut tester = sqllogictest::Runner::new(piestream);
    let files = glob::glob(glob).expect("failed to read glob pattern");
    for file in files {
        let file = file.unwrap();
        let path = file.as_path();
        println!("{}", path.display());
        // XXX: hack for kafka source test
        let tempfile = path.ends_with("kafka.slt").then(|| hack_kafka_test(path));
        let path = tempfile.as_ref().map(|p| p.path()).unwrap_or(path);
        for record in sqllogictest::parse_file(path).expect("failed to parse file") {
            if let sqllogictest::Record::Halt { .. } = record {
                break;
            }
            let (is_create, is_drop, is_write) =
                if let sqllogictest::Record::Statement { sql, .. } = &record {
                    let sql =
                        (sql.trim_start().split_once(' ').unwrap_or_default().0).to_lowercase();
                    (
                        sql == "create",
                        sql == "drop",
                        sql == "insert" || sql == "update" || sql == "delete" || sql == "flush",
                    )
                } else {
                    (false, false, false)
                };
            // we won't kill during insert/update/delete/flush since the atomicity is not guaranteed
            if !kill || is_write {
                match tester.run_async(record).await {
                    Ok(_) => continue,
                    Err(e) => panic!("{}", e),
                }
            }
            // spawn a background task to kill nodes
            let handle = tokio::spawn(async {
                let t = thread_rng().gen_range(Duration::default()..Duration::from_secs(1));
                tokio::time::sleep(t).await;
                kill_node().await;
                tokio::time::sleep(Duration::from_secs(30)).await;
            });
            // retry up to 5 times until it succeed
            for i in 0usize.. {
                let delay = Duration::from_secs(1 << i);
                match tester.run_async(record.clone()).await {
                    Ok(_) => break,
                    // allow 'table exists' error when retry CREATE statement
                    Err(e)
                        if is_create
                            && i != 0
                            && e.to_string().contains("exists")
                            && e.to_string().contains("Catalog error") =>
                    {
                        break
                    }
                    // allow 'not found' error when retry DROP statement
                    Err(e) if is_drop && i != 0 && e.to_string().contains("not found") => break,
                    Err(e) if i >= 5 => panic!("failed to run test after retry {i} times: {e}"),
                    Err(e) => tracing::error!("failed to run test: {e}\nretry after {delay:?}"),
                }
                tokio::time::sleep(delay).await;
            }
            handle.await.unwrap();
        }
    }
}

async fn run_parallel_slt_task(
    glob: &str,
    hosts: &[String],
    jobs: usize,
) -> Result<(), ParallelTestError> {
    let i = rand::thread_rng().gen_range(0..hosts.len());
    let db = piestream::connect(hosts[i].clone(), "dev".to_string())
        .await
        .unwrap();
    let mut tester = sqllogictest::Runner::new(db);
    tester
        .run_parallel_async(
            glob,
            hosts.to_vec(),
            |host, dbname| async move { piestream::connect(host, dbname).await.unwrap() },
            jobs,
        )
        .await
        .map_err(|e| panic!("{e}"))
}

/// Replace some strings in kafka.slt and write to a new temp file.
fn hack_kafka_test(path: &Path) -> tempfile::NamedTempFile {
    let content = std::fs::read_to_string(path).expect("failed to read file");
    let simple_avsc_full_path =
        std::fs::canonicalize("src/source/src/test_data/simple-schema.avsc")
            .expect("failed to get schema path");
    let complex_avsc_full_path =
        std::fs::canonicalize("src/source/src/test_data/complex-schema.avsc")
            .expect("failed to get schema path");
    let proto_full_path = std::fs::canonicalize("src/source/src/test_data/complex-schema.proto")
        .expect("failed to get schema path");
    let content = content
        .replace("127.0.0.1:29092", "192.168.11.1:29092")
        .replace(
            "/piestream/avro-simple-schema.avsc",
            simple_avsc_full_path.to_str().unwrap(),
        )
        .replace(
            "/piestream/avro-complex-schema.avsc",
            complex_avsc_full_path.to_str().unwrap(),
        )
        .replace(
            "/piestream/proto-complex-schema.proto",
            proto_full_path.to_str().unwrap(),
        );
    let file = tempfile::NamedTempFile::new().expect("failed to create temp file");
    std::fs::write(file.path(), content).expect("failed to write file");
    println!("created a temp file for kafka test: {:?}", file.path());
    file
}

struct piestream {
    client: tokio_postgres::Client,
    task: tokio::task::JoinHandle<()>,
    host: String,
    dbname: String,
}

impl piestream {
    async fn connect(host: String, dbname: String) -> Result<Self, tokio_postgres::error::Error> {
        let (client, connection) = tokio_postgres::Config::new()
            .host(&host)
            .port(4566)
            .dbname(&dbname)
            .user("root")
            .connect_timeout(Duration::from_secs(5))
            .connect(tokio_postgres::NoTls)
            .await?;
        let task = tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!("postgres connection error: {e}");
            }
        });
        Ok(piestream {
            client,
            task,
            host,
            dbname,
        })
    }
}

impl Drop for piestream {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[async_trait::async_trait]
impl sqllogictest::AsyncDB for piestream {
    type Error = tokio_postgres::error::Error;

    async fn run(&mut self, sql: &str) -> Result<String, Self::Error> {
        use std::fmt::Write;

        if self.client.is_closed() {
            // connection error, reset the client
            *self = Self::connect(self.host.clone(), self.dbname.clone()).await?;
        }

        let mut output = String::new();
        let rows = self.client.simple_query(sql).await?;
        for row in rows {
            match row {
                tokio_postgres::SimpleQueryMessage::Row(row) => {
                    for i in 0..row.len() {
                        if i != 0 {
                            write!(output, " ").unwrap();
                        }
                        match row.get(i) {
                            Some(v) if v.is_empty() => write!(output, "(empty)").unwrap(),
                            Some(v) => write!(output, "{}", v).unwrap(),
                            None => write!(output, "NULL").unwrap(),
                        }
                    }
                }
                tokio_postgres::SimpleQueryMessage::CommandComplete(_) => {}
                _ => unreachable!(),
            }
            writeln!(output).unwrap();
        }
        Ok(output)
    }

    fn engine_name(&self) -> &str {
        "piestream"
    }

    async fn sleep(dur: Duration) {
        tokio::time::sleep(dur).await
    }
}
