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

use itertools::Itertools;
use piestream_common::array::column::Column;
use piestream_common::array::DataChunk;
use piestream_common::error::Result;
use piestream_common::types::Datum;

use crate::SourceColumnDesc;

pub(crate) trait SourceChunkBuilder {
    fn build_columns<'a>(
        column_descs: &[SourceColumnDesc],
        rows: impl IntoIterator<Item = &'a Vec<Datum>>,
        chunk_size: usize,
    ) -> Result<Vec<Column>> {
        let mut builders: Vec<_> = column_descs
            .iter()
            .map(|k| k.data_type.create_array_builder(chunk_size))
            .collect();

        for row in rows {
            for (datum, builder) in row.iter().zip_eq(&mut builders) {
                builder.append_datum(datum);
            }
        }

        Ok(builders
            .into_iter()
            .map(|builder| builder.finish().into())
            .collect())
    }

    fn build_datachunk(
        column_desc: &[SourceColumnDesc],
        rows: &[Vec<Datum>],
        chunk_size: usize,
    ) -> Result<DataChunk> {
        let columns = Self::build_columns(column_desc, rows, chunk_size)?;
        Ok(DataChunk::new(columns, rows.len()))
    }
}
