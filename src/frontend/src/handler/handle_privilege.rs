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

use pgwire::pg_response::{PgResponse, StatementType};
use piestream_common::error::{ErrorCode, Result};
use piestream_pb::user::grant_privilege::{
    Action as ProstAction, ActionWithGrantOption, Object as ProstObject,
};
use piestream_pb::user::GrantPrivilege as ProstPrivilege;
use piestream_sqlparser::ast::{Action, GrantObjects, Privileges, Statement};

use crate::binder::Binder;
use crate::session::{OptimizerContext, SessionImpl};

// TODO: add user_privilege mod under user manager and move check and expand logic there, and bitmap
// impl for privilege check.
static AVAILABLE_ACTION_ON_DATABASE: &[Action] = &[Action::Connect, Action::Create];
static AVAILABLE_ACTION_ON_SCHEMA: &[Action] = &[Action::Create];
static AVAILABLE_ACTION_ON_SOURCE: &[Action] = &[
    Action::Select { columns: None },
    Action::Update { columns: None },
    Action::Insert { columns: None },
    Action::Delete,
];
static AVAILABLE_ACTION_ON_MVIEW: &[Action] = &[Action::Select { columns: None }];

pub(crate) fn check_privilege_type(privilege: &Privileges, objects: &GrantObjects) -> Result<()> {
    match privilege {
        Privileges::All { .. } => Ok(()),
        Privileges::Actions(actions) => {
            let valid = match objects {
                GrantObjects::Databases(_) => actions
                    .iter()
                    .all(|action| AVAILABLE_ACTION_ON_DATABASE.contains(action)),
                GrantObjects::Schemas(_) => actions
                    .iter()
                    .all(|action| AVAILABLE_ACTION_ON_SCHEMA.contains(action)),
                GrantObjects::Sources(_) | GrantObjects::AllSourcesInSchema { .. } => actions
                    .iter()
                    .all(|action| AVAILABLE_ACTION_ON_SOURCE.contains(action)),
                GrantObjects::Mviews(_) | GrantObjects::AllMviewsInSchema { .. } => actions
                    .iter()
                    .all(|action| AVAILABLE_ACTION_ON_MVIEW.contains(action)),
                _ => true,
            };
            if !valid {
                return Err(ErrorCode::BindError(
                    "Invalid privilege type for the given object.".to_string(),
                )
                .into());
            }

            Ok(())
        }
    }
}

pub(crate) fn available_privilege_actions(objects: &GrantObjects) -> Result<Vec<Action>> {
    match objects {
        GrantObjects::Databases(_) => Ok(AVAILABLE_ACTION_ON_DATABASE.to_vec()),
        GrantObjects::Schemas(_) => Ok(AVAILABLE_ACTION_ON_SCHEMA.to_vec()),
        GrantObjects::Sources(_) | GrantObjects::AllSourcesInSchema { .. } => {
            Ok(AVAILABLE_ACTION_ON_SOURCE.to_vec())
        }
        GrantObjects::Mviews(_) | GrantObjects::AllMviewsInSchema { .. } => {
            Ok(AVAILABLE_ACTION_ON_MVIEW.to_vec())
        }
        _ => Err(
            ErrorCode::BindError("Invalid privilege type for the given object.".to_string()).into(),
        ),
    }
}

