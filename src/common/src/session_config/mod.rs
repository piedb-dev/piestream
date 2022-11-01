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

mod query_mode;
use std::ops::Deref;
use std::str::FromStr;

pub use query_mode::QueryMode;

use crate::error::{ErrorCode, RwError};

// This is a hack, &'static str is not allowed as a const generics argument.
// TODO: refine this using the adt_const_params feature.
const CONFIG_KEYS: [&str; 8] = [
    "RW_IMPLICIT_FLUSH",
    "CREATE_COMPACTION_GROUP_FOR_MV",
    "QUERY_MODE",
    "EXTRA_FLOAT_DIGITS",
    "APPLICATION_NAME",
    "DATESTYLE",
    "RW_BATCH_ENABLE_LOOKUP_JOIN",
    "MAX_SPLIT_RANGE_GAP",
];

// MUST HAVE 1v1 relationship to CONFIG_KEYS. e.g. CONFIG_KEYS[IMPLICIT_FLUSH] =
// "RW_IMPLICIT_FLUSH".
const IMPLICIT_FLUSH: usize = 0;
const CREATE_COMPACTION_GROUP_FOR_MV: usize = 1;
const QUERY_MODE: usize = 2;
const EXTRA_FLOAT_DIGITS: usize = 3;
const APPLICATION_NAME: usize = 4;
const DATE_STYLE: usize = 5;
const BATCH_ENABLE_LOOKUP_JOIN: usize = 6;
const MAX_SPLIT_RANGE_GAP: usize = 7;

trait ConfigEntry: Default + FromStr<Err = RwError> {
    fn entry_name() -> &'static str;
}

struct ConfigBool<const NAME: usize, const DEFAULT: bool = false>(bool);

impl<const NAME: usize, const DEFAULT: bool> Default for ConfigBool<NAME, DEFAULT> {
    fn default() -> Self {
        ConfigBool(DEFAULT)
    }
}

impl<const NAME: usize, const DEFAULT: bool> ConfigEntry for ConfigBool<NAME, DEFAULT> {
    fn entry_name() -> &'static str {
        CONFIG_KEYS[NAME]
    }
}

impl<const NAME: usize, const DEFAULT: bool> FromStr for ConfigBool<NAME, DEFAULT> {
    type Err = RwError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("true") {
            Ok(ConfigBool(true))
        } else if s.eq_ignore_ascii_case("false") {
            Ok(ConfigBool(false))
        } else {
            Err(ErrorCode::InvalidConfigValue {
                config_entry: Self::entry_name().to_string(),
                config_value: s.to_string(),
            }
            .into())
        }
    }
}

impl<const NAME: usize, const DEFAULT: bool> Deref for ConfigBool<NAME, DEFAULT> {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
struct ConfigString<const NAME: usize>(String);

impl<const NAME: usize> Deref for ConfigString<NAME> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const NAME: usize> FromStr for ConfigString<NAME> {
    type Err = RwError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl<const NAME: usize> ConfigEntry for ConfigString<NAME> {
    fn entry_name() -> &'static str {
        CONFIG_KEYS[NAME]
    }
}

struct ConfigI32<const NAME: usize, const DEFAULT: i32 = 0>(i32);

impl<const NAME: usize, const DEFAULT: i32> Default for ConfigI32<NAME, DEFAULT> {
    fn default() -> Self {
        ConfigI32(DEFAULT)
    }
}

impl<const NAME: usize, const DEFAULT: i32> Deref for ConfigI32<NAME, DEFAULT> {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const NAME: usize, const DEFAULT: i32> ConfigEntry for ConfigI32<NAME, DEFAULT> {
    fn entry_name() -> &'static str {
        CONFIG_KEYS[NAME]
    }
}

impl<const NAME: usize, const DEFAULT: i32> FromStr for ConfigI32<NAME, DEFAULT> {
    type Err = RwError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<i32>().map(ConfigI32).map_err(|_e| {
            ErrorCode::InvalidConfigValue {
                config_entry: Self::entry_name().to_string(),
                config_value: s.to_string(),
            }
            .into()
        })
    }
}

