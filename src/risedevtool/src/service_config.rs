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

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct ComputeNodeConfig {
    #[serde(rename = "use")]
    phantom_use: Option<String>,
    pub id: String,

    pub address: String,
    #[serde(with = "string")]
    pub port: u16,
    pub listen_address: String,
    pub exporter_port: u16,
    pub enable_async_stack_trace: bool,
    pub enable_managed_cache: bool,
    pub enable_tiered_cache: bool,

    pub provide_minio: Option<Vec<MinioConfig>>,
    pub provide_meta_node: Option<Vec<MetaNodeConfig>>,
    pub provide_compute_node: Option<Vec<ComputeNodeConfig>>,
    pub provide_aws_s3: Option<Vec<AwsS3Config>>,
    pub provide_jaeger: Option<Vec<JaegerConfig>>,
    pub provide_compactor: Option<Vec<CompactorConfig>>,
    pub user_managed: bool,
    pub enable_in_memory_kv_state_backend: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct MetaNodeConfig {
    #[serde(rename = "use")]
    phantom_use: Option<String>,
    pub id: String,

    pub address: String,
    #[serde(with = "string")]
    pub port: u16,
    pub listen_address: String,
    pub dashboard_port: u16,
    pub exporter_port: u16,

    pub user_managed: bool,

    pub provide_etcd_backend: Option<Vec<EtcdConfig>>,

    pub enable_dashboard_v2: bool,
    pub max_heartbeat_interval_secs: u64,
    pub unsafe_disable_recovery: bool,
    pub max_idle_secs_to_exit: Option<u64>,
    pub vacuum_interval_sec: u64,
    pub collect_gc_watermark_spin_interval_sec: u64,
    pub min_sst_retention_time_sec: u64,
    pub enable_committed_sst_sanity_check: bool,
    pub periodic_compaction_interval_sec: u64,
    pub enable_compaction_deterministic: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct FrontendConfig {
    #[serde(rename = "use")]
    phantom_use: Option<String>,
    pub id: String,

    pub address: String,
    #[serde(with = "string")]
    pub port: u16,
    pub listen_address: String,
    pub exporter_port: u16,

    pub provide_meta_node: Option<Vec<MetaNodeConfig>>,
    pub user_managed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct CompactorConfig {
    #[serde(rename = "use")]
    phantom_use: Option<String>,
    pub id: String,

    pub address: String,
    #[serde(with = "string")]
    pub port: u16,
    pub listen_address: String,
    pub exporter_port: u16,

    pub provide_minio: Option<Vec<MinioConfig>>,
    pub provide_aws_s3: Option<Vec<AwsS3Config>>,
    pub provide_meta_node: Option<Vec<MetaNodeConfig>>,
    pub user_managed: bool,
    pub max_concurrent_task_number: u64,
    pub compaction_worker_threads_number: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct MinioConfig {
    #[serde(rename = "use")]
    phantom_use: Option<String>,
    pub id: String,

    pub address: String,
    #[serde(with = "string")]
    pub port: u16,
    pub listen_address: String,

    pub console_address: String,
    #[serde(with = "string")]
    pub console_port: u16,

    pub root_user: String,
    pub root_password: String,
    pub hummock_bucket: String,

    pub provide_prometheus: Option<Vec<PrometheusConfig>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct EtcdConfig {
    #[serde(rename = "use")]
    phantom_use: Option<String>,
    pub id: String,

    // TODO: only one node etcd is supported.
    pub address: String,
    #[serde(with = "string")]
    pub port: u16,
    pub listen_address: String,

    pub peer_port: u16,
    pub unsafe_no_fsync: bool,

    pub exporter_port: u16,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct PrometheusConfig {
    #[serde(rename = "use")]
    phantom_use: Option<String>,
    pub id: String,

    pub address: String,
    #[serde(with = "string")]
    pub port: u16,
    pub listen_address: String,

    pub remote_write: bool,
    pub remote_write_region: String,
    pub remote_write_url: String,

    pub provide_compute_node: Option<Vec<ComputeNodeConfig>>,
    pub provide_meta_node: Option<Vec<MetaNodeConfig>>,
    pub provide_minio: Option<Vec<MinioConfig>>,
    pub provide_compactor: Option<Vec<CompactorConfig>>,
    pub provide_etcd: Option<Vec<EtcdConfig>>,
    pub provide_redpanda: Option<Vec<RedPandaConfig>>,
    pub provide_frontend: Option<Vec<FrontendConfig>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct GrafanaConfig {
    #[serde(rename = "use")]
    phantom_use: Option<String>,
    pub id: String,
    pub address: String,
    pub listen_address: String,
    pub port: u16,
    pub provide_prometheus: Option<Vec<PrometheusConfig>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct JaegerConfig {
    #[serde(rename = "use")]
    phantom_use: Option<String>,
    pub id: String,
    pub dashboard_address: String,
    pub dashboard_port: u16,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct AwsS3Config {
    #[serde(rename = "use")]
    phantom_use: Option<String>,
    pub id: String,
    pub bucket: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct KafkaConfig {
    #[serde(rename = "use")]
    phantom_use: Option<String>,
    pub id: String,

    pub address: String,
    #[serde(with = "string")]
    pub port: u16,
    pub listen_address: String,

    pub provide_zookeeper: Option<Vec<ZooKeeperConfig>>,
    pub persist_data: bool,
    pub broker_id: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct ZooKeeperConfig {
    #[serde(rename = "use")]
    phantom_use: Option<String>,
    pub id: String,

    pub address: String,
    #[serde(with = "string")]
    pub port: u16,
    pub listen_address: String,

    pub persist_data: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct RedPandaConfig {
    #[serde(rename = "use")]
    phantom_use: Option<String>,
    pub id: String,
    pub internal_port: u16,
    pub outside_port: u16,
    pub address: String,
    pub cpus: usize,
    pub memory: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct RedisConfig {
    #[serde(rename = "use")]
    phantom_use: Option<String>,
    pub id: String,

    pub port: u16,
    pub address: String,
}

/// All service configuration
#[derive(Clone, Debug, PartialEq)]
pub enum ServiceConfig {
    ComputeNode(ComputeNodeConfig),
    MetaNode(MetaNodeConfig),
    Frontend(FrontendConfig),
    Compactor(CompactorConfig),
    Minio(MinioConfig),
    Etcd(EtcdConfig),
    Prometheus(PrometheusConfig),
    Grafana(GrafanaConfig),
    Jaeger(JaegerConfig),
    AwsS3(AwsS3Config),
    Kafka(KafkaConfig),
    Redis(RedisConfig),
    ZooKeeper(ZooKeeperConfig),
    RedPanda(RedPandaConfig),
}

impl ServiceConfig {
    pub fn id(&self) -> &str {
        match self {
            Self::ComputeNode(c) => &c.id,
            Self::MetaNode(c) => &c.id,
            Self::Frontend(c) => &c.id,
            Self::Compactor(c) => &c.id,
            Self::Minio(c) => &c.id,
            Self::Etcd(c) => &c.id,
            Self::Prometheus(c) => &c.id,
            Self::Grafana(c) => &c.id,
            Self::Jaeger(c) => &c.id,
            Self::AwsS3(c) => &c.id,
            Self::ZooKeeper(c) => &c.id,
            Self::Kafka(c) => &c.id,
            Self::Redis(c) => &c.id,
            Self::RedPanda(c) => &c.id,
        }
    }
}

mod string {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}
