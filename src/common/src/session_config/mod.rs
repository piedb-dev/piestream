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

mod query_mode;
use std::ops::Deref;
use std::str::FromStr;

pub use query_mode::QueryMode;

use crate::error::{ErrorCode, RwError};

// This is a hack, &'static str is not allowed as a const generics argument.
// TODO: refine this using the adt_const_params feature.
const CONFIG_KEYS: [&str; 6] = [
    "RW_IMPLICIT_FLUSH",
    "QUERY_MODE",
    "RW_FORCE_DELTA_JOIN",
    "EXTRA_FLOAT_DIGITS",
    "APPLICATION_NAME",
    "DATE_STYLE",
];
const IMPLICIT_FLUSH: usize = 0;
const QUERY_MODE: usize = 1;
const DELFA_JOIN: usize = 2;
const EXTRA_FLOAT_DIGITS: usize = 3;
const APPLICATION_NAME: usize = 4;
const DATE_STYLE: usize = 5;

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

type ImplicitFlush = ConfigBool<IMPLICIT_FLUSH, false>;
type DeltaJoin = ConfigBool<DELFA_JOIN, false>;
type ApplicationName = ConfigString<APPLICATION_NAME>;
type ExtraFloatDigit = ConfigI32<EXTRA_FLOAT_DIGITS, 1>;
// TODO: We should use more specified type here.
type DateStyle = ConfigString<DATE_STYLE>;

#[derive(Default)]
pub struct ConfigMap {
    /// If `RW_IMPLICIT_FLUSH` is on, then every INSERT/UPDATE/DELETE statement will block
    /// until the entire dataflow is refreshed. In other words, every related table & MV will
    /// be able to see the write.
    implicit_flush: ImplicitFlush,

    /// To force the usage of delta join in streaming execution.
    delta_join: DeltaJoin,

    /// A temporary config variable to force query running in either local or distributed mode.
    /// It will be removed in the future.
    query_mode: QueryMode,

    /// see <https://www.postgresql.org/docs/current/runtime-config-client.html#:~:text=for%20more%20information.-,extra_float_digits,-(integer)>
    extra_float_digit: ExtraFloatDigit,

    /// see <https://www.postgresql.org/docs/14/runtime-config-logging.html#:~:text=What%20to%20Log-,application_name,-(string)>
    application_name: ApplicationName,

    /// see https://www.postgresql.org/docs/current/runtime-config-client.html#GUC-DATESTYLE
    date_style: DateStyle,
}

impl ConfigMap {
    pub fn set(&mut self, key: &str, val: &str) -> Result<(), RwError> {
        if key.eq_ignore_ascii_case(ImplicitFlush::entry_name()) {
            self.implicit_flush = val.parse()?;
        } else if key.eq_ignore_ascii_case(DeltaJoin::entry_name()) {
            self.delta_join = val.parse()?;
        } else if key.eq_ignore_ascii_case(QueryMode::entry_name()) {
            self.query_mode = val.parse()?;
        } else if key.eq_ignore_ascii_case(ExtraFloatDigit::entry_name()) {
            self.extra_float_digit = val.parse()?;
        } else if key.eq_ignore_ascii_case(ApplicationName::entry_name()) {
            self.application_name = val.parse()?;
        } else if key.eq_ignore_ascii_case(DateStyle::entry_name()) {
            self.date_style = val.parse()?;
        } else {
            return Err(ErrorCode::UnrecognizedConfigurationParameter(key.to_string()).into());
        }

        Ok(())
    }

    pub fn get_implicit_flush(&self) -> bool {
        *self.implicit_flush
    }

    pub fn get_delta_join(&self) -> bool {
        *self.delta_join
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
}
