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

use std::cmp::Ordering;

use itertools::Itertools;
use piestream_hummock_sdk::key::user_key;
use piestream_hummock_sdk::key_range::KeyRangeCommon;
use piestream_pb::hummock::{KeyRange, SstableInfo};

pub trait OverlapInfo {
    //检查相交
    fn check_overlap(&self, a: &SstableInfo) -> bool;
    //返回相交文件列表
    fn check_multiple_overlap(&self, others: &[SstableInfo]) -> Vec<SstableInfo>;
    //更新
    fn update(&mut self, table: &SstableInfo);
}

pub trait OverlapStrategy: Send + Sync {
    //检查相交
    fn check_overlap(&self, a: &SstableInfo, b: &SstableInfo) -> bool;
    //检查多sst文件相交，返回文件列表
    fn check_base_level_overlap(
        &self,
        tables: &[SstableInfo],
        others: &[SstableInfo],
    ) -> Vec<SstableInfo> {
        let mut info = self.create_overlap_info();
        for table in tables {
            info.update(table);
        }
        info.check_multiple_overlap(others)
    }
    //减少相交获取文件列表
    fn check_overlap_with_tables(
        &self,
        tables: &[SstableInfo],
        others: &[SstableInfo],
    ) -> Vec<SstableInfo> {
        if tables.is_empty() || others.is_empty() {
            return vec![];
        }
        let mut info = self.create_overlap_info();
        for table in tables {
            info.update(table);
        }
        others
            .iter()
            .filter(|table| info.check_overlap(*table))
            .cloned()
            .collect_vec()
    }

    //创建OverlapStrategy对象
    fn create_overlap_info(&self) -> Box<dyn OverlapInfo>;
}

#[derive(Default)]
pub struct RangeOverlapInfo {
    target_range: Option<KeyRange>,
}

impl OverlapInfo for RangeOverlapInfo {
    //通过key_range检查是否覆盖
    fn check_overlap(&self, a: &SstableInfo) -> bool {
        match self.target_range.as_ref() {
            Some(range) => check_table_overlap(range, a),
            None => false,
        }
    }

    fn check_multiple_overlap(&self, others: &[SstableInfo]) -> Vec<SstableInfo> {
        match self.target_range.as_ref() {
            //两种场景两个集合不会重叠：
            //1.others列表所有SstableInfo.left都大于key_range.right
            //2.others列表所有SstableInfo.right都小于key_range.left
            Some(key_range) => {
                let mut tables = vec![];
                //other[i].right<key_range.left位置
                let overlap_begin = others.partition_point(|table_status| {
                    user_key(&table_status.key_range.as_ref().unwrap().right)
                        < user_key(&key_range.left)
                });
                //others所有right都小于key_range.left,不会有重叠
                if overlap_begin >= others.len() {
                    return vec![];
                }
                for table in &others[overlap_begin..] {
                    //当other.left>key_range.right退出
                    if user_key(&table.key_range.as_ref().unwrap().left)
                        > user_key(&key_range.right)
                    {
                        break;
                    }
                     //other[i].left<=key_range.right位置
                    tables.push(table.clone());
                }
                tables
            }
            None => vec![],
        }
    }

    fn update(&mut self, table: &SstableInfo) {
        let other = table.key_range.as_ref().unwrap();
        if let Some(range) = self.target_range.as_mut() {
            //扩张rang范围
            range.full_key_extend(other);
            return;
        }
        self.target_range = Some(other.clone());
    }
}

#[derive(Default)]
pub struct HashOverlapInfo {
    infos: Vec<SstableInfo>,
}

impl OverlapInfo for HashOverlapInfo {
    fn check_overlap(&self, a: &SstableInfo) -> bool {
        for info in &self.infos {
            //通过table_ids属性检查是否重复文件
            if check_key_vnode_overlap(info, a) {
                return true;
            }
        }
        false
    }

    //获取重复文件列表
    fn check_multiple_overlap(&self, others: &[SstableInfo]) -> Vec<SstableInfo> {
        others
            .iter()
            .filter(|table| self.check_overlap(*table))
            .cloned()
            .collect_vec()
    }

    //加入列表
    fn update(&mut self, table: &SstableInfo) {
        //加入table
        self.infos.push(table.clone());
    }
}

#[derive(Default)]
pub struct RangeOverlapStrategy {}

impl OverlapStrategy for RangeOverlapStrategy {
    //检查是否相交
    fn check_overlap(&self, a: &SstableInfo, b: &SstableInfo) -> bool {
        let key_range = a.key_range.as_ref().unwrap();
        check_table_overlap(key_range, b)
    }

    fn create_overlap_info(&self) -> Box<dyn OverlapInfo> {
        //创建Info对象
        Box::new(RangeOverlapInfo::default())
    }
}

//检查table重叠
fn check_table_overlap(key_range: &KeyRange, table: &SstableInfo) -> bool {
    let other = table.key_range.as_ref().unwrap();
    key_range.full_key_overlap(other)
}

/// check whether 2 SSTs may have same key by key range and vnode bitmaps in table info
/// 通过table_ids检查info和table是否有重复文件
fn check_key_vnode_overlap(info: &SstableInfo, table: &SstableInfo) -> bool {
    if !info
        .key_range
        .as_ref()
        .unwrap()
        .full_key_overlap(table.key_range.as_ref().unwrap())
    {
        //没有相互覆盖
        return false;
    }
    let text_len = info.get_table_ids().len();
    let other_len = table.get_table_ids().len();
    if text_len == 0 || other_len == 0 {
        return true;
    }
    let (mut i, mut j) = (0, 0);
    while i < text_len && j < other_len {
        let x = &info.get_table_ids()[i];
        let y = &table.get_table_ids()[j];
        match x.cmp(y) {
            Ordering::Less => {
                i += 1;
            }
            Ordering::Greater => {
                j += 1;
            }
            Ordering::Equal => {
                return true;
                // i += 1;
                // j += 1;
            }
        }
    }
    false
}

#[derive(Default)]
pub struct HashStrategy {}

impl OverlapStrategy for HashStrategy {
    fn check_overlap(&self, a: &SstableInfo, b: &SstableInfo) -> bool {
        check_key_vnode_overlap(a, b)
    }

    fn create_overlap_info(&self) -> Box<dyn OverlapInfo> {
        Box::new(HashOverlapInfo::default())
    }
}
