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

use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::io::Write;

use super::{OrderedF32, OrderedF64};
use crate::array::ArrayResult;

pub trait NativeType:
    PartialOrd + PartialEq + Debug + Copy + Send + Sync + Sized + Default + 'static
{
    fn to_protobuf<T: Write>(self, output: &mut T) -> ArrayResult<usize>;
    fn hash_wrapper<H: Hasher>(&self, state: &mut H);
}

impl NativeType for i16 {
    fn to_protobuf<T: Write>(self, output: &mut T) -> ArrayResult<usize> {
        output.write(&self.to_be_bytes()).map_err(Into::into)
    }

    fn hash_wrapper<H: Hasher>(&self, state: &mut H) {
        self.hash(state);
    }
}

impl NativeType for i32 {
    fn to_protobuf<T: Write>(self, output: &mut T) -> ArrayResult<usize> {
        output.write(&self.to_be_bytes()).map_err(Into::into)
    }

    fn hash_wrapper<H: Hasher>(&self, state: &mut H) {
        self.hash(state);
    }
}

impl NativeType for i64 {
    fn to_protobuf<T: Write>(self, output: &mut T) -> ArrayResult<usize> {
        output.write(&self.to_be_bytes()).map_err(Into::into)
    }

    fn hash_wrapper<H: Hasher>(&self, state: &mut H) {
        self.hash(state);
    }
}

impl NativeType for OrderedF32 {
    fn to_protobuf<T: Write>(self, output: &mut T) -> ArrayResult<usize> {
        output.write(&self.to_be_bytes()).map_err(Into::into)
    }

    fn hash_wrapper<H: Hasher>(&self, state: &mut H) {
        state.write_i32(self.0 as i32);
    }
}

impl NativeType for OrderedF64 {
    fn to_protobuf<T: Write>(self, output: &mut T) -> ArrayResult<usize> {
        output.write(&self.to_be_bytes()).map_err(Into::into)
    }

    fn hash_wrapper<H: Hasher>(&self, state: &mut H) {
        state.write_i64(self.0 as i64);
    }
}

impl NativeType for u8 {
    fn to_protobuf<T: Write>(self, output: &mut T) -> ArrayResult<usize> {
        output.write(&self.to_be_bytes()).map_err(Into::into)
    }

    fn hash_wrapper<H: Hasher>(&self, state: &mut H) {
        self.hash(state);
    }
}

impl NativeType for u16 {
    fn to_protobuf<T: Write>(self, output: &mut T) -> ArrayResult<usize> {
        output.write(&self.to_be_bytes()).map_err(Into::into)
    }

    fn hash_wrapper<H: Hasher>(&self, state: &mut H) {
        self.hash(state);
    }
}

impl NativeType for u32 {
    fn to_protobuf<T: Write>(self, output: &mut T) -> ArrayResult<usize> {
        output.write(&self.to_be_bytes()).map_err(Into::into)
    }

    fn hash_wrapper<H: Hasher>(&self, state: &mut H) {
        self.hash(state);
    }
}

impl NativeType for u64 {
    fn to_protobuf<T: Write>(self, output: &mut T) -> ArrayResult<usize> {
        output.write(&self.to_be_bytes()).map_err(Into::into)
    }

    fn hash_wrapper<H: Hasher>(&self, state: &mut H) {
        self.hash(state);
    }
}
