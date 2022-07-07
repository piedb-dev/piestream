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

mod intercept;
pub mod metrics;
pub mod server;
mod service;

pub use service::cluster_service::ClusterServiceImpl;
pub use service::ddl_service::DdlServiceImpl;
pub use service::heartbeat_service::HeartbeatServiceImpl;
pub use service::hummock_service::HummockServiceImpl;
pub use service::notification_service::NotificationServiceImpl;
pub use service::stream_service::StreamServiceImpl;

pub const META_CF_NAME: &str = "cf/meta";
pub const META_LEADER_KEY: &str = "leader";
pub const META_LEASE_KEY: &str = "lease";
