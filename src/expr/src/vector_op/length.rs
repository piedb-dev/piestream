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

use crate::Result;

#[inline(always)]
pub fn length_default(s: &str) -> Result<i32> {
    Ok(s.chars().count() as i32)
}

#[inline(always)]
pub fn octet_length(s: &str) -> Result<i32> {
    Ok(s.as_bytes().len() as i32)
}

#[inline(always)]
pub fn bit_length(s: &str) -> Result<i32> {
    octet_length(s).map(|n| n * 8)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_length() {
        let cases = [("hello world", 11), ("hello rust", 10)];

        for (s, expected) in cases {
            assert_eq!(length_default(s).unwrap(), expected)
        }
    }

    #[test]
    fn test_octet_length() {
        let cases = [("hello world", 11), ("你好", 6), ("😇哈哈hhh", 13)];

        for (s, expected) in cases {
            assert_eq!(octet_length(s).unwrap(), expected)
        }
    }

    #[test]
    fn test_bit_length() {
        let cases = [
            ("hello world", 11 * 8),
            ("你好", 6 * 8),
            ("😇哈哈hhh", 13 * 8),
        ];

        for (s, expected) in cases {
            assert_eq!(bit_length(s).unwrap(), expected)
        }
    }
}
