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

use std::collections::HashMap;

use itertools::Itertools;
use piestream_hummock_sdk::HummockSSTableId;
use piestream_pb::hummock::level_handler::SstTask;
use piestream_pb::hummock::SstableInfo;

#[derive(Clone, Debug, PartialEq)]
pub struct LevelHandler {
    level: u32,
    compacting_files: HashMap<HummockSSTableId, u64>,
    pending_tasks: Vec<(u64, u64, Vec<HummockSSTableId>)>,
}

impl LevelHandler {
    pub fn new(level: u32) -> Self {
        Self {
            level,
            compacting_files: HashMap::default(),
            pending_tasks: vec![],
        }
    }

    pub fn get_level(&self) -> u32 {
        self.level
    }

    pub fn remove_task(&mut self, target_task_id: u64) {
        for (task_id, _, ssts) in &self.pending_tasks {
            if *task_id == target_task_id {
                for sst in ssts {
                    //删除任务对应的sst文件
                    self.compacting_files.remove(sst);
                }
            }
        }
        //去除掉任务
        self.pending_tasks
            .retain(|(task_id, _, _)| *task_id != target_task_id);
    }

    pub fn is_pending_compact(&self, sst_id: &HummockSSTableId) -> bool {
        self.compacting_files.contains_key(sst_id)
    }

    pub fn add_pending_task(&mut self, task_id: u64, ssts: &[SstableInfo]) {
        let mut table_ids = vec![];
        let mut total_file_size = 0;
        for sst in ssts {
            self.compacting_files.insert(sst.id, task_id);
            total_file_size += sst.file_size;
            table_ids.push(sst.id);
        }
        /*println!("^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^");
        println!("self.compacting_files={:?} ssts={:?}", self.compacting_files.len(), ssts.len());
        println!("self.compacting_files={:?}", self.compacting_files);
        println!("^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^");*/
        //pending_tasks列表中设置task_id对应的total_file_size,table_ids
        self.pending_tasks
            .push((task_id, total_file_size, table_ids));
    }

    //获取pending文件总数
    pub fn get_pending_file_count(&self) -> usize {
        self.compacting_files.len()
    }

    //获取pending文件总大小
    pub fn get_pending_file_size(&self) -> u64 {
        self.pending_tasks
            .iter()
            .map(|(_, total_file_size, _)| *total_file_size)
            .sum::<u64>()
    }

    //获取pending task_id列表
    pub fn pending_tasks_ids(&self) -> Vec<u64> {
        self.pending_tasks
            .iter()
            .map(|(task_id, _, _)| *task_id)
            .collect_vec()
    }
}

impl From<&LevelHandler> for piestream_pb::hummock::LevelHandler {
    fn from(lh: &LevelHandler) -> Self {
        piestream_pb::hummock::LevelHandler {
            level: lh.level,
            tasks: lh
                .pending_tasks
                .iter()
                .map(|(task_id, total_file_size, ssts)| SstTask {
                    task_id: *task_id,
                    ssts: ssts.clone(),
                    total_file_size: *total_file_size,
                })
                .collect_vec(),
        }
    }
}

impl From<&piestream_pb::hummock::LevelHandler> for LevelHandler {
    fn from(lh: &piestream_pb::hummock::LevelHandler) -> Self {
        let mut pending_tasks = vec![];
        let mut compacting_files = HashMap::new();
        for task in &lh.tasks {
            pending_tasks.push((task.task_id, task.total_file_size, task.ssts.clone()));
            for s in &task.ssts {
                compacting_files.insert(*s, task.task_id);
            }
        }
        Self {
            pending_tasks,
            compacting_files,
            level: lh.level,
        }
    }
}