fn make_prost_privilege(
    session: &SessionImpl,
    privileges: Privileges,
    objects: GrantObjects,
) -> Result<Vec<ProstPrivilege>> {
    check_privilege_type(&privileges, &objects)?;

    let catalog_reader = session.env().catalog_reader();
    let reader = catalog_reader.read_guard();
    let actions = match privileges {
        Privileges::All { .. } => available_privilege_actions(&objects)?,
        Privileges::Actions(actions) => actions,
    };
    let mut grant_objs = vec![];
    match objects {
        GrantObjects::Databases(databases) => {
            for db in databases {
                let database_name = Binder::resolve_database_name(db)?;
                let database = reader.get_database_by_name(&database_name)?;
                grant_objs.push(ProstObject::DatabaseId(database.id()));
            }
        }
        GrantObjects::Schemas(schemas) => {
            for schema in schemas {
                let (database_name, schema_name) =
                    Binder::resolve_schema_name(session.database(), schema)?;
                let schema = reader.get_schema_by_name(&database_name, &schema_name)?;
                grant_objs.push(ProstObject::SchemaId(schema.id()));
            }
        }
        GrantObjects::Mviews(tables) => {
            for name in tables {
                let (schema_name, table_name) = Binder::resolve_table_name(name)?;
                let table =
                    reader.get_table_by_name(session.database(), &schema_name, &table_name)?;
                grant_objs.push(ProstObject::TableId(table.id().table_id));
            }
        }
        GrantObjects::Sources(sources) => {
            for name in sources {
                let (schema_name, table_name) = Binder::resolve_table_name(name)?;
                let source =
                    reader.get_source_by_name(session.database(), &schema_name, &table_name)?;
                grant_objs.push(ProstObject::SourceId(source.id));
            }
        }
        GrantObjects::AllSourcesInSchema { schemas } => {
            for schema in schemas {
                let (database_name, schema_name) =
                    Binder::resolve_schema_name(session.database(), schema)?;
                let schema = reader.get_schema_by_name(&database_name, &schema_name)?;
                grant_objs.push(ProstObject::AllSourcesSchemaId(schema.id()));
            }
        }
        GrantObjects::AllMviewsInSchema { schemas } => {
            for schema in schemas {
                let (database_name, schema_name) =
                    Binder::resolve_schema_name(session.database(), schema)?;
                let schema = reader.get_schema_by_name(&database_name, &schema_name)?;
                grant_objs.push(ProstObject::AllTablesSchemaId(schema.id()));
            }
        }
        _ => {
            return Err(ErrorCode::BindError(
                "GRANT statement does not support this object type".to_string(),
            )
            .into());
        }
    };
    let action_with_opts = actions
        .iter()
        .map(|action| {
            let prost_action = match action {
                Action::Select { .. } => ProstAction::Select,
                Action::Insert { .. } => ProstAction::Insert,
                Action::Update { .. } => ProstAction::Update,
                Action::Delete { .. } => ProstAction::Delete,
                Action::Connect => ProstAction::Connect,
                Action::Create => ProstAction::Create,
                _ => unreachable!(),
            };
            ActionWithGrantOption {
                action: prost_action as i32,
                granted_by: session.user_name().to_string(),
                ..Default::default()
            }
        })
        .collect::<Vec<_>>();

    let mut prost_privileges = vec![];
    for objs in grant_objs {
        prost_privileges.push(ProstPrivilege {
            action_with_opts: action_with_opts.clone(),
            object: Some(objs),
        });
    }
    Ok(prost_privileges)
}

pub async fn handle_grant_privilege(
    context: OptimizerContext,
    stmt: Statement,
) -> Result<PgResponse> {
    let session = context.session_ctx;
    let Statement::Grant {
        privileges,
        objects,
        grantees,
        with_grant_option,
        granted_by,
    } = stmt else { return Err(ErrorCode::BindError("Invalid grant statement".to_string()).into()); };
    let users = grantees.into_iter().map(|g| g.value).collect::<Vec<_>>();
    {
        let user_reader = session.env().user_info_reader();
        let reader = user_reader.read_guard();
        if users
            .iter()
            .any(|user| reader.get_user_by_name(user).is_none())
        {
            return Err(ErrorCode::BindError("Grantee does not exist".to_string()).into());
        }
        if let Some(granted_by) = &granted_by {
            let user = reader.get_user_by_name(&granted_by.value);
            if user.is_none() {
                return Err(ErrorCode::BindError("Grantor does not exist".to_string()).into());
            }
        }
    }

    let privileges = make_prost_privilege(&session, privileges, objects)?;
    let user_info_writer = session.env().user_info_writer();
    user_info_writer
        .grant_privilege(
            users,
            privileges,
            with_grant_option,
            session.user_name().to_string(),
        )
        .await?;
    Ok(PgResponse::empty_result(StatementType::GRANT_PRIVILEGE))
}