pub struct VariableInfo {
    pub name: String,
    pub setting: String,
    pub description: String,
}

type ImplicitFlush = ConfigBool<IMPLICIT_FLUSH, false>;
type CreateCompactionGroupForMv = ConfigBool<CREATE_COMPACTION_GROUP_FOR_MV, false>;
type ApplicationName = ConfigString<APPLICATION_NAME>;
type ExtraFloatDigit = ConfigI32<EXTRA_FLOAT_DIGITS, 1>;
// TODO: We should use more specified type here.
type DateStyle = ConfigString<DATE_STYLE>;
type BatchEnableLookupJoin = ConfigBool<BATCH_ENABLE_LOOKUP_JOIN, false>;
type MaxSplitRangeGap = ConfigI32<MAX_SPLIT_RANGE_GAP, 8>;

#[derive(Default)]
pub struct ConfigMap {
    /// If `RW_IMPLICIT_FLUSH` is on, then every INSERT/UPDATE/DELETE statement will block
    /// until the entire dataflow is refreshed. In other words, every related table & MV will
    /// be able to see the write.
    implicit_flush: ImplicitFlush,

    /// If `CREATE_COMPACTION_GROUP_FOR_MV` is on, dedicated compaction groups will be created in
    /// MV creation.
    create_compaction_group_for_mv: CreateCompactionGroupForMv,

    /// A temporary config variable to force query running in either local or distributed mode.
    /// It will be removed in the future.
    query_mode: QueryMode,

    /// see <https://www.postgresql.org/docs/current/runtime-config-client.html#:~:text=for%20more%20information.-,extra_float_digits,-(integer)>
    extra_float_digit: ExtraFloatDigit,

    /// see <https://www.postgresql.org/docs/14/runtime-config-logging.html#:~:text=What%20to%20Log-,application_name,-(string)>
    application_name: ApplicationName,

    /// see <https://www.postgresql.org/docs/current/runtime-config-client.html#GUC-DATESTYLE>
    date_style: DateStyle,

    /// To force the usage of lookup join instead of hash join in batch execution
    batch_enable_lookup_join: BatchEnableLookupJoin,

    /// It's the max gap allowed to transform small range scan scan into multi point lookup.
    max_split_range_gap: MaxSplitRangeGap,
}

