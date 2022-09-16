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

use std::mem::swap;
use std::sync::Arc;

use itertools::Itertools;

use crate::array::column::Column;
use crate::array::{ArrayBuilderImpl, ArrayResult, DataChunk, RowRef};
use crate::types::{DataType, Datum, DatumRef};

pub const DEFAULT_CHUNK_BUFFER_SIZE: usize = 2048;

/// A [`SlicedDataChunk`] is a [`DataChunk`] with offset.
pub struct SlicedDataChunk {
    data_chunk: DataChunk,
    offset: usize,
}

/// Used as a buffer for accumulating rows.
pub struct DataChunkBuilder {
    /// Data types for build array
    data_types: Vec<DataType>,
    batch_size: usize,

    /// Buffers storing current data
    array_builders: Vec<ArrayBuilderImpl>,
    buffered_count: usize,
}

impl DataChunkBuilder {
    pub fn with_default_size(data_types: Vec<DataType>) -> Self {
        Self::new(data_types, DEFAULT_CHUNK_BUFFER_SIZE)
    }

    pub fn new(data_types: Vec<DataType>, batch_size: usize) -> Self {
        Self {
            data_types,
            batch_size,
            array_builders: vec![],
            buffered_count: 0,
        }
    }

    /// Number of tuples left in one batch.
    #[inline(always)]
    fn left_buffer_count(&self) -> usize {
        self.batch_size - self.buffered_count
    }

    fn ensure_builders(&mut self) -> ArrayResult<()> {
        if self.array_builders.len() != self.data_types.len() {
            self.array_builders = self
                .data_types
                .iter()
                .map(|data_type| data_type.create_array_builder(self.batch_size))
                .collect::<Vec<ArrayBuilderImpl>>();

            ensure!(self.buffered_count == 0);
        }

        Ok(())
    }

    /// Returns not consumed input chunked data as sliced data chunk, and a data chunk of
    /// `batch_size`.
    ///
    /// If `input_chunk` is not totally consumed, it's returned with a new offset, which is equal to
    /// `old_offset + consumed_rows`. Otherwise the first value is `None`.
    ///
    /// If number of `batch_size` rows reached, it's returned as the second value of tuple.
    /// Otherwise it's `None`.
    pub fn append_chunk(
        &mut self,
        input_chunk: SlicedDataChunk,
    ) -> ArrayResult<(Option<SlicedDataChunk>, Option<DataChunk>)> {
        self.ensure_builders()?;

        let mut new_return_offset = input_chunk.offset;
        match input_chunk.data_chunk.visibility() {
            Some(vis) => {
                for vis in vis.iter_from(input_chunk.offset) {
                    new_return_offset += 1;
                    if !vis {
                        continue;
                    }

                    self.append_one_row_internal(&input_chunk.data_chunk, new_return_offset - 1)?;
                    if self.buffered_count >= self.batch_size {
                        break;
                    }
                }
            }
            None => {
                let num_rows_to_append = std::cmp::min(
                    self.batch_size - self.buffered_count,
                    input_chunk.data_chunk.capacity() - input_chunk.offset,
                );
                let end_offset = input_chunk.offset + num_rows_to_append;
                (input_chunk.offset..end_offset).try_for_each(|input_row_idx| {
                    new_return_offset += 1;
                    self.append_one_row_internal(&input_chunk.data_chunk, input_row_idx)
                })?;
            }
        }

        ensure!(self.buffered_count <= self.batch_size);

        let returned_input_chunk = if input_chunk.data_chunk.capacity() > new_return_offset {
            Some(input_chunk.with_new_offset_checked(new_return_offset)?)
        } else {
            None
        };

        let output_chunk = if self.buffered_count == self.batch_size {
            Some(self.build_data_chunk()?)
        } else {
            None
        };

        Ok((returned_input_chunk, output_chunk))
    }

    /// Returns all data in current buffer.
    ///
    /// If `buffered_count` is 0, `None` is returned.
    pub fn consume_all(&mut self) -> ArrayResult<Option<DataChunk>> {
        if self.buffered_count > 0 {
            self.build_data_chunk().map(Some)
        } else {
            Ok(None)
        }
    }