pub async fn handle_revoke_privilege(
    context: OptimizerContext,
    stmt: Statement,
) -> Result<PgResponse> {
    let session = context.session_ctx;
    let Statement::Revoke {
        privileges,
        objects,
        grantees,
        granted_by,
        revoke_grant_option,
        cascade,
    } = stmt else { return Err(ErrorCode::BindError("Invalid revoke statement".to_string()).into()); };
    let users = grantees.into_iter().map(|g| g.value).collect::<Vec<_>>();
    {
        let user_reader = session.env().user_info_reader();
        let reader = user_reader.read_guard();
        if users
            .iter()
            .any(|user| reader.get_user_by_name(user).is_none())
        {
            return Err(ErrorCode::BindError("Grantee does not exist".to_string()).into());
        }
        if let Some(ref granted_by) = granted_by {
            if reader.get_user_by_name(&granted_by.value).is_none() {
                return Err(ErrorCode::BindError("Grantor does not exist".to_string()).into());
            }
            // TODO: check whether if grantor is a super user or have the privilege to grant.
        }
    }
    let privileges = make_prost_privilege(&session, privileges, objects)?;
    let user_info_writer = session.env().user_info_writer();
    let granted_by = granted_by.map(|g| g.value);
    user_info_writer
        .revoke_privilege(
            users,
            privileges,
            granted_by,
            session.user_name().to_string(),
            revoke_grant_option,
            cascade,
        )
        .await?;

    Ok(PgResponse::empty_result(StatementType::REVOKE_PRIVILEGE))
}

#[cfg(test)]
mod tests {
    use piestream_common::catalog::DEFAULT_SUPPER_USER;

    use super::*;
    use crate::test_utils::LocalFrontend;

    #[tokio::test]
    async fn test_grant_privilege() {
        let frontend = LocalFrontend::new(Default::default()).await;
        let session = frontend.session_ref();
        frontend
            .run_sql("CREATE USER user WITH SUPERUSER PASSWORD 'password'")
            .await
            .unwrap();
        frontend
            .run_sql("CREATE USER user1 WITH PASSWORD 'password1'")
            .await
            .unwrap();
        frontend.run_sql("CREATE DATABASE db1").await.unwrap();
        frontend
            .run_sql("GRANT ALL ON DATABASE db1 TO user1 WITH GRANT OPTION GRANTED BY user")
            .await
            .unwrap();

        let database_id = {
            let catalog_reader = session.env().catalog_reader();
            let reader = catalog_reader.read_guard();
            reader.get_database_by_name("db1").unwrap().id()
        };

        {
            let user_reader = session.env().user_info_reader();
            let reader = user_reader.read_guard();
            let user_info = reader.get_user_by_name("user1").unwrap();
            assert_eq!(
                user_info.grant_privileges,
                vec![ProstPrivilege {
                    action_with_opts: vec![
                        ActionWithGrantOption {
                            action: ProstAction::Connect as i32,
                            with_grant_option: true,
                            granted_by: DEFAULT_SUPPER_USER.to_string(),
                        },
                        ActionWithGrantOption {
                            action: ProstAction::Create as i32,
                            with_grant_option: true,
                            granted_by: DEFAULT_SUPPER_USER.to_string(),
                        }
                    ],
                    object: Some(ProstObject::DatabaseId(database_id)),
                }]
            );
        }

        frontend
            .run_sql("REVOKE GRANT OPTION FOR ALL ON DATABASE db1 from user1 GRANTED BY user")
            .await
            .unwrap();
        {
            let user_reader = session.env().user_info_reader();
            let reader = user_reader.read_guard();
            let user_info = reader.get_user_by_name("user1").unwrap();
            assert!(user_info
                .grant_privileges
                .iter()
                .all(|p| p.action_with_opts.iter().all(|ao| !ao.with_grant_option)));
        }

        frontend
            .run_sql("REVOKE ALL ON DATABASE db1 from user1 GRANTED BY user")
            .await
            .unwrap();
        {
            let user_reader = session.env().user_info_reader();
            let reader = user_reader.read_guard();
            let user_info = reader.get_user_by_name("user1").unwrap();
            assert!(user_info.grant_privileges.is_empty());
        }
    }
}
