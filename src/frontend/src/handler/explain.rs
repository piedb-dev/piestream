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

use std::sync::atomic::Ordering;

use pgwire::pg_field_descriptor::{PgFieldDescriptor, TypeOid};
use pgwire::pg_response::{PgResponse, StatementType};
use pgwire::types::Row;
use piestream_common::error::{ErrorCode, Result};
use piestream_sqlparser::ast::{ExplainOptions, ExplainType, Statement};

use super::create_index::gen_create_index_plan;
use super::create_mv::gen_create_mv_plan;
use super::create_sink::gen_sink_plan;
use super::create_table::gen_create_table_plan;
use super::query::gen_batch_query_plan;
use super::RwPgResponse;
use crate::optimizer::plan_node::Convention;
use crate::scheduler::BatchPlanFragmenter;
use crate::session::OptimizerContext;
use crate::stream_fragmenter::build_graph;
use crate::utils::explain_stream_graph;

pub(super) fn handle_explain(
    context: OptimizerContext,
    stmt: Statement,
    options: ExplainOptions,
    analyze: bool,
) -> Result<RwPgResponse> {
    if analyze {
        return Err(ErrorCode::NotImplemented("explain analyze".to_string(), 4856.into()).into());
    }
    if options.explain_type == ExplainType::Logical {
        return Err(ErrorCode::NotImplemented("explain logical".to_string(), 4856.into()).into());
    }

    let session = context.session_ctx.clone();
    context
        .explain_verbose
        .store(options.verbose, Ordering::Release);
    context
        .explain_trace
        .store(options.trace, Ordering::Release);

    let plan = match stmt {
        Statement::CreateView {
            or_replace: false,
            materialized: true,
            query,
            name,
            ..
        } => gen_create_mv_plan(&session, context.into(), *query, name)?.0,

        Statement::CreateSink { stmt } => gen_sink_plan(&session, context.into(), stmt)?.0,

        Statement::CreateTable {
            name,
            columns,
            constraints,
            ..
        } => gen_create_table_plan(&session, context.into(), name, columns, constraints)?.0,

        Statement::CreateIndex {
            name,
            table_name,
            columns,
            include,
            distributed_by,
            ..
        } => {
            gen_create_index_plan(
                &session,
                context.into(),
                name,
                table_name,
                columns,
                include,
                distributed_by,
            )?
            .0
        }

        stmt => gen_batch_query_plan(&session, context.into(), stmt)?.0,
    };

    let ctx = plan.plan_base().ctx.clone();
    let explain_trace = ctx.is_explain_trace();
    let explain_verbose = ctx.is_explain_verbose();

    let mut rows = if explain_trace {
        let trace = ctx.take_trace();
        trace
            .iter()
            .flat_map(|s| s.lines())
            .map(|s| Row::new(vec![Some(s.to_string().into())]))
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    if options.explain_type == ExplainType::DistSql {
        match plan.convention() {
            Convention::Logical => unreachable!(),
            Convention::Batch => {
                let plan_fragmenter = BatchPlanFragmenter::new(
                    session.env().worker_node_manager_ref(),
                    session.env().catalog_reader().clone(),
                );
                let query = plan_fragmenter.split(plan)?;
                let stage_graph_json = serde_json::to_string_pretty(&query.stage_graph).unwrap();
                rows.extend(
                    vec![stage_graph_json]
                        .iter()
                        .flat_map(|s| s.lines())
                        .map(|s| Row::new(vec![Some(s.to_string().into())])),
                );
            }
            Convention::Stream => {
                let graph = build_graph(plan);
                rows.extend(
                    explain_stream_graph(&graph, explain_verbose)?
                        .lines()
                        .map(|s| Row::new(vec![Some(s.to_string().into())])),
                );
            }
        }
    } else {
        // if explain trace is open, the plan has been in the rows
        if !explain_trace {
            let output = plan.explain_to_string()?;
            rows.extend(
                output
                    .lines()
                    .map(|s| Row::new(vec![Some(s.to_string().into())])),
            );
        }
    }

    Ok(PgResponse::new_for_stream(
        StatementType::EXPLAIN,
        Some(rows.len() as i32),
        rows.into(),
        vec![PgFieldDescriptor::new(
            "QUERY PLAN".to_owned(),
            TypeOid::Varchar,
        )],
    ))
}