impl ConfigMap {
    pub fn set(&mut self, key: &str, val: &str) -> Result<(), RwError> {
        if key.eq_ignore_ascii_case(ImplicitFlush::entry_name()) {
            self.implicit_flush = val.parse()?;
        } else if key.eq_ignore_ascii_case(CreateCompactionGroupForMv::entry_name()) {
            self.create_compaction_group_for_mv = val.parse()?;
        } else if key.eq_ignore_ascii_case(QueryMode::entry_name()) {
            self.query_mode = val.parse()?;
        } else if key.eq_ignore_ascii_case(ExtraFloatDigit::entry_name()) {
            self.extra_float_digit = val.parse()?;
        } else if key.eq_ignore_ascii_case(ApplicationName::entry_name()) {
            self.application_name = val.parse()?;
        } else if key.eq_ignore_ascii_case(DateStyle::entry_name()) {
            self.date_style = val.parse()?;
        } else if key.eq_ignore_ascii_case(BatchEnableLookupJoin::entry_name()) {
            self.batch_enable_lookup_join = val.parse()?;
        } else if key.eq_ignore_ascii_case(MaxSplitRangeGap::entry_name()) {
            self.max_split_range_gap = val.parse()?;
        } else {
            return Err(ErrorCode::UnrecognizedConfigurationParameter(key.to_string()).into());
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<String, RwError> {
        if key.eq_ignore_ascii_case(ImplicitFlush::entry_name()) {
            Ok(self.implicit_flush.to_string())
        } else if key.eq_ignore_ascii_case(CreateCompactionGroupForMv::entry_name()) {
            Ok(self.create_compaction_group_for_mv.to_string())
        } else if key.eq_ignore_ascii_case(QueryMode::entry_name()) {
            Ok(self.query_mode.to_string())
        } else if key.eq_ignore_ascii_case(ExtraFloatDigit::entry_name()) {
            Ok(self.extra_float_digit.to_string())
        } else if key.eq_ignore_ascii_case(ApplicationName::entry_name()) {
            Ok(self.application_name.to_string())
        } else if key.eq_ignore_ascii_case(DateStyle::entry_name()) {
            Ok(self.date_style.to_string())
        } else if key.eq_ignore_ascii_case(BatchEnableLookupJoin::entry_name()) {
            Ok(self.batch_enable_lookup_join.to_string())
        } else {
            Err(ErrorCode::UnrecognizedConfigurationParameter(key.to_string()).into())
        }
    }

    pub fn get_all(&self) -> Vec<VariableInfo> {
        vec![
            VariableInfo{
                name : ImplicitFlush::entry_name().to_lowercase(),
                setting : self.implicit_flush.to_string(),
                description : String::from("If `RW_IMPLICIT_FLUSH` is on, then every INSERT/UPDATE/DELETE statement will block until the entire dataflow is refreshed.")
            },
            VariableInfo{
                name : CreateCompactionGroupForMv::entry_name().to_lowercase(),
                setting : self.create_compaction_group_for_mv.to_string(),
                description : String::from("If `CREATE_COMPACTION_GROUP_FOR_MV` is on, dedicated compaction groups will be created in MV creation.")
            },
            VariableInfo{
                name : QueryMode::entry_name().to_lowercase(),
                setting : self.query_mode.to_string(),
                description : String::from("A temporary config variable to force query running in either local or distributed mode.")
            },
            VariableInfo{
                name : ExtraFloatDigit::entry_name().to_lowercase(),
                setting : self.extra_float_digit.to_string(),
                description : String::from("Sets the number of digits displayed for floating-point values.")
            },
            VariableInfo{
                name : ApplicationName::entry_name().to_lowercase(),
                setting : self.application_name.to_string(),
                description : String::from("Sets the application name to be reported in statistics and logs.")
            },
            VariableInfo{
                name : DateStyle::entry_name().to_lowercase(),
                setting : self.date_style.to_string(),
                description : String::from("It is typically set by an application upon connection to the server.")
            },
            VariableInfo{
                name : BatchEnableLookupJoin::entry_name().to_lowercase(),
                setting : self.batch_enable_lookup_join.to_string(),
                description : String::from("To enable the usage of lookup join instead of hash join when possible for local batch execution.")
            },
            VariableInfo{
                name : MaxSplitRangeGap::entry_name().to_lowercase(),
                setting : self.max_split_range_gap.to_string(),
                description : String::from("It's the max gap allowed to transform small range scan scan into multi point lookup.")
            },
        ]
    }

    pub fn get_implicit_flush(&self) -> bool {
        *self.implicit_flush
    }

    pub fn get_create_compaction_group_for_mv(&self) -> bool {
        *self.create_compaction_group_for_mv
    }

    pub fn get_query_mode(&self) -> QueryMode {
        self.query_mode
    }

    pub fn get_extra_float_digit(&self) -> i32 {
        *self.extra_float_digit
    }

    pub fn get_application_name(&self) -> &str {
        &self.application_name
    }

    pub fn get_date_style(&self) -> &str {
        &self.date_style
    }

    pub fn get_batch_enable_lookup_join(&self) -> bool {
        *self.batch_enable_lookup_join
    }

    pub fn get_max_split_range_gap(&self) -> u64 {
        if *self.max_split_range_gap < 0 {
            0
        } else {
            *self.max_split_range_gap as u64
        }
    }
}
