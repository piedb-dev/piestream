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

use std::cmp::Ordering;
use std::ops::Range;

use bytes::{Buf, BufMut, BytesMut};
use piestream_hummock_sdk::VersionedComparator;
use piestream_common::types::{DataType};

use super::DEFAULT_DATA_TYPE_BIT_SIZE;
use crate::hummock::BlockHolder;
use crate::hummock::value::{HummockValue,VALUE_PUT, VALUE_DELETE};


/// [`BlockIterator`] is used to read kv pairs in a block.
pub struct BlockIterator {
    /// Block that iterates on.
    block: BlockHolder,
    /// Current restart point index.
    restart_point_index: usize,
    /// Current offset.
    offset: usize,
    /// Current key.
    key: BytesMut,
    /// Current value.
    value_range: Range<usize>,
    /// Current entry len.
    entry_len: usize,
    index: usize,
}

impl BlockIterator {
    pub fn new(block: BlockHolder) -> Self {
        Self {
            block,
            offset: usize::MAX,
            restart_point_index: usize::MAX,
            key: BytesMut::default(),
            value_range: 0..0,
            entry_len: 0,
            index: 0,
        }
    }

    pub fn next(&mut self) {
        assert!(self.is_valid());
        self.index+=1;
        //self.next_inner();
    }

    pub fn prev(&mut self) {
        assert!(self.is_valid());
        if self.index>0{
            self.index-=1;
        }else{
            self.index=self.block.entry_count();
        }
        //self.prev_inner();
    }

    pub fn key(&self) -> &[u8] {
        assert!(self.is_valid());
        let start=self.index*self.block.key_len as usize;
        let end=(((self.index+1)*self.block.key_len as usize)-3) as usize ;
        &self.block.keys[start..end]
        //&self.key[..]
    }

    //pub fn value(&self) -> &[u8] {
    pub fn value(&self) -> Box<Vec<u8>> {
        assert!(self.is_valid());
     
        let start=(self.index+1)*self.block.key_len as usize-3;
        let end=start+3 ;
        
        let mut buf=&self.block.keys[start..end];
        //let mut buf=Bytes::from(&self.block.keys[start..end]);
        let row_state=buf.get_u8();
        if row_state==VALUE_DELETE{
            return Box::new(vec![VALUE_DELETE]);
        }

        let put_entry_count=buf.get_u16_le() as usize;
        assert!(put_entry_count<self.block.vaild_entry_count as usize);
    
        let mut value =Vec::new();
        value.put_u8(VALUE_PUT);
        let col_state_bytes_number=(put_entry_count / DEFAULT_DATA_TYPE_BIT_SIZE as usize) * 4;

        let  mut variable_column_count=0_usize;
        for idx in 0..self.block.column_count as usize {
   
            let state_offset=idx * col_state_bytes_number + self.index / DEFAULT_DATA_TYPE_BIT_SIZE as usize ;
            let mut col_state=&self.block.states[..];
            col_state.advance(state_offset );
            //get current column value state
            let v=(col_state.get_u32_le()>>(self.index % DEFAULT_DATA_TYPE_BIT_SIZE as usize)) & 0x1;
            let mut column=&self.block.vec_text[idx][..];
            //is Some
            if v>0 {
                value.put_u8(1u8);
                //variable column
                if self.block.data_type_values[idx].1==0{
                    column.advance(self.index * 4  );
                    let  variable_column=&self.block.vec_text[variable_column_count+self.block.column_count as usize][..];
                    let offset=column.get_u32_le() as usize;
                    let mut next_offset=0;
                    if (put_entry_count+1) < self.block.vaild_entry_count as usize{
                        next_offset+=column.get_u32_le() as usize;
                    }else{
                        next_offset=variable_column.len();
                        assert!(offset<=next_offset);
                    }
                    let text=&variable_column[offset..next_offset];
                    value.extend_from_slice(text);
                    variable_column_count+=1;
                    
                    /*let index=self.get_offset(idx*put_entry_count, 4);
                    column.advance(index);
                    let offset=column.get_u32_le() as usize;
                    let mut next_offset=0;
                    if (put_entry_count+1) < self.block.vaild_entry_count as usize{
                        next_offset+=column.get_u32_le() as usize;
                    }else{
                        next_offset+=column.len();
                    }
                    let  variable_column=&self.block.vec_text[variable_column_count+self.block.column_count as usize][..];
                    let text=&variable_column[offset..next_offset];
                    value.extend_from_slice(text);
                    variable_column_count+=1;*/
                }else{
                    //fixed column
                    let offset=self.index * self.block.data_type_values[idx].1 ;
                    column.advance(offset);
                    let text=&column[offset..offset+self.block.data_type_values[idx].1];
                    value.extend_from_slice(text);
                }
            }else{
                //is None
                value.put_u8(0u8);
            }
        }
        Box::new(value)
    }

