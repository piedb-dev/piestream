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

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use itertools::Itertools;
use risingwave_common::catalog::TableOption;
use risingwave_hummock_sdk::compaction_group::{StateTableId, StaticCompactionGroupId};
use risingwave_hummock_sdk::CompactionGroupId;
use risingwave_pb::hummock::CompactionConfig;
use tokio::sync::RwLock;

use crate::hummock::compaction::compaction_config::CompactionConfigBuilder;
use crate::hummock::compaction_group::CompactionGroup;
use crate::hummock::error::{Error, Result};
use crate::manager::{MetaSrvEnv, SourceId};
use crate::model::{MetadataModel, TableFragments, ValTransaction, VarTransaction};
use crate::storage::{MetaStore, Transaction};

pub type CompactionGroupManagerRef<S> = Arc<CompactionGroupManager<S>>;

/// `CompactionGroupManager` manages `CompactionGroup`'s members.
///
/// Note that all hummock state store user should register to `CompactionGroupManager`. It includes:
/// - Materialized View via `register_table_fragments`.
/// - Materialized Source via `register_table_fragments`.
/// - Source via `register_source`.
pub struct CompactionGroupManager<S: MetaStore> {
    env: MetaSrvEnv<S>,
    inner: RwLock<CompactionGroupManagerInner>,
}

impl<S: MetaStore> CompactionGroupManager<S> {
    pub async fn new(env: MetaSrvEnv<S>) -> Result<Self> {
        let config = CompactionConfigBuilder::new().build();
        Self::new_with_config(env, config).await
    }

    pub async fn new_with_config(env: MetaSrvEnv<S>, config: CompactionConfig) -> Result<Self> {
        let instance = Self {
            env,
            inner: RwLock::new(Default::default()),
        };
        instance
            .inner
            .write()
            .await
            .init(&config, instance.env.meta_store())
            .await?;
        Ok(instance)
    }

    pub async fn compaction_groups(&self) -> Vec<CompactionGroup> {
        self.inner
            .read()
            .await
            .compaction_groups
            .values()
            .cloned()
            .collect_vec()
    }

    pub async fn compaction_group(&self, id: CompactionGroupId) -> Option<CompactionGroup> {
        self.inner.read().await.compaction_groups.get(&id).cloned()
    }

    /// Registers `table_fragments` to compaction groups.
    pub async fn register_table_fragments(
        &self,
        table_fragments: &TableFragments,
        table_properties: &HashMap<String, String>,
    ) -> Result<Vec<StateTableId>> {
        let table_option = TableOption::build_table_option(table_properties);
        let mut pairs = vec![];
        // materialized_view or materialized_source
        pairs.push((
            table_fragments.table_id().table_id,
            CompactionGroupId::from(StaticCompactionGroupId::MaterializedView),
            table_option,
        ));
        // internal states
        for table_id in table_fragments.internal_table_ids() {
            assert_ne!(table_id, table_fragments.table_id().table_id);
            pairs.push((
                table_id,
                CompactionGroupId::from(StaticCompactionGroupId::StateDefault),
                table_option,
            ));
        }
        self.inner
            .write()
            .await
            .register(&pairs, self.env.meta_store())
            .await
    }

    /// Unregisters `table_fragments` from compaction groups
    pub async fn unregister_table_fragments(&self, table_fragments: &TableFragments) -> Result<()> {
        let table_ids = table_fragments
            .internal_table_ids()
            .into_iter()
            .chain(std::iter::once(table_fragments.table_id().table_id))
            .collect_vec();
        self.inner
            .write()
            .await
            .unregister(&table_ids, self.env.meta_store())
            .await
    }

    pub async fn register_source(
        &self,
        source_id: u32,
        table_properties: &HashMap<String, String>,
    ) -> Result<Vec<StateTableId>> {
        let table_option = TableOption::build_table_option(table_properties);
        self.inner
            .write()
            .await
            .register(
                &[(
                    source_id,
                    StaticCompactionGroupId::StateDefault.into(),
                    table_option,
                )],
                self.env.meta_store(),
            )
            .await
    }

    pub async fn unregister_source(&self, source_id: u32) -> Result<()> {
        self.inner
            .write()
            .await
            .unregister(&[source_id], self.env.meta_store())
            .await
    }

