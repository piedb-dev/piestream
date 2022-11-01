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

use std::fmt;

use itertools::Itertools;
use piestream_pb::data::{Op as ProstOp, StreamChunk as ProstStreamChunk};

use super::{ArrayResult, DataChunkTestExt};
use crate::array::column::Column;
use crate::array::{DataChunk, Row, Vis};
use crate::buffer::Bitmap;
use crate::types::DataType;

/// `Op` represents three operations in `StreamChunk`.
///
/// `UpdateDelete` and `UpdateInsert` are semantically equivalent to `Delete` and `Insert`
/// but always appear in pairs to represent an update operation.
/// For example, table source, aggregation and outer join can generate updates by themselves,
/// while most of the other operators only pass through updates with best effort.
#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub enum Op {
    Insert,
    Delete,
    UpdateDelete,
    UpdateInsert,
}

impl Op {
    pub fn to_protobuf(self) -> ProstOp {
        match self {
            Op::Insert => ProstOp::Insert,
            Op::Delete => ProstOp::Delete,
            Op::UpdateInsert => ProstOp::UpdateInsert,
            Op::UpdateDelete => ProstOp::UpdateDelete,
        }
    }

    pub fn from_protobuf(prost: &i32) -> ArrayResult<Op> {
        let op = match ProstOp::from_i32(*prost) {
            Some(ProstOp::Insert) => Op::Insert,
            Some(ProstOp::Delete) => Op::Delete,
            Some(ProstOp::UpdateInsert) => Op::UpdateInsert,
            Some(ProstOp::UpdateDelete) => Op::UpdateDelete,
            Some(ProstOp::Unspecified) => unreachable!(),
            None => bail!("No such op type"),
        };
        Ok(op)
    }
}

pub type Ops<'a> = &'a [Op];

/// `StreamChunk` is used to pass data over the streaming pathway.
#[derive(Clone, PartialEq)]
pub struct StreamChunk {
    // TODO: Optimize using bitmap
    ops: Vec<Op>,

    pub(super) data: DataChunk,
}

impl Default for StreamChunk {
    /// Create a 0-row-0-col `StreamChunk`. Only used in some existing tests.
    /// This is NOT the same as an **empty** chunk, which has 0 rows but with
    /// columns aligned with executor schema.
    fn default() -> Self {
        Self {
            ops: Default::default(),
            data: DataChunk::new(vec![], 0),
        }
    }
}

impl StreamChunk {
    pub fn new(ops: Vec<Op>, columns: Vec<Column>, visibility: Option<Bitmap>) -> Self {
        for col in &columns {
            assert_eq!(col.array_ref().len(), ops.len());
        }

        let vis = match visibility {
            Some(b) => Vis::Bitmap(b),
            None => Vis::Compact(ops.len()),
        };
        let data = DataChunk::new(columns, vis);
        StreamChunk { ops, data }
    }

    /// Build a `StreamChunk` from rows.
    // TODO: introducing something like `StreamChunkBuilder` maybe better.
    pub fn from_rows(rows: &[(Op, Row)], data_types: &[DataType]) -> Self {
        let mut array_builders = data_types
            .iter()
            .map(|data_type| data_type.create_array_builder(rows.len()))
            .collect::<Vec<_>>();
        let mut ops = vec![];

        for (op, row) in rows {
            ops.push(*op);
            for (datum, builder) in row.0.iter().zip_eq(array_builders.iter_mut()) {
                builder.append_datum(datum);
            }
        }

        let new_columns = array_builders
            .into_iter()
            .map(|builder| builder.finish().into())
            .collect::<Vec<_>>();
        StreamChunk::new(ops, new_columns, None)
    }

    /// `cardinality` return the number of visible tuples
    pub fn cardinality(&self) -> usize {
        self.data.cardinality()
    }

    /// `capacity` return physical length of internals ops & columns
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Get the reference of the underlying data chunk.
    pub fn data_chunk(&self) -> &DataChunk {
        &self.data
    }

    pub fn columns(&self) -> &[Column] {
        self.data.columns()
    }

    pub fn column_at(&self, index: usize) -> &Column {
        self.data.column_at(index)
    }

