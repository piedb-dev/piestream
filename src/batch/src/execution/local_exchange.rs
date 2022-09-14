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

use std::fmt::{Debug, Formatter};
use std::future::Future;

use piestream_common::array::DataChunk;
use piestream_common::error::Result;

use crate::exchange_source::ExchangeSource;
use crate::task::{BatchTaskContext, TaskId, TaskOutput, TaskOutputId};

/// Exchange data from a local task execution.
pub struct LocalExchangeSource {
    task_output: TaskOutput,

    /// Id of task which contains the `ExchangeExecutor` of this source.
    task_id: TaskId,
}

impl LocalExchangeSource {
    pub fn create<C: BatchTaskContext>(
        output_id: TaskOutputId,
        context: C,
        task_id: TaskId,
    ) -> Result<Self> {
        //任务输出
        let task_output = context.get_task_output(output_id)?;
        Ok(Self {
            task_output,
            task_id,
        })
    }
}

impl Debug for LocalExchangeSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalExchangeSource")
            .field("task_output_id", self.task_output.id())
            .finish()
    }
}

impl ExchangeSource for LocalExchangeSource {
    type TakeDataFuture<'a> = impl Future<Output = Result<Option<DataChunk>>>;

    fn take_data(&mut self) -> Self::TakeDataFuture<'_> {
        async {
            let ret = self.task_output.direct_take_data().await?;
            if let Some(data) = ret {
                let data = data.compact()?;
                trace!(
                    "Receiver task: {:?}, source task output: {:?}, data: {:?}",
                    self.task_id,
                    self.task_output.id(),
                    data
                );
                Ok(Some(data))
            } else {
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread::sleep;
    use std::time::Duration;

    use piestream_common::util::addr::HostAddr;
    use piestream_pb::batch_plan::{ExchangeSource as ProstExchangeSource, TaskId, TaskOutputId};
    use piestream_pb::data::DataChunk;
    use piestream_pb::task_service::exchange_service_server::{
        ExchangeService, ExchangeServiceServer,
    };
    use piestream_pb::task_service::{
        GetDataRequest, GetDataResponse, GetStreamRequest, GetStreamResponse,
    };
    use tokio_stream::wrappers::ReceiverStream;
    use tonic::{Request, Response, Status};

    use crate::exchange_source::ExchangeSource;
    use crate::execution::grpc_exchange::GrpcExchangeSource;

    struct FakeExchangeService {
        rpc_called: Arc<AtomicBool>,
    }

    #[async_trait::async_trait]
    impl ExchangeService for FakeExchangeService {
        type GetDataStream = ReceiverStream<Result<GetDataResponse, Status>>;
        type GetStreamStream = ReceiverStream<std::result::Result<GetStreamResponse, Status>>;

        async fn get_data(
            &self,
            _: Request<GetDataRequest>,
        ) -> Result<Response<Self::GetDataStream>, Status> {
            println!("***********FakeExchangeService->get_data");
            let (tx, rx) = tokio::sync::mpsc::channel(10);
            self.rpc_called.store(true, Ordering::SeqCst);
            for _ in 0..3 {
                tx.send(Ok(GetDataResponse {
                    status: None,
                    record_batch: Some(DataChunk::default()),
                }))
                .await
                .unwrap();
            }
            Ok(Response::new(ReceiverStream::new(rx)))
        }

        async fn get_stream(
            &self,
            _request: Request<GetStreamRequest>,
        ) -> Result<Response<Self::GetStreamStream>, Status> {
            unimplemented!()
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_exchange_client() {
        let rpc_called = Arc::new(AtomicBool::new(false));
        let server_run = Arc::new(AtomicBool::new(false));
        let addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        // Start a server.
        let (shutdown_send, shutdown_recv) = tokio::sync::oneshot::channel();
        let exchange_svc = ExchangeServiceServer::new(FakeExchangeService {
            rpc_called: rpc_called.clone(),
        });
        let cp_server_run = server_run.clone();
        //GrpcExchangeSource::create里ComputeClient调用此服务
        let join_handle = tokio::spawn(async move {
            cp_server_run.store(true, Ordering::SeqCst);
            tonic::transport::Server::builder()
                .add_service(exchange_svc)
                .serve_with_shutdown(addr, async move {
                    shutdown_recv.await.unwrap();
                })
                .await
                .unwrap();
        });

        sleep(Duration::from_secs(1));
        assert!(server_run.load(Ordering::SeqCst));

        let exchange_source = ProstExchangeSource {
            task_output_id: Some(TaskOutputId {
                task_id: Some(TaskId::default()),
                ..Default::default()
            }),
            host: Some(HostAddr::from(addr).to_protobuf()),
            local_execute_plan: None,
        };
        let mut src = GrpcExchangeSource::create(exchange_source).await.unwrap();
        for _ in 0..3 {
            assert!(src.take_data().await.unwrap().is_some());
        }
        assert!(src.take_data().await.unwrap().is_none());
        assert!(rpc_called.load(Ordering::SeqCst));

        // Gracefully terminate the server.
        shutdown_send.send(()).unwrap();
        join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_unconnectable_node() {
        let addr: HostAddr = "127.0.0.1:1001".parse().unwrap();
        let exchange_source = ProstExchangeSource {
            task_output_id: Some(TaskOutputId {
                task_id: Some(TaskId::default()),
                ..Default::default()
            }),
            host: Some(addr.to_protobuf()),
            local_execute_plan: None,
        };
        let res = GrpcExchangeSource::create(exchange_source).await;
        assert!(res.is_err());
    }
}
