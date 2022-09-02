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

use futures_async_stream::for_await;
use log::debug;
use pgwire::pg_field_descriptor::PgFieldDescriptor;
use pgwire::pg_response::{PgResponse, StatementType};
use piestream_batch::executor::BoxedDataChunkStream;
use piestream_common::error::Result;
use piestream_common::session_config::QueryMode;
use piestream_sqlparser::ast::Statement;
use tracing::info;

use crate::binder::{Binder, BoundStatement};
use crate::handler::util::{to_pg_field, to_pg_rows};
use crate::planner::Planner;
use crate::scheduler::{
    BatchPlanFragmenter, ExecutionContext, ExecutionContextRef, LocalQueryExecution,
};
use crate::session::OptimizerContext;
/* dml select类型查询语句的处理流程，*/
pub async fn handle_query(context: OptimizerContext, stmt: Statement) -> Result<PgResponse> {
    let stmt_type = to_statement_type(&stmt);
    let session = context.session_ctx.clone();
    //生成BoundStatement对象，对ast的statment对象做了进一步封装
    let bound = {
        let mut binder = Binder::new(
            session.env().catalog_reader().read_guard(),
            session.database().to_string(),
        );
        binder.bind(stmt)?
    };
    // query_mode = Distributed 分布式，一般指查询模式,一共有二个值Local和Distributed
    let query_mode = session.config().get_query_mode();
    debug!("query_mode:{:?}", query_mode);
    println!("***************query_mode:{:?}", query_mode);

    //data_stream:返回的数据流，pg_descs:字段描述
    let (data_stream, pg_descs) = match query_mode {
        QueryMode::Local => local_execute(context, bound)?,
        QueryMode::Distributed => distribute_execute(context, bound).await?,
    };

    //循环获取数据
    let mut rows = vec![];
    #[for_await]
    for chunk in data_stream {
        rows.extend(to_pg_rows(chunk?));
    }

    //select查询需要返回记录数
    let rows_count = match stmt_type {
        StatementType::SELECT => rows.len() as i32,
        _ => unreachable!(),
    };

    Ok(PgResponse::new(stmt_type, rows_count, rows, pg_descs, true))
}

fn to_statement_type(stmt: &Statement) -> StatementType {
    use StatementType::*;

    match stmt {
        Statement::Query(_) => SELECT,
        _ => unreachable!(),
    }
}

// 分布式执行
async fn distribute_execute(
    context: OptimizerContext,
    stmt: BoundStatement,
) -> Result<(BoxedDataChunkStream, Vec<PgFieldDescriptor>)> {
    let session = context.session_ctx.clone();
    // Subblock to make sure PlanRef (an Rc) is dropped before `await` below.
    let (query, pg_descs) = {
        let root = Planner::new(context.into()).plan(stmt)?;
        let pg_descs = root
            .schema()
            .fields()
            .iter()
            .map(to_pg_field)
            .collect::<Vec<PgFieldDescriptor>>();

        let plan = root.gen_batch_query_plan()?;

        info!(
            "Generated distributed plan: {:?}",
            plan.explain_to_string()?
        );

        let plan_fragmenter = BatchPlanFragmenter::new(session.env().worker_node_manager_ref());
        let query = plan_fragmenter.split(plan)?;
        info!("Generated query after plan fragmenter: {:?}", &query);
        (query, pg_descs)
    };
    // 
    let execution_context: ExecutionContextRef = ExecutionContext::new(session.clone()).into();
    let query_manager = execution_context.session().env().query_manager().clone();
    Ok((
        Box::pin(query_manager.schedule(execution_context, query).await?),
        pg_descs,
    ))
}

fn local_execute(
    context: OptimizerContext,
    stmt: BoundStatement,
) -> Result<(BoxedDataChunkStream, Vec<PgFieldDescriptor>)> {
    let session = context.session_ctx.clone();

    // Subblock to make sure PlanRef (an Rc) is dropped before `await` below.
    let (query, pg_descs) = {
        let root = Planner::new(context.into()).plan(stmt)?;

        let pg_descs = root
            .schema()
            .fields()
            .iter()
            .map(to_pg_field)
            .collect::<Vec<PgFieldDescriptor>>();

        let plan = root.gen_batch_local_plan()?;

        info!(
            "Generated local execution plan: {:?}",
            plan.explain_to_string()?
        );

        let plan_fragmenter = BatchPlanFragmenter::new(session.env().worker_node_manager_ref());
        let query = plan_fragmenter.split(plan)?;
        info!("Generated query after plan fragmenter: {:?}", &query);
        (query, pg_descs)
    };

    let front_env = session.env();

    // TODO: Passing sql here
    let execution = LocalQueryExecution::new(query, front_env.clone(), "", session.auth_context());
    Ok((Box::pin(execution.run()), pg_descs))
}
