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

use piestream_common::error::{ErrorCode, Result};
use piestream_sqlparser::ast::Ident;

use crate::binder::Binder;
use crate::expr::{CorrelatedInputRef, ExprImpl, ExprType, FunctionCall, InputRef};

impl Binder {
    pub fn bind_column(&mut self, idents: &[Ident]) -> Result<ExprImpl> {
        // TODO: check quote style of `ident`.
        let (_schema_name, table_name, column_name) = match idents {
            [column] => (None, None, column.real_value()),
            [table, column] => (None, Some(table.real_value()), column.real_value()),
            [schema, table, column] => (
                Some(schema.real_value()),
                Some(table.real_value()),
                column.real_value(),
            ),
            _ => {
                return Err(
                    ErrorCode::InternalError(format!("Too many idents: {:?}", idents)).into(),
                )
            }
        };

        match self
            .context
            .get_column_binding_indices(&table_name, &column_name)
        {
            Ok(mut indices) => {
                match indices.len() {
                    0 => unreachable!(),
                    1 => {
                        let index = indices[0];
                        let column = &self.context.columns[index];
                        return Ok(
                            InputRef::new(column.index, column.field.data_type.clone()).into()
                        );
                    }
                    _ => {
                        indices.sort(); // make sure we have a consistent result
                        let inputs = indices
                            .iter()
                            .map(|index| {
                                let column = &self.context.columns[*index];
                                InputRef::new(column.index, column.field.data_type.clone()).into()
                            })
                            .collect::<Vec<_>>();
                        return Ok(FunctionCall::new(ExprType::Coalesce, inputs)?.into());
                    }
                }
            }
            Err(e) => {
                // If the error message is not that the column is not found, throw the error
                if let ErrorCode::ItemNotFound(_) = e.inner() {
                } else {
                    return Err(e);
                }
            }
        }

        // Try to find a correlated column in `upper_contexts`, starting from the innermost context.
        let mut err = ErrorCode::ItemNotFound(format!("Invalid column: {}", column_name)).into();
        for (i, (context, _)) in self.upper_subquery_contexts.iter().rev().enumerate() {
            // `depth` starts from 1.
            let depth = i + 1;
            match context.get_column_binding_index(&table_name, &column_name) {
                Ok(index) => {
                    let column = &context.columns[index];
                    return Ok(CorrelatedInputRef::new(
                        column.index,
                        column.field.data_type.clone(),
                        depth,
                    )
                    .into());
                }
                Err(e) => {
                    err = e;
                }
            }
        }
        Err(err)
    }
}
