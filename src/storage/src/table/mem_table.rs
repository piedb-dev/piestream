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
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::ops::RangeBounds;

use piestream_common::array::Row;

#[derive(Clone)]
pub enum RowOp {
    Insert(Row),
    Delete(Row),
    Update((Row, Row)),
}

/// `MemTable` is a buffer for modify operations without encoding
#[derive(Clone)]
pub struct MemTable {
    buffer: BTreeMap<Vec<u8>, RowOp>,
}

pub type MemTableIter<'a> = impl Iterator<Item = (&'a Vec<u8>, &'a RowOp)>;

impl Default for MemTable {
    fn default() -> Self {
        Self::new()
    }
}

impl MemTable {
    pub fn new() -> Self {
        Self {
            buffer: BTreeMap::new(),
        }
    }

    pub fn is_dirty(&self) -> bool {
        !self.buffer.is_empty()
    }

    /// read methods
    pub fn get_row_op(&self, pk: &[u8]) -> Option<&RowOp> {
        self.buffer.get(pk)
    }

    /// write methods
    pub fn insert(&mut self, pk: Vec<u8>, value: Row) {
        let entry = self.buffer.entry(pk);
        match entry {
            Entry::Vacant(e) => {
                e.insert(RowOp::Insert(value));
            }
            Entry::Occupied(mut e) => match e.get_mut() {
                //delete类型改成update,并保留最新值
                x @ RowOp::Delete(_) => {
                    if let RowOp::Delete(ref mut old_value) = x {
                        let old_val = std::mem::take(old_value);
                        e.insert(RowOp::Update((old_val, value)));
                    } else {
                        unreachable!();
                    }
                }

                _ => {
                    panic!(
                        "invalid flush status: double insert {:?} -> {:?}",
                        e.key(),
                        value
                    );
                }
            },
        }
    }

    pub fn delete(&mut self, pk: Vec<u8>, old_value: Row) {
        let entry = self.buffer.entry(pk.clone());
        match entry {
            Entry::Vacant(e) => {
                //实际往状态表插入了一行，标记为delete
                e.insert(RowOp::Delete(old_value));
                println!("delete Vacant pk={:?}", pk.clone());
            }
            Entry::Occupied(mut e) => match e.get_mut() {
                RowOp::Insert(original_value) => {
                    //insert记录真执行删除
                    debug_assert_eq!(original_value, &old_value);
                    e.remove();
                }
                RowOp::Delete(_) => {
                    panic!(
                        "invalid flush status: double delete {:?} -> {:?}",
                        e.key(),
                        old_value
                    );
                }
                RowOp::Update(value) => {
                    let (original_old_value, original_new_value) = std::mem::take(value);
                    debug_assert_eq!(original_new_value, old_value);
                    //设置rowop:Delete状态,删除最原始字段值
                    e.insert(RowOp::Delete(original_old_value));
                }
            },
        }
    }

    pub fn update(&mut self, pk: Vec<u8>, old_value: Row, new_value: Row) {
        let entry = self.buffer.entry(pk);
        match entry {
            Entry::Vacant(e) => {
                e.insert(RowOp::Update((old_value, new_value)));
            }
            Entry::Occupied(mut e) => match e.get_mut() {
                RowOp::Insert(original_value) => {
                    debug_assert_eq!(original_value, &old_value);
                    e.insert(RowOp::Update((old_value, new_value)));
                }
                RowOp::Delete(_) => {
                    //应该是删除状态不让修改，原始值已经删除了，不能再改，是否修改成insert，new_value也是完整数据
                    panic!(
                        "invalid flush status: double delete {:?} -> {:?}",
                        e.key(),
                        old_value
                    );
                }
                RowOp::Update(value) => {
                    let (original_old_value, original_new_value) = std::mem::take(value);
                    debug_assert_eq!(original_new_value, old_value);
                    //更新保存原始字段值
                    e.insert(RowOp::Update((original_old_value, new_value)));
                }
            },
        }
    }

    pub fn into_parts(self) -> BTreeMap<Vec<u8>, RowOp> {
        self.buffer
    }

    pub fn iter<'a, R>(&'a self, key_range: R) -> MemTableIter<'a>
    where
        R: RangeBounds<Vec<u8>> + 'a,
    {
        self.buffer.range(key_range)
    }
}
