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

mod numeric;
mod timestamp;
mod varchar;

use std::time::Duration;

use anyhow::Result;
pub use numeric::*;
use serde_json::Value;
pub use timestamp::*;
pub use varchar::*;

use crate::types::DataType;

pub const DEFAULT_MIN: i16 = i16::MIN;
pub const DEFAULT_MAX: i16 = i16::MAX;
pub const DEFAULT_START: i16 = 0;
pub const DEFAULT_END: i16 = i16::MAX;

/// default max past for `TimestampField` = 1 day
pub const DEFAULT_MAX_PAST: Duration = Duration::from_secs(60 * 60 * 24);

/// default length for `VarcharField` = 10
pub const DEFAULT_LENGTH: usize = 10;

/// fields that can be randomly generated impl this trait
pub trait NumericFieldRandomGenerator {
    fn new(min: Option<String>, max: Option<String>, seed: u64) -> Result<Self>
    where
        Self: Sized;

    fn generate(&mut self, offset: u64) -> Value;
}

/// fields that can be continuously generated impl this trait
pub trait NumericFieldSequenceGenerator {
    fn new(start: Option<String>, end: Option<String>, offset: u64, step: u64) -> Result<Self>
    where
        Self: Sized;

    fn generate(&mut self) -> Value;
}

/// the way that datagen create the field data. such as 'sequence' or 'random'.
pub enum FieldKind {
    Sequence,
    Random,
}

impl Default for FieldKind {
    fn default() -> Self {
        FieldKind::Random
    }
}

pub enum FieldGeneratorImpl {
    I16Sequence(I16SequenceField),
    I32Sequence(I32SequenceField),
    I64Sequence(I64SequenceField),
    F32Sequence(F32SequenceField),
    F64Sequence(F64SequenceField),
    I16Random(I16RandomField),
    I32Random(I32RandomField),
    I64Random(I64RandomField),
    F32Random(F32RandomField),
    F64Random(F64RandomField),
    Varchar(VarcharField),
    Timestamp(TimestampField),
}

impl FieldGeneratorImpl {
    pub fn with_sequence(
        data_type: DataType,
        start: Option<String>,
        end: Option<String>,
        split_index: u64,
        split_num: u64,
    ) -> Result<Self> {
        match data_type {
            DataType::Int16 => Ok(FieldGeneratorImpl::I16Sequence(I16SequenceField::new(
                start,
                end,
                split_index,
                split_num,
            )?)),
            DataType::Int32 => Ok(FieldGeneratorImpl::I32Sequence(I32SequenceField::new(
                start,
                end,
                split_index,
                split_num,
            )?)),
            DataType::Int64 => Ok(FieldGeneratorImpl::I64Sequence(I64SequenceField::new(
                start,
                end,
                split_index,
                split_num,
            )?)),
            DataType::Float32 => Ok(FieldGeneratorImpl::F32Sequence(F32SequenceField::new(
                start,
                end,
                split_index,
                split_num,
            )?)),
            DataType::Float64 => Ok(FieldGeneratorImpl::F64Sequence(F64SequenceField::new(
                start,
                end,
                split_index,
                split_num,
            )?)),
            _ => unimplemented!(),
        }
    }

    pub fn with_random(
        data_type: DataType,
        min: Option<String>,
        max: Option<String>,
        mast_past: Option<String>,
        length: Option<String>,
        seed: u64,
    ) -> Result<Self> {
        match data_type {
            DataType::Int16 => Ok(FieldGeneratorImpl::I16Random(I16RandomField::new(
                min, max, seed,
            )?)),
            DataType::Int32 => Ok(FieldGeneratorImpl::I32Random(I32RandomField::new(
                min, max, seed,
            )?)),
            DataType::Int64 => Ok(FieldGeneratorImpl::I64Random(I64RandomField::new(
                min, max, seed,
            )?)),
            DataType::Float32 => Ok(FieldGeneratorImpl::F32Random(F32RandomField::new(
                min, max, seed,
            )?)),
            DataType::Float64 => Ok(FieldGeneratorImpl::F64Random(F64RandomField::new(
                min, max, seed,
            )?)),
            DataType::Varchar => Ok(FieldGeneratorImpl::Varchar(VarcharField::new(length)?)),
            DataType::Timestamp => Ok(FieldGeneratorImpl::Timestamp(TimestampField::new(
                mast_past,
            )?)),
            _ => unimplemented!(),
        }
    }

    pub fn generate(&mut self, offset: u64) -> Value {
        match self {
            FieldGeneratorImpl::I16Sequence(f) => f.generate(),
            FieldGeneratorImpl::I32Sequence(f) => f.generate(),
            FieldGeneratorImpl::I64Sequence(f) => f.generate(),
            FieldGeneratorImpl::F32Sequence(f) => f.generate(),
            FieldGeneratorImpl::F64Sequence(f) => f.generate(),
            FieldGeneratorImpl::I16Random(f) => f.generate(offset),
            FieldGeneratorImpl::I32Random(f) => f.generate(offset),
            FieldGeneratorImpl::I64Random(f) => f.generate(offset),
            FieldGeneratorImpl::F32Random(f) => f.generate(offset),
            FieldGeneratorImpl::F64Random(f) => f.generate(offset),
            FieldGeneratorImpl::Varchar(f) => f.generate(),
            FieldGeneratorImpl::Timestamp(f) => f.generate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_sequence() {
        let split_num = 4;
        let mut i32_fields = vec![];
        for split_index in 0..split_num {
            i32_fields.push(
                FieldGeneratorImpl::with_sequence(
                    DataType::Int32,
                    Some("1".to_string()),
                    Some("20".to_string()),
                    split_index,
                    split_num,
                )
                .unwrap(),
            );
        }

        for step in 0..5 {
            for (index, i32_field) in i32_fields.iter_mut().enumerate() {
                let value = i32_field.generate(0);
                assert!(value.is_number());
                let num = value.as_u64();
                let expected_num = split_num * step + 1 + index as u64;
                assert_eq!(expected_num, num.unwrap());
            }
        }
    }
}
