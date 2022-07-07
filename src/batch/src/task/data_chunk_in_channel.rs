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

use anyhow::anyhow;
use either::Either;
use piestream_common::array::DataChunk;
use piestream_common::error::{ErrorCode, Result};
use piestream_pb::data::DataChunk as ProstDataChunk;
use tokio::sync::OnceCell;

#[derive(Debug, Clone)]
pub(super) struct DataChunkInChannel {
    data_chunk: DataChunk,
    /// If the data chunk is only needed to transfer locally,
    /// this field should not be initialized.
    prost_data_chunk: OnceCell<Either<ProstDataChunk, String>>,
}

impl DataChunkInChannel {
    pub fn new(data_chunk: DataChunk) -> Self {
        Self {
            data_chunk,
            prost_data_chunk: OnceCell::new(),
        }
    }

    pub async fn to_protobuf(&self) -> Result<ProstDataChunk> {
        let prost_data_chunk = self
            .prost_data_chunk
            .get_or_init(|| async {
                let res = self.data_chunk.clone().compact();
                match res {
                    Ok(chunk) => Either::Left(chunk.to_protobuf()),
                    Err(e) => Either::Right(format!("{:?}", e)),
                }
            })
            .await;
        // Pass the error message out in this ugly way. Better way to do this?
        match prost_data_chunk {
            Either::Left(chunk) => Ok(chunk.clone()),
            Either::Right(error_msg) => {
                Err(ErrorCode::ArrayError(anyhow!(error_msg.clone()).into()).into())
            }
        }
    }

    pub fn into_data_chunk(self) -> DataChunk {
        self.data_chunk
    }

    pub fn cardinality(&self) -> usize {
        self.data_chunk.cardinality()
    }
}