    pub fn is_valid(&self) -> bool {
        //self.offset < self.block.len()
        self.index<self.block.entry_count()
    }

    pub fn seek_to_first(&mut self) {
        self.index=0;
    }

    pub fn seek_to_last(&mut self) {
        self.index=self.block.entry_count()-1;
        //println!("self.index={:?}", self.index);
        //self.seek_restart_point_by_index(self.block.restart_point_len() - 1);
        //self.next_until_prev_offset(self.block.len());
    }

    pub fn seek(&mut self, key: &[u8]) {
        let mut left = 0_i32;
        let mut right = (self.block.entry_count() - 1) as i32;
        let mut is_hit=false;
        while left>=0 && right>=0 && left <= right {
            let middle = (left + right)/2;
            let start=middle as usize *self.block.key_len as usize;
            let end=start+self.block.key_len as usize -3 ;
            self.index=middle as usize;
            let ordering=VersionedComparator::compare_key(&self.block.keys[start..end], key);
            if ordering==Ordering::Equal{
                is_hit=true;
                break;
            }else if ordering==Ordering::Less{
                left = middle + 1;
            }else{
                right = middle - 1;
            }
        }

        //search key greater all key of block 
        if is_hit==false &&  self.index+1==self.block.entry_count() {
            self.index=self.block.entry_count();
        }
        /*for idx in 0..self.block.entry_count(){
            let start=idx*self.block.key_len as usize;
            let end=start+self.block.key_len as usize -3 ;
            self.index=idx;
            if VersionedComparator::compare_key(&self.block.keys[start..end], key)==Ordering::Less{
                continue;
            }else{
                break;
            }
        }*/
    }

    pub fn seek_le(&mut self, key: &[u8]) {
        self.seek(key);
        if !self.is_valid(){
            self.seek_to_last();
        }
        while self.is_valid(){ 
            let start=self.index*self.block.key_len as usize;
            let end=start+self.block.key_len as usize -3 ;
            if VersionedComparator::compare_key(&self.block.keys[start..end], key)==Ordering::Greater{
                if self.index==0 {
                    self.index=self.block.entry_count();
                }else{
                    self.index-=1;
                }
            }else{
                break;
            }
        }
        
        /*while self.index>0 {
            let start=self.index*self.block.key_len as usize;
            let end=start+self.block.key_len as usize -3 ;
            if VersionedComparator::compare_key(&self.block.keys[start..end], key)==Ordering::Greater{
                self.index-=1;
                continue;
            }else{
                break;
            }
        }*/
    }
}