    fn append_one_row_internal(
        &mut self,
        data_chunk: &DataChunk,
        row_idx: usize,
    ) -> ArrayResult<()> {
        self.do_append_one_row_from_datum_refs(data_chunk.row_at(row_idx)?.0.values())
    }

    fn do_append_one_row_from_datum_refs<'a>(
        &mut self,
        datum_refs: impl Iterator<Item = DatumRef<'a>>,
    ) -> ArrayResult<()> {
        self.array_builders
            .iter_mut()
            .zip_eq(datum_refs)
            .try_for_each(|(array_builder, datum_ref)| array_builder.append_datum_ref(datum_ref))?;
        self.buffered_count += 1;
        Ok(())
    }

    fn do_append_one_row_from_datums<'a>(
        &mut self,
        datums: impl Iterator<Item = &'a Datum>,
    ) -> ArrayResult<()> {
        self.array_builders
            .iter_mut()
            .zip_eq(datums)
            .try_for_each(|(array_builder, datum)| array_builder.append_datum(datum))?;
        self.buffered_count += 1;
        Ok(())
    }

    /// Append one row from the given iterator of datum refs.
    /// Return a data chunk if the buffer is full after append one row. Otherwise `None`.
    pub fn append_one_row_from_datum_refs<'a>(
        &mut self,
        datum_refs: impl Iterator<Item = DatumRef<'a>>,
    ) -> ArrayResult<Option<DataChunk>> {
        ensure!(self.buffered_count < self.batch_size);
        self.ensure_builders()?;

        self.do_append_one_row_from_datum_refs(datum_refs)?;
        if self.buffered_count == self.batch_size {
            Ok(Some(self.build_data_chunk()?))
        } else {
            Ok(None)
        }
    }

    /// Append one row from the given `row_ref`.
    /// Return a data chunk if the buffer is full after append one row. Otherwise `None`.
    pub fn append_one_row_ref(&mut self, row_ref: RowRef<'_>) -> ArrayResult<Option<DataChunk>> {
        self.append_one_row_from_datum_refs(row_ref.values())
    }

    /// Append one row from the given iterator of owned datums.
    /// Return a data chunk if the buffer is full after append one row. Otherwise `None`.
    pub fn append_one_row_from_datums<'a>(
        &mut self,
        datums: impl Iterator<Item = &'a Datum>,
    ) -> ArrayResult<Option<DataChunk>> {
        ensure!(self.buffered_count < self.batch_size);
        self.ensure_builders()?;

        self.do_append_one_row_from_datums(datums)?;
        if self.buffered_count == self.batch_size {
            Ok(Some(self.build_data_chunk()?))
        } else {
            Ok(None)
        }
    }

    fn build_data_chunk(&mut self) -> ArrayResult<DataChunk> {
        let mut new_array_builders = vec![];
        swap(&mut new_array_builders, &mut self.array_builders);
        let cardinality = self.buffered_count;
        self.buffered_count = 0;

        let columns = new_array_builders.into_iter().try_fold(
            Vec::with_capacity(self.data_types.len()),
            |mut vec, array_builder| -> ArrayResult<Vec<Column>> {
                let array = array_builder.finish()?;
                let column = Column::new(Arc::new(array));
                vec.push(column);
                Ok(vec)
            },
        )?;
        Ok(DataChunk::new(columns, cardinality))
    }

    pub fn buffered_count(&self) -> usize {
        self.buffered_count
    }
}

impl SlicedDataChunk {
    pub fn new_checked(data_chunk: DataChunk) -> ArrayResult<Self> {
        SlicedDataChunk::with_offset_checked(data_chunk, 0)
    }

    pub fn with_offset_checked(data_chunk: DataChunk, offset: usize) -> ArrayResult<Self> {
        ensure!(offset < data_chunk.capacity());
        Ok(Self { data_chunk, offset })
    }

    pub fn with_new_offset_checked(self, new_offset: usize) -> ArrayResult<Self> {
        SlicedDataChunk::with_offset_checked(self.data_chunk, new_offset)
    }

    fn capacity(&self) -> usize {
        self.data_chunk.capacity() - self.offset
    }
}

#[cfg(test)]
mod tests {
    use crate::array::DataChunk;
    use crate::test_prelude::DataChunkTestExt;
    use crate::types::DataType;
    use crate::util::chunk_coalesce::{DataChunkBuilder, SlicedDataChunk};

