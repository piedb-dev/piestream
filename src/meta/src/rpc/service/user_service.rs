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

use piestream_common::error::{tonic_err, Result as RwResult};
use piestream_pb::user::grant_privilege::Object;
use piestream_pb::user::user_service_server::UserService;
use piestream_pb::user::{
    CreateUserRequest, CreateUserResponse, DropUserRequest, DropUserResponse, GrantPrivilege,
    GrantPrivilegeRequest, GrantPrivilegeResponse, RevokePrivilegeRequest, RevokePrivilegeResponse,
};
use tonic::{Request, Response, Status};

use crate::manager::{CatalogManagerRef, UserInfoManagerRef};
use crate::storage::MetaStore;

// TODO: Change user manager as a part of the catalog manager, to ensure that operations on Catalog
// and User are transactional.
pub struct UserServiceImpl<S: MetaStore> {
    catalog_manager: CatalogManagerRef<S>,
    user_manager: UserInfoManagerRef<S>,
}

impl<S> UserServiceImpl<S>
where
    S: MetaStore,
{
    pub fn new(catalog_manager: CatalogManagerRef<S>, user_manager: UserInfoManagerRef<S>) -> Self {
        Self {
            catalog_manager,
            user_manager,
        }
    }

    /// Expands `GrantPrivilege` with object `GrantAllTables` or `GrantAllSources` to specific
    /// tables and sources, and set `with_grant_option` inside when grant privilege to a user.
    async fn expand_privilege(
        &self,
        privileges: &[GrantPrivilege],
        with_grant_option: Option<bool>,
    ) -> RwResult<Vec<GrantPrivilege>> {
        let mut expanded_privileges = Vec::new();
        for privilege in privileges {
            if let Some(Object::AllTablesSchemaId(schema_id)) = &privilege.object {
                let tables = self.catalog_manager.list_tables(*schema_id).await?;
                for table_id in tables {
                    let mut privilege = privilege.clone();
                    privilege.object = Some(Object::TableId(table_id));
                    if let Some(true) = with_grant_option {
                        privilege
                            .action_with_opts
                            .iter_mut()
                            .for_each(|p| p.with_grant_option = true);
                    }
                    expanded_privileges.push(privilege);
                }
            } else if let Some(Object::AllSourcesSchemaId(source_id)) = &privilege.object {
                let sources = self.catalog_manager.list_sources(*source_id).await?;
                for source_id in sources {
                    let mut privilege = privilege.clone();
                    privilege.object = Some(Object::SourceId(source_id));
                    if let Some(with_grant_option) = with_grant_option {
                        privilege.action_with_opts.iter_mut().for_each(|p| {
                            p.with_grant_option = with_grant_option;
                        });
                    }
                    expanded_privileges.push(privilege);
                }
            } else {
                let mut privilege = privilege.clone();
                if let Some(with_grant_option) = with_grant_option {
                    privilege.action_with_opts.iter_mut().for_each(|p| {
                        p.with_grant_option = with_grant_option;
                    });
                }
                expanded_privileges.push(privilege);
            }
        }

        Ok(expanded_privileges)
    }
}

#[async_trait::async_trait]
impl<S: MetaStore> UserService for UserServiceImpl<S> {
    #[cfg_attr(coverage, no_coverage)]
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        let req = request.into_inner();
        let user = req.get_user().map_err(tonic_err)?;
        let version = self
            .user_manager
            .create_user(user)
            .await
            .map_err(tonic_err)?;

        Ok(Response::new(CreateUserResponse {
            status: None,
            version,
        }))
    }

    #[cfg_attr(coverage, no_coverage)]
    async fn drop_user(
        &self,
        request: Request<DropUserRequest>,
    ) -> Result<Response<DropUserResponse>, Status> {
        let req = request.into_inner();
        let user_name = req.name;
        let version = self
            .user_manager
            .drop_user(&user_name)
            .await
            .map_err(tonic_err)?;

        Ok(Response::new(DropUserResponse {
            status: None,
            version,
        }))
    }

    #[cfg_attr(coverage, no_coverage)]
    async fn grant_privilege(
        &self,
        request: Request<GrantPrivilegeRequest>,
    ) -> Result<Response<GrantPrivilegeResponse>, Status> {
        let req = request.into_inner();
        let new_privileges = self
            .expand_privilege(req.get_privileges(), Some(req.with_grant_option))
            .await
            .map_err(tonic_err)?;
        let version = self
            .user_manager
            .grant_privilege(&req.users, &new_privileges, req.granted_by)
            .await
            .map_err(tonic_err)?;

        Ok(Response::new(GrantPrivilegeResponse {
            status: None,
            version,
        }))
    }

    #[cfg_attr(coverage, no_coverage)]
    async fn revoke_privilege(
        &self,
        request: Request<RevokePrivilegeRequest>,
    ) -> Result<Response<RevokePrivilegeResponse>, Status> {
        let req = request.into_inner();
        let privileges = self
            .expand_privilege(req.get_privileges(), None)
            .await
            .map_err(tonic_err)?;
        let revoke_grant_option = req.revoke_grant_option;
        let version = self
            .user_manager
            .revoke_privilege(
                &req.users,
                &privileges,
                req.granted_by,
                req.revoke_by,
                revoke_grant_option,
                req.cascade,
            )
            .await
            .map_err(tonic_err)?;

        Ok(Response::new(RevokePrivilegeResponse {
            status: None,
            version,
        }))
    }
}
