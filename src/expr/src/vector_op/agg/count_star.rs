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

use piestream_common::array::*;
use piestream_common::error::{ErrorCode, Result};
use piestream_common::types::*;

use crate::vector_op::agg::aggregator::Aggregator;
use crate::vector_op::agg::general_sorted_grouper::EqGroups;

pub struct CountStar {
    return_type: DataType,
    result: usize,
    reached_limit: bool,
}

impl CountStar {
    pub fn new(return_type: DataType, result: usize) -> Self {
        Self {
            return_type,
            result,
            reached_limit: false,
        }
    }
}

impl Aggregator for CountStar {
    fn return_type(&self) -> DataType {
        self.return_type.clone()
    }

    fn update(&mut self, input: &DataChunk) -> Result<()> {
        self.result += input.cardinality();
        Ok(())
    }

    fn output(&self, builder: &mut ArrayBuilderImpl) -> Result<()> {
        match builder {
            ArrayBuilderImpl::Int64(b) => b.append(Some(self.result as i64)).map_err(Into::into),
            _ => Err(ErrorCode::InternalError("Unexpected builder for count(*).".into()).into()),
        }
    }

    fn update_and_output_with_sorted_groups(
        &mut self,
        input: &DataChunk,
        builder: &mut ArrayBuilderImpl,
        groups: &EqGroups,
    ) -> Result<()> {
        let builder = match builder {
            ArrayBuilderImpl::Int64(b) => b,
            _ => {
                return Err(
                    ErrorCode::InternalError("Unexpected builder for count(*).".into()).into(),
                )
            }
        };
        // The first element continues the same group in `self.result`. The following
        // groups' sizes are simply distance between group start indices. The distance
        // between last element and `input.cardinality()` is the ongoing group that
        // may continue in following chunks.
        //
        // Since the number of groups in an output chunk is limited, if we reach the limit
        // in the process of counting, we set the `reached_limit` flag and save the start
        // index of previous group to `self.result`.
        let mut groups_iter = groups.starting_indices().iter();
        if let Some(first) = groups_iter.next() {
            let first_count = {
                if self.reached_limit {
                    first - self.result
                } else {
                    first + self.result
                }
            };
            builder.append(Some(first_count as i64))?;
            let mut group_cnt = 1;
            let mut prev = first;
            for g in groups_iter {
                builder.append(Some((g - prev) as i64))?;
                prev = g;
                group_cnt += 1;

                // stop and save state if we reach limit
                if groups.is_reach_limit(group_cnt) {
                    self.reached_limit = true;
                    self.result = *prev;
                    break;
                }
            }
            if group_cnt == groups.len() {
                self.reached_limit = false;
                self.result = input.cardinality() - prev;
            }
        } else {
            self.result += input.cardinality();
        }

        Ok(())
    }

    fn update_with_row(&mut self, input: &DataChunk, row_id: usize) -> Result<()> {
        if let Some(visibility) = input.visibility() {
            if visibility.is_set(row_id)? {
                self.result += 1;
            }
        } else {
            self.result += 1;
        }
        Ok(())
    }
}