    #[test]
    fn test_append_chunk() {
        let mut builder = DataChunkBuilder::new(vec![DataType::Int32, DataType::Int64], 3);

        // Append a chunk with 2 rows
        let input = SlicedDataChunk::new_checked(DataChunk::from_pretty(
            "i I
             3 .
             . 7",
        ))
        .expect("Failed to create sliced data chunk");

        let (returned_input, output) = builder
            .append_chunk(input)
            .expect("Failed to append chunk!");
        assert!(returned_input.is_none());
        assert!(output.is_none());

        // Append a chunk with 4 rows
        let input = SlicedDataChunk::new_checked(DataChunk::from_pretty(
            "i I
             3 .
             . 7
             4 8
             . 9",
        ))
        .expect("Failed to create sliced data chunk");
        let (returned_input, output) = builder
            .append_chunk(input)
            .expect("Failed to append chunk!");
        assert_eq!(Some(1), returned_input.as_ref().map(|c| c.offset));
        assert_eq!(Some(3), output.as_ref().map(DataChunk::cardinality));
        assert_eq!(Some(3), output.as_ref().map(DataChunk::capacity));
        assert!(output.unwrap().visibility().is_none());

        // Append last input
        let (returned_input, output) = builder
            .append_chunk(returned_input.unwrap())
            .expect("Failed to append chunk!");
        assert!(returned_input.is_none());
        assert_eq!(Some(3), output.as_ref().map(DataChunk::cardinality));
        assert_eq!(Some(3), output.as_ref().map(DataChunk::capacity));
        assert!(output.unwrap().visibility().is_none());
    }

    #[test]
    fn test_append_chunk_with_bitmap() {
        let mut builder = DataChunkBuilder::new(vec![DataType::Int32, DataType::Int64], 3);

        // Append a chunk with 2 rows
        let input = SlicedDataChunk::new_checked(DataChunk::from_pretty(
            "i I
             3 .
             . 7 D",
        ))
        .expect("Failed to create sliced data chunk");

        let (returned_input, output) = builder
            .append_chunk(input)
            .expect("Failed to append chunk!");
        assert!(returned_input.is_none());
        assert!(output.is_none());
        assert_eq!(1, builder.buffered_count());

        // Append a chunk with 4 rows
        let input = SlicedDataChunk::new_checked(DataChunk::from_pretty(
            "i I
             3 . D
             . 7
             4 8
             . 9 D",
        ))
        .expect("Failed to create sliced data chunk");
        let (returned_input, output) = builder
            .append_chunk(input)
            .expect("Failed to append chunk!");
        assert_eq!(Some(3), returned_input.as_ref().map(|c| c.offset));
        assert_eq!(Some(3), output.as_ref().map(DataChunk::cardinality));
        assert_eq!(Some(3), output.as_ref().map(DataChunk::capacity));
        assert!(output.unwrap().visibility().is_none());
        assert_eq!(0, builder.buffered_count());

        // Append last input
        let (returned_input, output) = builder
            .append_chunk(returned_input.unwrap())
            .expect("Failed to append chunk!");
        assert!(returned_input.is_none());
        assert!(output.is_none());
        assert_eq!(0, builder.buffered_count());
    }

    #[test]
    fn test_consume_all() {
        let mut builder = DataChunkBuilder::new(vec![DataType::Int32, DataType::Int64], 3);

        // It should return `None` when builder is empty
        assert!(builder.consume_all().unwrap().is_none());

        // Append a chunk with 2 rows
        let input = SlicedDataChunk::new_checked(DataChunk::from_pretty(
            "i I
             3 .
             . 7",
        ))
        .expect("Failed to create sliced data chunk");

        let (returned_input, output) = builder
            .append_chunk(input)
            .expect("Failed to append chunk!");
        assert!(returned_input.is_none());
        assert!(output.is_none());

        let output = builder.consume_all().expect("Failed to consume all!");
        assert!(output.is_some());
        assert_eq!(Some(2), output.as_ref().map(DataChunk::cardinality));
        assert_eq!(Some(2), output.as_ref().map(DataChunk::capacity));
        assert!(output.unwrap().visibility().is_none());
    }
}
