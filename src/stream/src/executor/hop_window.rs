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

use std::num::NonZeroUsize;

use futures::StreamExt;
use futures_async_stream::try_stream;
use num_traits::CheckedSub;
use risingwave_common::array::column::Column;
use risingwave_common::array::{DataChunk, StreamChunk};
use risingwave_common::types::{DataType, IntervalUnit, ScalarImpl};
use risingwave_expr::expr::expr_binary_nonnull::new_binary_expr;
use risingwave_expr::expr::{Expression, InputRefExpression, LiteralExpression};
use risingwave_pb::expr::expr_node;

use super::error::StreamExecutorError;
use super::{BoxedExecutor, Executor, ExecutorInfo, Message};

pub struct HopWindowExecutor {
    pub input: BoxedExecutor,
    pub info: ExecutorInfo,

    pub time_col_idx: usize,
    pub window_slide: IntervalUnit,
    pub window_size: IntervalUnit,
}

impl HopWindowExecutor {
    pub fn new(
        input: BoxedExecutor,
        info: ExecutorInfo,
        time_col_idx: usize,
        window_slide: IntervalUnit,
        window_size: IntervalUnit,
    ) -> Self {
        HopWindowExecutor {
            input,
            info,
            time_col_idx,
            window_slide,
            window_size,
        }
    }
}

impl Executor for HopWindowExecutor {
    fn execute(self: Box<Self>) -> super::BoxedMessageStream {
        self.execute_inner().boxed()
    }

    fn schema(&self) -> &risingwave_common::catalog::Schema {
        &self.info.schema
    }

    fn pk_indices(&self) -> super::PkIndicesRef {
        &self.info.pk_indices
    }

    fn identity(&self) -> &str {
        &self.info.identity
    }
}