    /// Unregisters stale members
    ///
    /// Valid members includes:
    /// - MV fragments.
    /// - Source.
    /// - Source in fragments. It's possible a source is dropped while associated fragments still
    ///   exist. See `SourceManager::drop_source`.
    pub async fn purge_stale_members(
        &self,
        table_fragments_list: &[TableFragments],
        source_ids: &[SourceId],
        source_ids_in_fragments: &[SourceId],
    ) -> Result<()> {
        let mut guard = self.inner.write().await;
        let registered_members = guard
            .compaction_groups
            .values()
            .flat_map(|cg| cg.member_table_ids.iter())
            .cloned()
            .collect_vec();
        let valid_ids = table_fragments_list
            .iter()
            .flat_map(|table_fragments| {
                table_fragments
                    .internal_table_ids()
                    .iter()
                    .cloned()
                    .chain(std::iter::once(table_fragments.table_id().table_id))
                    .collect_vec()
            })
            .chain(source_ids.iter().cloned())
            .chain(source_ids_in_fragments.iter().cloned())
            .dedup()
            .collect_vec();
        let to_unregister = registered_members
            .into_iter()
            .filter(|table_id| !valid_ids.contains(table_id))
            .collect_vec();
        guard
            .unregister(&to_unregister, self.env.meta_store())
            .await
    }

    pub async fn internal_table_ids_by_compaction_group_id(
        &self,
        compaction_group_id: u64,
    ) -> Result<HashSet<StateTableId>> {
        let inner = self.inner.read().await;
        let table_id_set = inner.table_ids_by_compaction_group_id(compaction_group_id)?;
        Ok(table_id_set)
    }

    pub async fn register_table_ids(
        &self,
        pairs: &[(StateTableId, CompactionGroupId, TableOption)],
    ) -> Result<Vec<StateTableId>> {
        self.inner
            .write()
            .await
            .register(pairs, self.env.meta_store())
            .await
    }

    pub async fn unregister_table_ids(&self, table_ids: &[StateTableId]) -> Result<()> {
        self.inner
            .write()
            .await
            .unregister(table_ids, self.env.meta_store())
            .await
    }

    pub async fn get_table_option(
        &self,
        id: CompactionGroupId,
        table_id: u32,
    ) -> Result<TableOption> {
        let inner = self.inner.read().await;
        inner.table_option_by_table_id(id, table_id)
    }
}

#[derive(Default)]
struct CompactionGroupManagerInner {
    compaction_groups: BTreeMap<CompactionGroupId, CompactionGroup>,
    index: BTreeMap<StateTableId, CompactionGroupId>,
}

impl CompactionGroupManagerInner {
    async fn init<S: MetaStore>(
        &mut self,
        config: &CompactionConfig,
        meta_store: &S,
    ) -> Result<()> {
        let loaded_compaction_groups: BTreeMap<CompactionGroupId, CompactionGroup> =
            CompactionGroup::list(meta_store)
                .await?
                .into_iter()
                .map(|cg| (cg.group_id(), cg))
                .collect();
        if !loaded_compaction_groups.is_empty() {
            self.compaction_groups = loaded_compaction_groups;
        } else {
            let compaction_groups = &mut self.compaction_groups;
            let mut new_compaction_groups = VarTransaction::new(compaction_groups);
            let static_compaction_groups = vec![
                CompactionGroup::new(StaticCompactionGroupId::StateDefault.into(), config.clone()),
                CompactionGroup::new(
                    StaticCompactionGroupId::MaterializedView.into(),
                    config.clone(),
                ),
            ];
            for static_compaction_group in static_compaction_groups {
                new_compaction_groups
                    .insert(static_compaction_group.group_id(), static_compaction_group);
            }
            let mut trx = Transaction::default();
            new_compaction_groups.apply_to_txn(&mut trx)?;
            meta_store.txn(trx).await?;
            new_compaction_groups.commit();
        }

        // Build in-memory index
        for (id, compaction_group) in &self.compaction_groups {
            for member in &compaction_group.member_table_ids {
                assert!(self.index.insert(*member, *id).is_none());
            }
        }

        Ok(())
    }

