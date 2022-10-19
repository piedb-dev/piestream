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

//! This crate includes dependencies that need to be statically-linked.
#[cfg(all(
    not(debug_assertions),
    any(
        not(feature = "enable-static-link"),
        not(feature = "enable-static-log-level"),
    ),
))]
compile_error!(
    "must enable `static-link` and `static-log-level` in release build with `--features \"static-link static-log-level\"`"
);