    /// compact the `StreamChunk` with its visibility map
    pub fn compact(self) -> Self {
        if self.visibility().is_none() {
            return self;
        }

        let (ops, columns, visibility) = self.into_inner();
        let visibility = visibility.unwrap();

        let cardinality = visibility
            .iter()
            .fold(0, |vis_cnt, vis| vis_cnt + vis as usize);
        let columns: Vec<_> = columns
            .into_iter()
            .map(|col| {
                let array = col.array();
                array.compact(&visibility, cardinality).into()
            })
            .collect();
        let mut new_ops = Vec::with_capacity(cardinality);
        for (op, visible) in ops.into_iter().zip_eq(visibility.iter()) {
            if visible {
                new_ops.push(op);
            }
        }
        StreamChunk::new(new_ops, columns, None)
    }

    pub fn into_parts(self) -> (DataChunk, Vec<Op>) {
        (self.data, self.ops)
    }

    pub fn from_parts(ops: Vec<Op>, data_chunk: DataChunk) -> Self {
        let (columns, vis) = data_chunk.into_parts();
        let visibility = match vis {
            Vis::Bitmap(b) => Some(b),
            Vis::Compact(_) => None,
        };
        Self::new(ops, columns, visibility)
    }

    pub fn into_inner(self) -> (Vec<Op>, Vec<Column>, Option<Bitmap>) {
        let (columns, vis) = self.data.into_parts();
        let visibility = match vis {
            Vis::Bitmap(b) => Some(b),
            Vis::Compact(_) => None,
        };
        (self.ops, columns, visibility)
    }

    pub fn to_protobuf(&self) -> ProstStreamChunk {
        ProstStreamChunk {
            cardinality: self.cardinality() as u32,
            ops: self.ops.iter().map(|op| op.to_protobuf() as i32).collect(),
            columns: self.columns().iter().map(|col| col.to_protobuf()).collect(),
        }
    }

    pub fn from_protobuf(prost: &ProstStreamChunk) -> ArrayResult<Self> {
        let cardinality = prost.get_cardinality() as usize;
        let mut ops = Vec::with_capacity(cardinality);
        for op in prost.get_ops() {
            ops.push(Op::from_protobuf(op)?);
        }
        let mut columns = vec![];
        for column in prost.get_columns() {
            columns.push(Column::from_protobuf(column, cardinality)?);
        }
        Ok(StreamChunk::new(ops, columns, None))
    }

    pub fn ops(&self) -> &[Op] {
        &self.ops
    }

    pub fn visibility(&self) -> Option<&Bitmap> {
        self.data.visibility()
    }

    /// `to_pretty_string` returns a table-like text representation of the `StreamChunk`.
    pub fn to_pretty_string(&self) -> String {
        use comfy_table::{Cell, CellAlignment, Table};

        let mut table = Table::new();
        table.load_preset("||--+-++|    ++++++");
        for (op, row_ref) in self.rows() {
            let mut cells = Vec::with_capacity(row_ref.size() + 1);
            cells.push(
                Cell::new(match op {
                    Op::Insert => "+",
                    Op::Delete => "-",
                    Op::UpdateDelete => "U-",
                    Op::UpdateInsert => "U+",
                })
                .set_alignment(CellAlignment::Right),
            );
            for datum in row_ref.values() {
                let str = match datum {
                    None => "".to_owned(), // NULL
                    Some(scalar) => scalar.to_string(),
                };
                cells.push(Cell::new(&str));
            }
            table.add_row(cells);
        }
        table.to_string()
    }

    /// Reorder (and possibly remove) columns. e.g. if `column_mapping` is `[2, 1, 0]`, and
    /// the chunk contains column `[a, b, c]`, then the output will be
    /// `[c, b, a]`. If `column_mapping` is [2, 0], then the output will be `[c, a]`.
    /// If the input mapping is identity mapping, no reorder will be performed.
    pub fn reorder_columns(self, column_mapping: &[usize]) -> Self {
        if column_mapping
            .iter()
            .copied()
            .eq((0..self.data.columns().len()).into_iter())
        {
            // no reorder is needed
            self
        } else {
            Self {
                ops: self.ops,
                data: self.data.reorder_columns(column_mapping),
            }
        }
    }
}

impl fmt::Debug for StreamChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StreamChunk {{ cardinality = {}, capacity = {}, data = \n{} }}",
            self.cardinality(),
            self.capacity(),
            self.to_pretty_string()
        )
    }
}