    async fn register<S: MetaStore>(
        &mut self,
        pairs: &[(StateTableId, CompactionGroupId, TableOption)],
        meta_store: &S,
    ) -> Result<Vec<StateTableId>> {
        let mut compaction_groups = VarTransaction::new(&mut self.compaction_groups);
        for (table_id, compaction_group_id, table_option) in pairs {
            let compaction_group = compaction_groups
                .get_mut(compaction_group_id)
                .ok_or(Error::InvalidCompactionGroup(*compaction_group_id))?;
            compaction_group.member_table_ids.insert(*table_id);
            compaction_group
                .table_id_to_options
                .insert(*table_id, *table_option);
        }
        let mut trx = Transaction::default();
        compaction_groups.apply_to_txn(&mut trx)?;
        meta_store.txn(trx).await?;
        compaction_groups.commit();

        // Update in-memory index
        for (table_id, compaction_group_id, _) in pairs {
            self.index.insert(*table_id, *compaction_group_id);
        }
        Ok(pairs.iter().map(|(table_id, ..)| *table_id).collect_vec())
    }

    async fn unregister<S: MetaStore>(
        &mut self,
        table_ids: &[StateTableId],
        meta_store: &S,
    ) -> Result<()> {
        let mut compaction_groups = VarTransaction::new(&mut self.compaction_groups);
        for table_id in table_ids {
            let compaction_group_id = self
                .index
                .get(table_id)
                .cloned()
                .ok_or(Error::InvalidCompactionGroupMember(*table_id))?;
            let compaction_group = compaction_groups
                .get_mut(&compaction_group_id)
                .ok_or(Error::InvalidCompactionGroup(compaction_group_id))?;
            compaction_group.member_table_ids.remove(table_id);
            compaction_group.table_id_to_options.remove(table_id);
        }
        let mut trx = Transaction::default();
        compaction_groups.apply_to_txn(&mut trx)?;
        meta_store.txn(trx).await?;
        compaction_groups.commit();

        // Update in-memory index
        for table_id in table_ids {
            self.index.remove(table_id);
        }
        Ok(())
    }

    fn compaction_group(&self, compaction_group_id: u64) -> Result<CompactionGroup> {
        match self.compaction_groups.get(&compaction_group_id) {
            Some(compaction_group) => Ok(compaction_group.clone()),

            None => Err(Error::InvalidCompactionGroup(compaction_group_id)),
        }
    }

    pub fn table_ids_by_compaction_group_id(
        &self,
        compaction_group_id: u64,
    ) -> Result<HashSet<StateTableId>> {
        let compaction_group = self.compaction_group(compaction_group_id)?;
        Ok(compaction_group.member_table_ids)
    }

