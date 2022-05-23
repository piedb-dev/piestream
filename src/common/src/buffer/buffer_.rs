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

// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! This file is adapted from [arrow-rs](https://github.com/apache/arrow-rs)

use std::mem::{size_of, transmute};
use std::ops::{BitAnd, BitOr};
use std::ptr::NonNull;
use std::slice::{from_raw_parts, from_raw_parts_mut};

use itertools::Itertools;

use crate::alloc::{alloc_aligned, free_aligned};
use crate::error::{ErrorCode, Result};
use crate::types::NativeType;

#[derive(Debug)]
pub struct Buffer {
    ptr: NonNull<u8>,
    len: usize,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        free_aligned(self.len, &self.ptr)
    }
}

impl Clone for Buffer {
    fn clone(&self) -> Self {
        Self::try_from(self.as_slice()).unwrap()
    }
}

impl Buffer {
    /// New a block of memory with content init to 0 (All bits set to 0).
    pub fn with_default(size: usize) -> Result<Buffer> {
        alloc_aligned(size).map(|ptr| {
            // Fill init value.
            unsafe { std::slice::from_raw_parts_mut(ptr.as_ptr(), size).fill(0) }
            Buffer { ptr, len: size }
        })
    }

    pub fn from_slice<T: NativeType, S: AsRef<[T]>>(data: S) -> Result<Buffer> {
        let buffer = Buffer::with_default(data.as_ref().len() * size_of::<T>())?;
        unsafe {
            let dest_slice =
                from_raw_parts_mut::<T>(transmute(buffer.ptr.as_ptr()), data.as_ref().len());
            dest_slice.copy_from_slice(data.as_ref());
        }

        Ok(buffer)
    }

    pub fn typed_data<T: NativeType>(&self) -> &[T] {
        unsafe {
            let (prefix, offsets, suffix) = self.as_slice().align_to::<T>();
            assert!(prefix.is_empty() && suffix.is_empty());
            offsets
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }

    pub fn try_from<T: AsRef<[u8]>>(src: T) -> Result<Self> {
        let buffer = Buffer::from_slice(src)?;
        Ok(buffer)
    }

    fn buffer_bin_op<F>(left: &Buffer, right: &Buffer, op: F) -> Result<Buffer>
    where
        F: Fn(u8, u8) -> u8,
    {
        let ret: Vec<u8> = left
            .as_slice()
            .iter()
            .zip_eq(right.as_slice())
            .map(|a| op(*a.0, *a.1))
            .collect();

        Buffer::try_from(ret)
    }
}

unsafe impl Sync for Buffer {}
unsafe impl Send for Buffer {}

impl<'a, 'b> BitAnd<&'b Buffer> for &'a Buffer {
    type Output = Result<Buffer>;

    fn bitand(self, rhs: &'b Buffer) -> Result<Buffer> {
        if self.len() != rhs.len() {
            return Err(ErrorCode::InternalError(
                "Buffers must be the same size to apply Bitwise AND.".to_string(),
            )
            .into());
        }

        Buffer::buffer_bin_op(self, rhs, |a, b| a & b)
    }
}

impl<'a, 'b> BitOr<&'b Buffer> for &'a Buffer {
    type Output = Result<Buffer>;

    fn bitor(self, rhs: &'b Buffer) -> Result<Buffer> {
        if self.len() != rhs.len() {
            return Err(ErrorCode::InternalError(
                "Buffers must be the same size to apply Bitwise OR.".to_string(),
            )
            .into());
        }

        Buffer::buffer_bin_op(self, rhs, |a, b| a | b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;

    #[test]
    fn test_buffer_from_slice() -> Result<()> {
        let buf = Buffer::from_slice(vec![1i32])?;
        assert_eq!(buf.len(), 4);
        Ok(())
    }

    #[test]
    fn test_buffer_new() {
        let buf = Buffer::with_default(1).unwrap();
        assert_eq!(buf.len(), 1);
    }

    #[test]
    fn test_clone() {
        let buf1 = Buffer::from_slice(vec![1i32]).unwrap();
        let buf2 = buf1.clone();
        assert_eq!(buf1.len(), 4);
        assert_eq!(buf2.len(), 4);
        assert_eq!(buf2.as_slice(), 1i32.to_le_bytes());
    }
}
