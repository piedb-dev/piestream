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

#[cfg(test)]
mod compactor_tests;
#[cfg(all(test, feature = "failpoints"))]
mod failpoint_tests;
#[cfg(test)]
mod local_version_manager_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod state_store_tests;
#[cfg(test)]
mod test_utils;
#[cfg(test)]
mod vacuum_tests;

#[cfg(test)]
mod hummock_read_version_tests;

#[cfg(test)]
mod hummock_storage_tests;
