/****************************************************************************
 * Copyright (c) 2023, Haiyong Xie
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

use piestream_common::types::DataType;
use crate::basic::Compression as CodecType;
use crate::compression::{Codec, create_codec, CodecOptionsBuilder};
use crate::errors::{ParquetError, Result};

pub struct PiestreamCompression {
    codectype : CodecType,
    level : usize,
    datatype : DataType,
    codec: Box<dyn Codec>,
}

impl PiestreamCompression {

    pub fn new(c: CodecType, dt: DataType, level: usize) -> Self {
        let codec_options = CodecOptionsBuilder::default()
            .set_backward_compatible_lz4(false)
            .set_type_value(dt.type_to_index())  // this only affects q-compression
            .set_compression_level(level)
            .build();
        let cdc = create_codec(c, &codec_options).unwrap().unwrap();

        Self {
            codectype : c,
            level : level,
            datatype: dt,
            codec: cdc,
        }
    }

    pub fn compress(
        &mut self, 
        input: &[u8], 
        output: &mut Vec<u8>) -> Result<()>
    {
        if input.len() == 0 { return Ok(()); }

        match self.datatype {
            DataType::Int16 
            | DataType::Int32 
            | DataType::Int64 
            | DataType::Float32 
            | DataType::Float64 => {

                self.codec.compress(input, output)

            },

            DataType::Decimal 
            | DataType::Date
            | DataType::Timestamp
            | DataType::Timestampz 
            | DataType::Interval 
            | DataType::Varchar 
            => {
                println!("Now compress data type: {:?}, input: {:?}", self.datatype, input);
                Ok(())
            },

            _ => {
                Err(ParquetError::General(
                    format!("Unsupported data type {:?}", self.datatype).into(),
                ))
            },
        }

        
    }

    pub fn decompress(
        &mut self, 
        input: &[u8], 
        output: &mut Vec<u8>,
        uncompress_size: Option<usize>) -> Result<usize> 
    {
        if input.len() == 0 { return Ok(0); }

        match self.datatype {
            DataType::Int16 
            | DataType::Int32 
            | DataType::Int64 
            | DataType::Float32 
            | DataType::Float64 => {

                self.codec.decompress(input, output, uncompress_size)

            },

            DataType::Decimal 
            | DataType::Date
            | DataType::Timestamp
            | DataType::Timestampz 
            | DataType::Interval 
            | DataType::Varchar 
            => {
                println!("Now compress data type: {:?}, input: {:?}", self.datatype, input);
                Ok(0)
            },

            _ => {
                Err(ParquetError::General(
                    format!("Unsupported data type {:?}", self.datatype).into(),
                ))
            },
        }
    }
}