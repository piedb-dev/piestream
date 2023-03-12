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

 #![allow(unused_imports)]
 
// imported from arrow-rs::parquet::errors

//! Common Parquet errors and macros.
use std::error::Error;
use std::{cell, io, result, str};

#[derive(Debug)]
pub enum ParquetError {
    /// General Parquet error.
    /// Returned when code violates normal workflow of working with Parquet files.
    General(String),
    /// "Not yet implemented" Parquet error.
    /// Returned when functionality is not yet available.
    NYI(String),
    /// "End of file" Parquet error.
    /// Returned when IO related failures occur, e.g. when there are not enough bytes to
    /// decode.
    EOF(String),
    /// Arrow error.
    /// Returned when reading into arrow or writing from arrow.
    ArrowError(String),
    IndexOutOfBound(usize, usize),
    /// An external error variant
    External(Box<dyn Error + Send + Sync>),
}

impl std::fmt::Display for ParquetError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ParquetError::General(message) => {
                write!(fmt, "Parquet error: {message}")
            }
            ParquetError::NYI(message) => write!(fmt, "NYI: {message}"),
            ParquetError::EOF(message) => write!(fmt, "EOF: {message}"),
            ParquetError::ArrowError(message) => write!(fmt, "Arrow: {message}"),
            ParquetError::IndexOutOfBound(index, ref bound) => {
                write!(fmt, "Index {index} out of bound: {bound}")
            }
            ParquetError::External(e) => write!(fmt, "External: {e}"),
        }
    }
}

impl Error for ParquetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParquetError::External(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

impl From<io::Error> for ParquetError {
    fn from(e: io::Error) -> ParquetError {
        ParquetError::External(Box::new(e))
    }
}

impl From<cell::BorrowMutError> for ParquetError {
    fn from(e: cell::BorrowMutError) -> ParquetError {
        ParquetError::External(Box::new(e))
    }
}

impl From<str::Utf8Error> for ParquetError {
    fn from(e: str::Utf8Error) -> ParquetError {
        ParquetError::External(Box::new(e))
    }
}

impl From<snap::Error> for ParquetError {
    fn from(e: snap::Error) -> ParquetError {
        ParquetError::External(Box::new(e))
    }
}

// ----------------------------------------------------------------------
// Conversion from `ParquetError` to other types of `Error`s

impl From<ParquetError> for io::Error {
    fn from(e: ParquetError) -> Self {
        io::Error::new(io::ErrorKind::Other, e)
    }
}

/// A specialized `Result` for Parquet errors.
pub type Result<T, E = ParquetError> = result::Result<T, E>;