    pub fn table_option_by_table_id(
        &self,
        compaction_group_id: u64,
        table_id: u32,
    ) -> Result<TableOption> {
        let compaction_group = self.compaction_group(compaction_group_id)?;
        match compaction_group.table_id_to_options().get(&table_id) {
            Some(table_option) => Ok(*table_option),

            None => Ok(TableOption::default()),
        }
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;
    use std::ops::Deref;

    use risingwave_common::catalog::{TableId, TableOption};
    use risingwave_hummock_sdk::compaction_group::StaticCompactionGroupId;

    use crate::hummock::compaction_group::manager::{
        CompactionGroupManager, CompactionGroupManagerInner,
    };
    use crate::hummock::test_utils::setup_compute_env;
    use crate::model::TableFragments;

    #[tokio::test]
    async fn test_inner() {
        let (env, ..) = setup_compute_env(8080).await;
        let compaction_group_manager = CompactionGroupManager::new(env.clone()).await.unwrap();
        let inner = compaction_group_manager.inner;

        let registered_number = |inner: &CompactionGroupManagerInner| {
            inner
                .compaction_groups
                .iter()
                .map(|(_, cg)| cg.member_table_ids.len())
                .sum::<usize>()
        };

        let table_option_number = |inner: &CompactionGroupManagerInner| {
            inner
                .compaction_groups
                .iter()
                .map(|(_, cg)| cg.table_id_to_options().len())
                .sum::<usize>()
        };

        assert!(inner.read().await.index.is_empty());
        assert_eq!(registered_number(inner.read().await.deref()), 0);

        let table_properties = HashMap::from([(String::from("ttl"), String::from("300"))]);
        let table_option = TableOption::build_table_option(&table_properties);

        // Test register
        inner
            .write()
            .await
            .register(
                &[(
                    1u32,
                    StaticCompactionGroupId::StateDefault.into(),
                    table_option,
                )],
                env.meta_store(),
            )
            .await
            .unwrap();
        inner
            .write()
            .await
            .register(
                &[(
                    2u32,
                    StaticCompactionGroupId::MaterializedView.into(),
                    table_option,
                )],
                env.meta_store(),
            )
            .await
            .unwrap();
        assert_eq!(inner.read().await.index.len(), 2);
        assert_eq!(registered_number(inner.read().await.deref()), 2);

        // Test init
        let compaction_group_manager = CompactionGroupManager::new(env.clone()).await.unwrap();
        let inner = compaction_group_manager.inner;
        assert_eq!(inner.read().await.index.len(), 2);
        assert_eq!(registered_number(inner.read().await.deref()), 2);
        assert_eq!(table_option_number(inner.read().await.deref()), 2);

        // Test unregister
        inner
            .write()
            .await
            .unregister(&[2u32], env.meta_store())
            .await
            .unwrap();
        assert_eq!(inner.read().await.index.len(), 1);
        assert_eq!(registered_number(inner.read().await.deref()), 1);
        assert_eq!(table_option_number(inner.read().await.deref()), 1);

        // Test init
        let compaction_group_manager = CompactionGroupManager::new(env.clone()).await.unwrap();
        let inner = compaction_group_manager.inner;
        assert_eq!(inner.read().await.index.len(), 1);
        assert_eq!(registered_number(inner.read().await.deref()), 1);
        assert_eq!(table_option_number(inner.read().await.deref()), 1);

        // Test table_option_by_table_id
        {
            let table_option = inner
                .read()
                .await
                .table_option_by_table_id(StaticCompactionGroupId::StateDefault.into(), 1u32)
                .unwrap();
            assert_eq!(300, table_option.ttl.unwrap());
        }

        {
            // unregistered table_id
            let table_option_default = inner
                .read()
                .await
                .table_option_by_table_id(StaticCompactionGroupId::StateDefault.into(), 2u32);
            assert!(table_option_default.is_ok());
            assert_eq!(None, table_option_default.unwrap().ttl);
        }
    }

    #[tokio::test]
    async fn test_manager() {
        let (env, ..) = setup_compute_env(8080).await;
        let compaction_group_manager = CompactionGroupManager::new(env.clone()).await.unwrap();
        let table_fragment_1 =
            TableFragments::new(TableId::new(10), Default::default(), [11, 12, 13].into());
        let table_fragment_2 =
            TableFragments::new(TableId::new(20), Default::default(), [21, 22, 23].into());
        let source_1 = 100;
        let source_2 = 200;
        let source_3 = 300;

        // Test register_table_fragments
        let registered_number = || async {
            compaction_group_manager
                .compaction_groups()
                .await
                .iter()
                .map(|cg| cg.member_table_ids.len())
                .sum::<usize>()
        };
        assert_eq!(registered_number().await, 0);
        let table_properties = HashMap::from([(String::from("ttl"), String::from("300"))]);

        compaction_group_manager
            .register_table_fragments(&table_fragment_1, &table_properties)
            .await
            .unwrap();
        assert_eq!(registered_number().await, 4);
        compaction_group_manager
            .register_table_fragments(&table_fragment_2, &table_properties)
            .await
            .unwrap();
        assert_eq!(registered_number().await, 8);

        // Test unregister_table_fragments
        compaction_group_manager
            .unregister_table_fragments(&table_fragment_1)
            .await
            .unwrap();
        assert_eq!(registered_number().await, 4);

        // Test purge_stale_members: table fragments
        compaction_group_manager
            .purge_stale_members(&[table_fragment_2], &[], &[])
            .await
            .unwrap();
        assert_eq!(registered_number().await, 4);
        compaction_group_manager
            .purge_stale_members(&[], &[], &[])
            .await
            .unwrap();
        assert_eq!(registered_number().await, 0);

        // Test register_source
        compaction_group_manager
            .register_source(source_1, &table_properties)
            .await
            .unwrap();
        assert_eq!(registered_number().await, 1);
        compaction_group_manager
            .register_source(source_2, &table_properties)
            .await
            .unwrap();
        assert_eq!(registered_number().await, 2);
        compaction_group_manager
            .register_source(source_2, &table_properties)
            .await
            .unwrap();
        assert_eq!(registered_number().await, 2);
        compaction_group_manager
            .register_source(source_3, &table_properties)
            .await
            .unwrap();
        assert_eq!(registered_number().await, 3);

        // Test unregister_source
        compaction_group_manager
            .unregister_source(source_2)
            .await
            .unwrap();
        assert_eq!(registered_number().await, 2);

        // Test purge_stale_members: source
        compaction_group_manager
            .purge_stale_members(&[], &[source_3], &[])
            .await
            .unwrap();
        assert_eq!(registered_number().await, 1);
    }
}
