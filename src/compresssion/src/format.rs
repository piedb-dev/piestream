
/****************************************************************************
 * Copyright (c) 2023, PieStream.
 * All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License"); you may not 
 * use this file except in compliance with the License. You may obtain a copy 
 * of the License at http://www.apache.org/licenses/LICENSE-2.0
 * 
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *   - Redistributions of source code must retain the above copyright
 *     notice, this list of conditions and the following disclaimer.
 *   - Redistributions in binary form must reproduce the above copyright
 *     notice, this list of conditions and the following disclaimer in the
 *     documentation and/or other materials provided with the distribution.
 *   - Neither the name of the author nor the names of its contributors may be
 *     used to endorse or promote products derived from this software without
 *     specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER, AUTHOR OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS;
 * OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
 * WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR
 * OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF
 * ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 ****************************************************************************/

// imported from arrow-rs::parquet::format

/// Supported compression algorithms.
///
/// Codecs added in format version X.Y can be read by readers based on X.Y and later.
/// Codec support may vary between readers based on the format version and
/// libraries available at runtime.
///
/// See Compression.md for a detailed specification of these algorithms.
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CompressionCodec(pub i32);

impl CompressionCodec {
  pub const UNCOMPRESSED: CompressionCodec = CompressionCodec(0);
  pub const SNAPPY: CompressionCodec = CompressionCodec(1);
  pub const GZIP: CompressionCodec = CompressionCodec(2);
  pub const LZO: CompressionCodec = CompressionCodec(3);
  pub const BROTLI: CompressionCodec = CompressionCodec(4);
  pub const LZ4: CompressionCodec = CompressionCodec(5);
  pub const ZSTD: CompressionCodec = CompressionCodec(6);
  pub const LZ4_RAW: CompressionCodec = CompressionCodec(7);
  pub const QCOM: CompressionCodec = CompressionCodec(8);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::UNCOMPRESSED,
    Self::SNAPPY,
    Self::GZIP,
    Self::LZO,
    Self::BROTLI,
    Self::LZ4,
    Self::ZSTD,
    Self::LZ4_RAW,
    Self::QCOM,
  ];
}