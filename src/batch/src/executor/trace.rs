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

use futures::stream::StreamExt;
use futures_async_stream::try_stream;
use piestream_common::array::DataChunk;
use piestream_common::catalog::Schema;
use piestream_common::error::RwError;
use tracing::event;
use tracing_futures::Instrument;

use crate::executor::{BoxedDataChunkStream, BoxedExecutor, Executor};

/// If tracing is enabled, we build a [`TraceExecutor`] on top of the underlying executor.
/// So the duration of performance-critical operations will be traced, such as open/next/close.
pub struct TraceExecutor {
    child: BoxedExecutor,
    /// Description of input executor
    input_desc: String,
}

impl TraceExecutor {
    pub fn new(child: BoxedExecutor, input_desc: String) -> Self {
        Self { child, input_desc }
    }
}

impl Executor for TraceExecutor {
    fn schema(&self) -> &Schema {
        self.child.schema()
    }

    fn identity(&self) -> &str {
        "TraceExecutor"
    }

    fn execute(self: Box<Self>) -> BoxedDataChunkStream {
        self.do_execute()
    }
}

impl TraceExecutor {
    #[try_stream(boxed, ok = DataChunk, error = RwError)]
    async fn do_execute(self: Box<Self>) {
        let input_desc = self.input_desc.as_str();
        let span_name = format!("{input_desc}_next");
        let mut child_stream = self.child.execute();
        while let Some(chunk) = child_stream
            .next()
            .instrument(tracing::trace_span!(
                "next",
                otel.name = span_name.as_str(),
                next = input_desc,
            ))
            .await
        {
            let chunk = chunk?;
            event!(tracing::Level::TRACE, prev = %input_desc, msg = "chunk", "input = \n{:#?}", 
                chunk);
            yield chunk;
        }
    }
}