impl HopWindowExecutor {
    #[try_stream(ok = Message, error = StreamExecutorError)]
    async fn execute_inner(self: Box<Self>) {
        let Self {
            input,
            time_col_idx,
            window_slide,
            window_size,
            ..
        } = *self;
        let units = window_size
            .exact_div(&window_slide)
            .and_then(|x| NonZeroUsize::new(usize::try_from(x).ok()?))
            .ok_or_else(|| {
                StreamExecutorError::invalid_argument(format!(
                    "window_size {} cannot be divided by window_slide {}",
                    window_size, window_slide
                ))
            })?
            .get();

        let schema = self.info.schema;
        let time_col_data_type = schema.fields()[time_col_idx].data_type();
        let time_col_ref = InputRefExpression::new(time_col_data_type, self.time_col_idx).boxed();

        let window_slide_expr =
            LiteralExpression::new(DataType::Interval, Some(ScalarImpl::Interval(window_slide)))
                .boxed();

        // The first window_start of hop window should be:
        // tumble_start(`time_col` - (`window_size` - `window_slide`), `window_slide`).
        // Let's pre calculate (`window_size` - `window_slide`).
        let window_size_sub_slide = window_size.checked_sub(&window_slide).ok_or_else(|| {
            StreamExecutorError::invalid_argument(format!(
                "window_size {} cannot be subtracted by window_slide {}",
                window_size, window_slide
            ))
        })?;
        let window_size_sub_slide_expr = LiteralExpression::new(
            DataType::Interval,
            Some(ScalarImpl::Interval(window_size_sub_slide)),
        )
        .boxed();

        let hop_start = new_binary_expr(
            expr_node::Type::TumbleStart,
            risingwave_common::types::DataType::Timestamp,
            new_binary_expr(
                expr_node::Type::Subtract,
                DataType::Timestamp,
                time_col_ref,
                window_size_sub_slide_expr,
            ),
            window_slide_expr,
        );
        let mut window_start_exprs = Vec::with_capacity(units);
        let mut window_end_exprs = Vec::with_capacity(units);
        for i in 0..units {
            let window_start_offset = window_slide.checked_mul_int(i).ok_or_else(|| {
                StreamExecutorError::invalid_argument(format!(
                    "window_slide {} cannot be multiplied by {}",
                    window_slide, i
                ))
            })?;
            let window_start_offset_expr = LiteralExpression::new(
                DataType::Interval,
                Some(ScalarImpl::Interval(window_start_offset)),
            )
            .boxed();
            let window_end_offset = window_slide.checked_mul_int(i + units).ok_or_else(|| {
                StreamExecutorError::invalid_argument(format!(
                    "window_slide {} cannot be multiplied by {}",
                    window_slide, i
                ))
            })?;
            let window_end_offset_expr = LiteralExpression::new(
                DataType::Interval,
                Some(ScalarImpl::Interval(window_end_offset)),
            )
            .boxed();
            let window_start_expr = new_binary_expr(
                expr_node::Type::Add,
                DataType::Timestamp,
                InputRefExpression::new(DataType::Timestamp, 0).boxed(),
                window_start_offset_expr,
            );
            window_start_exprs.push(window_start_expr);
            let window_end_expr = new_binary_expr(
                expr_node::Type::Add,
                DataType::Timestamp,
                InputRefExpression::new(DataType::Timestamp, 0).boxed(),
                window_end_offset_expr,
            );
            window_end_exprs.push(window_end_expr);
        }

        #[for_await]
        for msg in input.execute() {
            let msg = msg?;
            let Message::Chunk(chunk) = msg else {
                // TODO: syn has not supported `let_else`, we desugar here manually.
                yield std::task::Poll::Ready(msg);
                continue;
            };
            // TODO: compact may be not necessary here.
            let chunk = chunk.compact().map_err(StreamExecutorError::executor_v1)?;
            let (data_chunk, ops) = chunk.into_parts();
            let hop_start = hop_start
                .eval(&data_chunk)
                .map_err(StreamExecutorError::eval_error)?;
            let hop_start_chunk = DataChunk::new(vec![Column::new(hop_start)], None);
            let (origin_cols, visibility) = data_chunk.into_parts();
            // SAFETY: Already compacted.
            assert!(visibility.is_none());
            for i in 0..units {
                let window_start_col = window_start_exprs[i]
                    .eval(&hop_start_chunk)
                    .map_err(StreamExecutorError::eval_error)?;
                let window_end_col = window_end_exprs[i]
                    .eval(&hop_start_chunk)
                    .map_err(StreamExecutorError::eval_error)?;
                let mut new_cols = origin_cols.clone();
                new_cols.extend_from_slice(&[
                    Column::new(window_start_col),
                    Column::new(window_end_col),
                ]);
                let new_chunk = StreamChunk::new(ops.clone(), new_cols, None);
                yield Message::Chunk(new_chunk);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use risingwave_common::array::stream_chunk::StreamChunkTestExt;
    use risingwave_common::catalog::{Field, Schema};
    use risingwave_common::types::{DataType, IntervalUnit};

    use crate::executor::test_utils::MockSource;
    use crate::executor::{Executor, ExecutorInfo, StreamChunk};

    #[madsim::test]
    async fn test_execute() {
        let field1 = Field::unnamed(DataType::Int64);
        let field2 = Field::unnamed(DataType::Int64);
        let field3 = Field::with_name(DataType::Timestamp, "created_at");
        let schema = Schema::new(vec![field1, field2, field3]);
        let pk_indices = vec![0];

        let chunk = StreamChunk::from_pretty(
            &"I I TS
            + 1 1 ^10:00:00
            + 2 3 ^10:05:00
            - 3 2 ^10:14:00
            + 4 1 ^10:22:00
            - 5 3 ^10:33:00
            + 6 2 ^10:42:00
            - 7 1 ^10:51:00
            + 8 3 ^11:02:00"
                .replace('^', "2022-2-2T"),
        );

        let input =
            MockSource::with_chunks(schema.clone(), pk_indices.clone(), vec![chunk]).boxed();

        let window_slide = IntervalUnit::from_minutes(15);
        let window_size = IntervalUnit::from_minutes(30);

        let executor = super::HopWindowExecutor::new(
            input,
            ExecutorInfo {
                // TODO: the schema is incorrect, but it seems useless here.
                schema: schema.clone(),
                pk_indices,
                identity: "test".to_string(),
            },
            2,
            window_slide,
            window_size,
        )
        .boxed();

        let mut stream = executor.execute();
        // TODO: add more test infra to reduce the duplicated codes below.

        let chunk = stream.next().await.unwrap().unwrap().into_chunk().unwrap();
        assert_eq!(
            chunk,
            StreamChunk::from_pretty(
                &"I I TS        TS        TS
                + 1 1 ^10:00:00 ^09:45:00 ^10:15:00
                + 2 3 ^10:05:00 ^09:45:00 ^10:15:00
                - 3 2 ^10:14:00 ^09:45:00 ^10:15:00
                + 4 1 ^10:22:00 ^10:00:00 ^10:30:00
                - 5 3 ^10:33:00 ^10:15:00 ^10:45:00
                + 6 2 ^10:42:00 ^10:15:00 ^10:45:00
                - 7 1 ^10:51:00 ^10:30:00 ^11:00:00
                + 8 3 ^11:02:00 ^10:45:00 ^11:15:00"
                    .replace('^', "2022-2-2T"),
            )
        );

        let chunk = stream.next().await.unwrap().unwrap().into_chunk().unwrap();
        assert_eq!(
            chunk,
            StreamChunk::from_pretty(
                &"I I TS        TS        TS
                + 1 1 ^10:00:00 ^10:00:00 ^10:30:00
                + 2 3 ^10:05:00 ^10:00:00 ^10:30:00
                - 3 2 ^10:14:00 ^10:00:00 ^10:30:00
                + 4 1 ^10:22:00 ^10:15:00 ^10:45:00
                - 5 3 ^10:33:00 ^10:30:00 ^11:00:00
                + 6 2 ^10:42:00 ^10:30:00 ^11:00:00
                - 7 1 ^10:51:00 ^10:45:00 ^11:15:00
                + 8 3 ^11:02:00 ^11:00:00 ^11:30:00"
                    .replace('^', "2022-2-2T"),
            )
        );
    }
}
