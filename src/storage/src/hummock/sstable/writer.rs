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

use bytes::Bytes;

use super::BlockMeta;
use crate::hummock::{HummockResult, SstableBuilderOptions, SstableMeta};

/// A consumer of SST data.
#[async_trait::async_trait]
pub trait SstableWriter: Send {
    type Output;

    /// Write an SST block to the writer.
    async fn write_block(&mut self, block: &[u8], meta: &BlockMeta) -> HummockResult<()>;

    /// Finish writing the SST.
    async fn finish(self, meta: SstableMeta) -> HummockResult<Self::Output>;

    /// Get the length of data that has already been written.
    fn data_len(&self) -> usize;
}

/// Append SST data to a buffer. Used for tests and benchmarks.
pub struct InMemWriter {
    buf: Vec<u8>,
}

impl InMemWriter {
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
        }
    }
}

impl From<&SstableBuilderOptions> for InMemWriter {
    fn from(options: &SstableBuilderOptions) -> Self {
        Self::new(options.capacity + options.block_capacity)
    }
}

#[async_trait::async_trait]
impl SstableWriter for InMemWriter {
    type Output = (Bytes, SstableMeta);

    async fn write_block(&mut self, block: &[u8], _meta: &BlockMeta) -> HummockResult<()> {
        self.buf.extend_from_slice(block);
        Ok(())
    }

    async fn finish(mut self, meta: SstableMeta) -> HummockResult<Self::Output> {
        meta.encode_to(&mut self.buf);
        Ok((Bytes::from(self.buf), meta))
    }

    fn data_len(&self) -> usize {
        self.buf.len()
    }
}

#[cfg(test)]
mod tests {

    use bytes::Bytes;
    use itertools::Itertools;
    use rand::{Rng, SeedableRng};

    use crate::hummock::sstable::VERSION;
    use crate::hummock::{BlockMeta, InMemWriter, SstableMeta, SstableWriter};

    fn get_sst() -> (Bytes, Vec<Bytes>, SstableMeta) {
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        let mut buffer: Vec<u8> = vec![0; 5000];
        rng.fill(&mut buffer[..]);
        buffer.extend((5_u32).to_le_bytes());
        let data = Bytes::from(buffer);

        let mut blocks = Vec::with_capacity(5);
        let mut block_metas = Vec::with_capacity(5);
        for i in 0..5 {
            block_metas.push(BlockMeta {
                smallest_key: Vec::new(),
                len: 1000,
                offset: i * 1000,
                uncompressed_size: 0, // dummy value
                table_id:0,
            });
            blocks.push(data.slice((i * 1000) as usize..((i + 1) * 1000) as usize));
        }
        let meta = SstableMeta {
            block_metas,
            bloom_filter: Vec::new(),
            estimated_size: 0,
            key_count: 0,
            smallest_key: Vec::new(),
            largest_key: Vec::new(),
            meta_offset: data.len() as u64,
            version: VERSION,
        };

        (data, blocks, meta)
    }

    #[tokio::test]
    async fn test_in_mem_writer() {
        let (data, blocks, meta) = get_sst();
        let mut writer = Box::new(InMemWriter::new(0));
        for (block, meta) in blocks.iter().zip_eq(meta.block_metas.iter()) {
            writer.write_block(&block[..], meta).await.unwrap();
        }

        let meta_offset = meta.meta_offset as usize;
        let (output_data, _) = writer.finish(meta).await.unwrap();
        assert_eq!(output_data.slice(0..meta_offset), data);
    }
}