/// Test utilities for [`StreamChunk`].
pub trait StreamChunkTestExt: Sized {
    fn from_pretty(s: &str) -> Self;

    /// Validate the `StreamChunk` layout.
    fn valid(&self) -> bool;

    /// Concatenate multiple `StreamChunk` into one.
    fn concat(chunks: Vec<Self>) -> Self;

    /// Sort rows.
    fn sort_rows(self) -> Self;
}

impl StreamChunkTestExt for StreamChunk {
    /// Parse a chunk from string.
    ///
    /// See also [`DataChunkTestExt::from_pretty`].
    ///
    /// # Format
    ///
    /// The first line is a header indicating the column types.
    /// The following lines indicate rows within the chunk.
    /// Each line starts with an operation followed by values.
    /// NULL values are represented as `.`.
    ///
    /// # Example
    /// ```
    /// use piestream_common::array::stream_chunk::StreamChunkTestExt as _;
    /// use piestream_common::array::StreamChunk;
    /// let chunk = StreamChunk::from_pretty(
    ///     "  I I I I      // type chars
    ///     U- 2 5 . .      // '.' means NULL
    ///     U+ 2 5 2 6 D    // 'D' means deleted in visibility
    ///     +  . . 4 8      // ^ comments are ignored
    ///     -  . . 3 4",
    /// );
    /// //  ^ operations:
    /// //     +: Insert
    /// //     -: Delete
    /// //    U+: UpdateInsert
    /// //    U-: UpdateDelete
    ///
    /// // type chars:
    /// //     I: i64
    /// //     i: i32
    /// //     F: f64
    /// //     f: f32
    /// //     T: str
    /// //    TS: Timestamp
    /// // {i,f}: struct
    /// ```
    fn from_pretty(s: &str) -> Self {
        let mut chunk_str = String::new();
        let mut ops = vec![];

        let (header, body) = match s.split_once('\n') {
            Some(pair) => pair,
            None => {
                // empty chunk
                return StreamChunk {
                    ops: vec![],
                    data: DataChunk::from_pretty(s),
                };
            }
        };
        chunk_str.push_str(header);
        chunk_str.push('\n');

        for line in body.split_inclusive('\n') {
            if line.trim_start().is_empty() {
                continue;
            }
            let (op, row) = line
                .trim_start()
                .split_once(|c: char| c.is_ascii_whitespace())
                .ok_or_else(|| panic!("missing operation: {line:?}"))
                .unwrap();
            ops.push(match op {
                "+" => Op::Insert,
                "-" => Op::Delete,
                "U+" => Op::UpdateInsert,
                "U-" => Op::UpdateDelete,
                t => panic!("invalid op: {t:?}"),
            });
            chunk_str.push_str(row);
        }
        StreamChunk {
            ops,
            data: DataChunk::from_pretty(&chunk_str),
        }
    }

    fn valid(&self) -> bool {
        let len = self.ops.len();
        let data = &self.data;
        data.vis().len() == len
            && data
                .columns()
                .iter()
                .all(|col| col.array_ref().len() == len)
    }

    fn concat(chunks: Vec<StreamChunk>) -> StreamChunk {
        assert!(!chunks.is_empty());
        let mut ops = vec![];
        let mut data_chunks = vec![];
        let mut capacity = 0;
        for chunk in chunks {
            capacity += chunk.capacity();
            ops.extend(chunk.ops);
            data_chunks.push(chunk.data);
        }
        let data = DataChunk::rechunk(&data_chunks, capacity)
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        StreamChunk { ops, data }
    }

    fn sort_rows(self) -> Self {
        if self.capacity() == 0 {
            return self;
        }
        let rows = self.rows().collect_vec();
        let mut idx = (0..self.capacity()).collect_vec();
        idx.sort_by_key(|&i| &rows[i]);
        StreamChunk {
            ops: idx.iter().map(|&i| self.ops[i]).collect(),
            data: self.data.reorder_rows(&idx),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pretty_string() {
        let chunk = StreamChunk::from_pretty(
            "  I I
             + 1 6
             - 2 .
            U- 3 7
            U+ 4 .",
        );
        assert_eq!(
            chunk.to_pretty_string(),
            "\
+----+---+---+
|  + | 1 | 6 |
|  - | 2 |   |
| U- | 3 | 7 |
| U+ | 4 |   |
+----+---+---+"
        );
    }
}