/*impl BlockIterator {
    /// Invalidates current state after reaching a invalid state.
    fn invalidate(&mut self) {
        self.offset = self.block.len();
        self.restart_point_index = self.block.restart_point_len();
        self.key.clear();
        self.value_range = 0..0;
        self.entry_len = 0;
    }

    /// Moves to the next entry.
    ///
    /// Note: Ensures that the current state is valid.
    fn next_inner(&mut self) {
        let offset = self.offset + self.entry_len;
        if offset >= self.block.len() {
            self.invalidate();
            return;
        }
        let prefix = self.decode_prefix_at(offset);
        self.key.truncate(prefix.overlap_len());
        self.key
            .extend_from_slice(&self.block.data()[prefix.diff_key_range()]);
        self.value_range = prefix.value_range();
        self.offset = offset;
        self.entry_len = prefix.entry_len();
        if self.restart_point_index + 1 < self.block.restart_point_len()
            && self.offset >= self.block.restart_point(self.restart_point_index + 1) as usize
        {
            self.restart_point_index += 1;
        }
    }

    /// Moves forward until reaching the first that equals or larger than the given `key`.
    fn next_until_key(&mut self, key: &[u8]) {
        while self.is_valid()
            && VersionedComparator::compare_key(&self.key[..], key) == Ordering::Less
        {
            self.next_inner();
        }
    }

    /// Moves backward until reaching the first key that equals or smaller than the given `key`.
    fn prev_until_key(&mut self, key: &[u8]) {
        while self.is_valid()
            && VersionedComparator::compare_key(&self.key[..], key) == Ordering::Greater
        {
            self.prev_inner();
        }
    }

    /// Moves forward until the position reaches the previous position of the given `next_offset` or
    /// the last valid position if exists.
    fn next_until_prev_offset(&mut self, offset: usize) {
        while self.offset + self.entry_len < std::cmp::min(self.block.len(), offset) {
            self.next_inner();
        }
    }

    /// Moves to the previous entry.
    ///
    /// Note: Ensure that the current state is valid.
    fn prev_inner(&mut self) {
        if self.offset == 0 {
            self.invalidate();
            return;
        }
        if self.block.restart_point(self.restart_point_index) as usize == self.offset {
            self.restart_point_index -= 1;
        }
        let origin_offset = self.offset;
        self.seek_restart_point_by_index(self.restart_point_index);
        self.next_until_prev_offset(origin_offset);
    }

    /// Decodes [`KeyPrefix`] at given offset.
    fn decode_prefix_at(&self, offset: usize) -> KeyPrefix {
        KeyPrefix::decode(&mut &self.block.data()[offset..], offset)
    }

    /// Searches the restart point index that the given `key` belongs to.
    fn search_restart_point_index_by_key(&self, key: &[u8]) -> usize {
        // Find the largest restart point that restart key equals or less than the given key.
        self.block
            .search_restart_partition_point(|&probe| {
                let prefix = self.decode_prefix_at(probe as usize);
                let probe_key = &self.block.data()[prefix.diff_key_range()];
                match VersionedComparator::compare_key(probe_key, key) {
                    Ordering::Less | Ordering::Equal => true,
                    Ordering::Greater => false,
                }
            })
            .saturating_sub(1) // Prevent from underflowing when given is smaller than the first.
    }

    /// Seeks to the restart point that the given `key` belongs to.
    fn seek_restart_point_by_key(&mut self, key: &[u8]) {
        let index = self.search_restart_point_index_by_key(key);
        self.seek_restart_point_by_index(index)
    }

    /// Seeks to the restart point by given restart point index.
    fn seek_restart_point_by_index(&mut self, index: usize) {
        let offset = self.block.restart_point(index) as usize;
        let prefix = self.decode_prefix_at(offset);
        self.key = BytesMut::from(&self.block.data()[prefix.diff_key_range()]);
        self.value_range = prefix.value_range();
        self.offset = offset;
        self.entry_len = prefix.entry_len();
        self.restart_point_index = index;
    }
}*/

#[cfg(test)]
mod tests {
    use bytes::{BufMut, Bytes};

    use super::*;
    use crate::hummock::{Block, BlockBuilder, BlockBuilderOptions};

    fn get_hummock_value( value: &[u8])->Bytes{
        let mut v=vec![1_u8];
        v.extend_from_slice(&(value.len() as u32).to_ne_bytes());
        v.extend_from_slice(value);
        let  mut raw_value=BytesMut::new();
        HummockValue::put(&v[..]).encode(&mut raw_value);
        raw_value.freeze()
    }

    fn build_iterator_for_test() -> BlockIterator {
        let options = BlockBuilderOptions::default();
        let mut builder = BlockBuilder::new(options);
        builder.set_row_deserializer(vec![DataType::Varchar]);
        builder.add(&full_key(b"k01", 1), &get_hummock_value(b"v01")[..]);
        builder.add(&full_key(b"k02", 2), &get_hummock_value(b"v02")[..]);
        builder.add(&full_key(b"k04", 4), &get_hummock_value(b"v04")[..]);
        builder.add(&full_key(b"k05", 5), &get_hummock_value(b"v05")[..]);
        let buf = builder.build().to_vec();
        let capacity = builder.uncompressed_block_size();
        BlockIterator::new(BlockHolder::from_owned_block(Box::new(
            Block::decode(buf.into(), capacity).unwrap(),
        )))
    }

    #[test]
    fn test_seek_first() {
        let mut it = build_iterator_for_test();
        it.seek_to_first();
        assert!(it.is_valid());
        assert_eq!(&full_key(b"k01", 1)[..], it.key());
        assert_eq!(get_hummock_value(b"v01"), it.value().as_slice());
    }

