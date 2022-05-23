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

use std::collections::hash_map::Entry;
use std::str::FromStr;

use risingwave_common::catalog::{Field, DEFAULT_SCHEMA_NAME};
use risingwave_common::error::{internal_error, ErrorCode, Result};
use risingwave_sqlparser::ast::{Ident, ObjectName, TableAlias, TableFactor};

use super::bind_context::ColumnBinding;
use crate::binder::Binder;

mod generate_series;
mod join;
mod subquery;
mod table_or_source;
mod window_table_function;
pub use generate_series::BoundGenerateSeriesFunction;
pub use join::BoundJoin;
pub use subquery::BoundSubquery;
pub use table_or_source::{BoundBaseTable, BoundSource, BoundTableSource};
pub use window_table_function::{BoundWindowTableFunction, WindowTableFunctionKind};

/// A validated item that refers to a table-like entity, including base table, subquery, join, etc.
/// It is usually part of the `from` clause.
#[derive(Debug)]
pub enum Relation {
    Source(Box<BoundSource>),
    BaseTable(Box<BoundBaseTable>),
    Subquery(Box<BoundSubquery>),
    Join(Box<BoundJoin>),
    WindowTableFunction(Box<BoundWindowTableFunction>),
    GenerateSeriesFunction(Box<BoundGenerateSeriesFunction>),
}

impl Binder {
    /// return first and second name in identifiers,
    /// must have one name and can use default name as other one.
    fn resolve_double_name(
        mut identifiers: Vec<Ident>,
        err_str: &str,
        default_name: &str,
    ) -> Result<(String, String)> {
        let second_name = identifiers
            .pop()
            .ok_or_else(|| ErrorCode::InternalError(err_str.into()))?
            .value;

        let first_name = identifiers
            .pop()
            .map(|ident| ident.value)
            .unwrap_or_else(|| default_name.into());

        Ok((first_name, second_name))
    }

    /// return the (`schema_name`, `table_name`)
    pub fn resolve_table_name(name: ObjectName) -> Result<(String, String)> {
        Self::resolve_double_name(name.0, "empty table name", DEFAULT_SCHEMA_NAME)
    }

    /// return the ( `database_name`, `schema_name`)
    pub fn resolve_schema_name(
        default_db_name: &str,
        name: ObjectName,
    ) -> Result<(String, String)> {
        Self::resolve_double_name(name.0, "empty schema name", default_db_name)
    }

    /// return the `database_name`
    pub fn resolve_database_name(name: ObjectName) -> Result<String> {
        let mut identifiers = name.0;
        if identifiers.len() > 1 {
            return Err(internal_error("database name must contain 1 argument"));
        }
        let database_name = identifiers
            .pop()
            .ok_or_else(|| internal_error("empty database name"))?
            .value;

        Ok(database_name)
    }

    /// Fill the [`BindContext`](super::BindContext) for table.
    pub(super) fn bind_context(
        &mut self,
        columns: impl IntoIterator<Item = (bool, Field)>, // bool indicates if the field is hidden
        table_name: String,
        alias: Option<TableAlias>,
    ) -> Result<()> {
        let (table_name, column_aliases) = match alias {
            None => (table_name, vec![]),
            Some(TableAlias { name, columns }) => (name.value, columns),
        };

        let begin = self.context.columns.len();
        // Column aliases can be less than columns, but not more.
        // It also needs to skip hidden columns.
        let mut alias_iter = column_aliases.into_iter().fuse();
        columns
            .into_iter()
            .enumerate()
            .for_each(|(index, (is_hidden, mut field))| {
                let name = match is_hidden {
                    true => field.name.to_string(),
                    false => alias_iter
                        .next()
                        .map(|t| t.value)
                        .unwrap_or_else(|| field.name.to_string()),
                };
                field.name = name.clone();
                self.context.columns.push(ColumnBinding::new(
                    table_name.clone(),
                    begin + index,
                    is_hidden,
                    field,
                ));
                self.context
                    .indexs_of
                    .entry(name)
                    .or_default()
                    .push(self.context.columns.len() - 1);
            });
        if alias_iter.next().is_some() {
            return Err(ErrorCode::BindError(format!(
                "table \"{table_name}\" has less columns available but more aliases specified",
            ))
            .into());
        }

        match self.context.range_of.entry(table_name.clone()) {
            Entry::Occupied(_) => Err(ErrorCode::InternalError(format!(
                "Duplicated table name while binding context: {}",
                table_name
            ))
            .into()),
            Entry::Vacant(entry) => {
                entry.insert((begin, self.context.columns.len()));
                Ok(())
            }
        }
    }

    pub(super) fn bind_table_factor(&mut self, table_factor: TableFactor) -> Result<Relation> {
        match table_factor {
            TableFactor::Table { name, alias, args } => {
                if args.is_empty() {
                    let (schema_name, table_name) = Self::resolve_table_name(name)?;
                    self.bind_table_or_source(&schema_name, &table_name, alias)
                } else {
                    let func_name = &name.0[0].value;
                    if func_name.eq_ignore_ascii_case("generate_series") {
                        return Ok(Relation::GenerateSeriesFunction(Box::new(
                            self.bind_generate_series_function(args)?,
                        )));
                    }
                    let kind = WindowTableFunctionKind::from_str(func_name).map_err(|_| {
                        ErrorCode::NotImplemented(
                            format!("unknown window function kind: {}", name.0[0].value),
                            1191.into(),
                        )
                    })?;
                    Ok(Relation::WindowTableFunction(Box::new(
                        self.bind_window_table_function(alias, kind, args)?,
                    )))
                }
            }
            TableFactor::Derived {
                lateral,
                subquery,
                alias,
            } => {
                if lateral {
                    Err(ErrorCode::NotImplemented("unsupported lateral".into(), None.into()).into())
                } else {
                    Ok(Relation::Subquery(Box::new(
                        self.bind_subquery_relation(*subquery, alias)?,
                    )))
                }
            }
            _ => Err(ErrorCode::NotImplemented(
                format!("unsupported table factor {:?}", table_factor),
                None.into(),
            )
            .into()),
        }
    }
}