    #[test]
    fn test_seek_last() {
        let mut it = build_iterator_for_test();
        it.seek_to_last();
        assert!(it.is_valid());
        assert_eq!(&full_key(b"k05", 5)[..], it.key());
        assert_eq!(get_hummock_value(b"v05"), it.value().as_slice());
    }

    #[test]
    fn test_seek_none_front() {
        let mut it = build_iterator_for_test();
        it.seek(&full_key(b"k00", 0)[..]);
        assert!(it.is_valid());
        assert_eq!(&full_key(b"k01", 1)[..], it.key());
        assert_eq!(get_hummock_value(b"v01"), it.value().as_slice());

  
        let mut it = build_iterator_for_test();

        it.seek_le(&full_key(b"k00", 0)[..]);
        assert!(!it.is_valid());
    }

    #[test]
    fn test_seek_none_back() {
        let mut it = build_iterator_for_test();
        it.seek(&full_key(b"k06", 6)[..]);
        assert!(!it.is_valid());

        let mut it = build_iterator_for_test();
        it.seek_le(&full_key(b"k06", 6)[..]);
        assert!(it.is_valid());
        assert_eq!(&full_key(b"k05", 5)[..], it.key());
        assert_eq!(get_hummock_value(b"v05"), it.value().as_slice());
    }

    #[test]
    fn bi_direction_seek() {
        let mut it = build_iterator_for_test();
        it.seek(&full_key(b"k03", 3)[..]);
        assert_eq!(&full_key(format!("k{:02}", 4).as_bytes(), 4)[..], it.key());

        it.seek_le(&full_key(b"k03", 3)[..]);
        assert_eq!(&full_key(format!("k{:02}", 2).as_bytes(), 2)[..], it.key());
    }

    #[test]
    fn test_forward_iterate() {
        let mut it = build_iterator_for_test();

        it.seek_to_first();
        assert!(it.is_valid());
        assert_eq!(&full_key(b"k01", 1)[..], it.key());
        assert_eq!(get_hummock_value(b"v01"), it.value().as_slice());

        it.next();
        assert!(it.is_valid());
        assert_eq!(&full_key(b"k02", 2)[..], it.key());
        assert_eq!(get_hummock_value(b"v02"), it.value().as_slice());

        it.next();
        assert!(it.is_valid());
        assert_eq!(&full_key(b"k04", 4)[..], it.key());
        assert_eq!(get_hummock_value(b"v04"), it.value().as_slice());

        it.next();
        assert!(it.is_valid());
        assert_eq!(&full_key(b"k05", 5)[..], it.key());
        assert_eq!(get_hummock_value(b"v05"), it.value().as_slice());

        it.next();
        assert!(!it.is_valid());
    }

    #[test]
    fn test_backward_iterate() {
        let mut it = build_iterator_for_test();

        it.seek_to_last();
        assert!(it.is_valid());
        assert_eq!(&full_key(b"k05", 5)[..], it.key());
        assert_eq!(get_hummock_value(b"v05"), it.value().as_slice());

        it.prev();
        assert!(it.is_valid());
        assert_eq!(&full_key(b"k04", 4)[..], it.key());
        assert_eq!(get_hummock_value(b"v04"), it.value().as_slice());

        it.prev();
        assert!(it.is_valid());
        assert_eq!(&full_key(b"k02", 2)[..], it.key());
        assert_eq!(get_hummock_value(b"v02"), it.value().as_slice());

        it.prev();
        assert!(it.is_valid());
        assert_eq!(&full_key(b"k01", 1)[..], it.key());
        assert_eq!(get_hummock_value(b"v01"), it.value().as_slice());

        it.prev();
        assert!(!it.is_valid());
    }

    #[test]
    fn test_seek_forward_backward_iterate() {
        let mut it = build_iterator_for_test();

        it.seek(&full_key(b"k03", 3)[..]);
        assert_eq!(&full_key(format!("k{:02}", 4).as_bytes(), 4)[..], it.key());

        it.prev();
        assert_eq!(&full_key(format!("k{:02}", 2).as_bytes(), 2)[..], it.key());

        it.next();
        assert_eq!(&full_key(format!("k{:02}", 4).as_bytes(), 4)[..], it.key());
    }

    pub fn full_key(user_key: &[u8], epoch: u64) -> Bytes {
        let mut buf = BytesMut::with_capacity(user_key.len() + 8);
        buf.put_slice(user_key);
        buf.put_u64(!epoch);
        buf.freeze()
    }
}
