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
    //restart_point_index: usize,
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
            //restart_point_index: usize::MAX,
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
        self.block.get_key(self.index)
    }

    pub fn value(&self) -> Box<Vec<u8>> {
        assert!(self.is_valid());
        self.block.get_value(self.index)
    }
    /*pub fn value(&self) -> Box<Vec<u8>> {
        assert!(self.is_valid());
     
        let start=(self.index+1)*self.block.key_len as usize-3;
        let end=start+3 ;
        
        //println!("start={:?} end={:?} keys[start..end]={:?}",start, end, &self.block.keys[start..end]);
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
        let col_state_bytes_number=((self.block.vaild_entry_count as usize+DEFAULT_DATA_TYPE_BIT_SIZE as usize-1) / DEFAULT_DATA_TYPE_BIT_SIZE as usize) * 4;

        let  mut variable_column_count=0_usize;
        for idx in 0..self.block.column_count as usize {
   
            let state_offset=idx * col_state_bytes_number + (put_entry_count / DEFAULT_DATA_TYPE_BIT_SIZE as usize)*4 ;
            let mut col_state=&self.block.states[..];
            col_state.advance(state_offset );
  
            //get current column value state
            let v=(col_state.get_u32_le()>>(put_entry_count % DEFAULT_DATA_TYPE_BIT_SIZE as usize)) & 0x1;
            let mut column=&self.block.vec_text[idx][..];
            //is Some
            if v>0 {
                value.put_u8(1u8);
                //variable column
                if self.block.data_type_values[idx].1==0{
                    column.advance(put_entry_count * 4  );
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
                    //println!("value={:?}", &value);
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
                    let offset=put_entry_count * self.block.data_type_values[idx].1 ;
                    let text=&column[offset..offset+self.block.data_type_values[idx].1];
                    value.extend_from_slice(text);
                    //println!("value={:?}", &value);
                }
            }else{
                //is None
                value.put_u8(0u8);
            }
        }
        //println!("self.index={:?} value={:?}", self.index, value);
        //assert!(false);
        Box::new(value)
    }*/

    pub fn is_valid(&self) -> bool {
        self.index<self.block.entry_count()
    }

    pub fn seek_to_first(&mut self) {
        self.index=0;
    }

    pub fn seek_to_last(&mut self) {
        self.index=self.block.entry_count()-1;
    }

    pub fn seek(&mut self, key: &[u8]) {
        let mut left = 0_i32;
        let mut right = (self.block.entry_count() - 1) as i32;
        let mut is_hit=false;
        let mut is_into=false;
        let mut compare_type=Ordering::Equal;
        while left>=0 && right>=0 && left <= right {
            //println!("-----------------into seek while----------------");
            let middle = (left + right)/2;
            let raw_key=self.block.get_key(middle as usize);
            is_into=true;
            self.index=middle as usize;
            let ordering=VersionedComparator::compare_key(raw_key, key);
            if ordering==Ordering::Equal{
                is_hit=true;
                break;
            }else if ordering==Ordering::Less{
                left = middle + 1;
                compare_type=Ordering::Less;
            }else{
                right = middle - 1;
                compare_type=Ordering::Greater;
                
            }
        }

        //search key greater all key of block 
        if is_hit==false && is_into {
            if compare_type==Ordering::Less {
                self.index+=1;
            }
        }
    }

    pub fn seek_le(&mut self, key: &[u8]) {
        self.seek(key);
        if !self.is_valid(){
            self.seek_to_last();
        }
        while self.is_valid(){ 
            let raw_key=self.block.get_key(self.index);
            if VersionedComparator::compare_key(raw_key, key)==Ordering::Greater{
                if self.index==0 {
                    self.index=self.block.entry_count();
                }else{
                    self.index-=1;
                }
            }else{
                break;
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use bytes::{BufMut, Bytes};

    use super::*;
    use crate::hummock::{Block, BlockBuilder, BlockBuilderOptions};

    use crate::hummock::test_utils::{TEST_KEYS_COUNT};


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
        let buf = builder.build().1.to_vec();
        let capacity = builder.uncompressed_block_size();
        BlockIterator::new(BlockHolder::from_owned_block(Box::new(
            Block::decode(buf.into(), capacity).unwrap(),
        )))
    }

    fn get_hummock_new_value(number: u32, value: &[u8])->Bytes{
        let mut v=vec![];
        if number==0 || number==35{
            v.push(0_u8);
        }else{
            v.push(1_u8);    
            v.extend_from_slice(&number.to_ne_bytes());
        }
        v.push(1_u8);
        v.extend_from_slice(&(value.len() as u32).to_ne_bytes());
        v.extend_from_slice(value);
        let  mut raw_value=BytesMut::new();
        HummockValue::put(&v[..]).encode(&mut raw_value);
        raw_value.freeze()
    }

    fn new_get_hummock_new_value(number: u32, value: &[u8], number1: u32, value1: &[u8])->Bytes{
        let mut v=vec![];
      
        if number==100{
            v.push(0_u8);
            v.push(0_u8);
            v.push(1_u8);    
            v.extend_from_slice(&number1.to_ne_bytes());
            v.push(0_u8);
        }else{
            v.push(1_u8);    
            v.extend_from_slice(&number.to_ne_bytes());
            
            v.push(1_u8);
            v.extend_from_slice(&(value.len() as u32).to_ne_bytes());
            v.extend_from_slice(value);

            v.push(1_u8);    
            v.extend_from_slice(&number1.to_ne_bytes());
            
            v.push(1_u8);
            v.extend_from_slice(&(value1.len() as u32).to_ne_bytes());
            v.extend_from_slice(value1);
        }

        let  mut raw_value=BytesMut::new();
        HummockValue::put(&v[..]).encode(&mut raw_value);
        raw_value.freeze()
    }

    fn get_delete_hummock_value()->Bytes{
        let mut raw_value=BytesMut::new();
        HummockValue::<&[u8]>::Delete.encode(&mut raw_value);
        raw_value.freeze()
    }
    fn build_iterator_for_new_delete_test() -> BlockIterator {
        let options = BlockBuilderOptions::default();
        let mut builder = BlockBuilder::new(options);
        builder.set_row_deserializer(vec![DataType::Int32, DataType::Varchar]);
        builder.add(&full_key(b"k01", 1), &get_delete_hummock_value()[..]);
        let buf = builder.build().1.to_vec();
        let capacity = builder.uncompressed_block_size();
        BlockIterator::new(BlockHolder::from_owned_block(Box::new(
            Block::decode(buf.into(), capacity).unwrap(),
        )))
    }
    fn build_iterator_for_new_test() -> BlockIterator {
        let options = BlockBuilderOptions::default();
        let mut builder = BlockBuilder::new(options);
        builder.set_row_deserializer(vec![DataType::Int32, DataType::Varchar]);
        builder.add(&full_key(b"k01", 1), &get_hummock_new_value(0, b"v01")[..]);
        builder.add(&full_key(b"k02", 2), &get_hummock_new_value(1, b"v02")[..]);
        builder.add(&full_key(b"k03", 3), &get_delete_hummock_value()[..]);
        builder.add(&full_key(b"k04", 4), &get_hummock_new_value(4,b"v04")[..]);
        builder.add(&full_key(b"k05", 5), &get_hummock_new_value(5, b"v05")[..]);
        builder.add(&full_key(b"k06", 6), &get_hummock_new_value(6,b"v06")[..]);
        builder.add(&full_key(b"k07", 7), &get_delete_hummock_value()[..]);
        builder.add(&full_key(b"k08", 8), &get_hummock_new_value(8,b"v08")[..]);
        builder.add(&full_key(b"k09", 9), &get_hummock_new_value(9, b"v09")[..]);

        builder.add(&full_key(b"k10", 10), &get_hummock_new_value(10, b"v10")[..]);
        builder.add(&full_key(b"k11", 11), &get_hummock_new_value(11, b"v11")[..]);
        builder.add(&full_key(b"k12", 12), &get_hummock_new_value(12,b"v12")[..]);
        builder.add(&full_key(b"k13", 13), &get_hummock_new_value(13, b"v13")[..]);
        builder.add(&full_key(b"k14", 14), &get_hummock_new_value(14, b"v14")[..]);
        builder.add(&full_key(b"k15", 15), &get_hummock_new_value(15, b"v15")[..]);
        builder.add(&full_key(b"k16", 16), &get_hummock_new_value(16,b"v16")[..]);
        builder.add(&full_key(b"k17", 17), &get_hummock_new_value(17, b"v17")[..]);
        builder.add(&full_key(b"k18", 18), &get_hummock_new_value(18, b"v18")[..]);
        builder.add(&full_key(b"k19", 19), &get_hummock_new_value(19, b"v19")[..]);

        builder.add(&full_key(b"k20", 20), &get_hummock_new_value(20, b"v20")[..]);
        builder.add(&full_key(b"k21", 21), &get_hummock_new_value(21, b"v21")[..]);
        builder.add(&full_key(b"k22", 22), &get_hummock_new_value(22,b"v22")[..]);
        builder.add(&full_key(b"k23", 23), &get_hummock_new_value(23, b"v23")[..]);
        builder.add(&full_key(b"k24", 24), &get_hummock_new_value(24, b"v24")[..]);
        builder.add(&full_key(b"k25", 25), &get_hummock_new_value(25, b"v25")[..]);
        builder.add(&full_key(b"k26", 26), &get_hummock_new_value(26,b"v26")[..]);
        builder.add(&full_key(b"k27", 27), &get_hummock_new_value(27, b"v27")[..]);
        builder.add(&full_key(b"k28", 28), &get_hummock_new_value(28, b"v28")[..]);
        builder.add(&full_key(b"k29", 29), &get_hummock_new_value(29, b"v29")[..]);

        builder.add(&full_key(b"k30", 30), &get_hummock_new_value(30, b"v30")[..]);
        builder.add(&full_key(b"k31", 31), &get_hummock_new_value(31, b"v31")[..]);
        builder.add(&full_key(b"k32", 32), &get_hummock_new_value(32,b"v32")[..]);
        builder.add(&full_key(b"k33", 33), &get_hummock_new_value(33, b"v33")[..]);
        builder.add(&full_key(b"k34", 34), &get_hummock_new_value(34, b"v34")[..]);
        builder.add(&full_key(b"k35", 35), &get_hummock_new_value(35, b"v35")[..]);
        builder.add(&full_key(b"k36", 36), &get_hummock_new_value(36, b"v36")[..]);
        //builder.add(&full_key(b"k11", 11), &get_hummock_new_value(11, b"v11")[..]);
        let buf = builder.build().1.to_vec();
        let capacity = builder.uncompressed_block_size();
        BlockIterator::new(BlockHolder::from_owned_block(Box::new(
            Block::decode(buf.into(), capacity).unwrap(),
        )))
    }
    fn new_build_iterator_for_new_test() -> BlockIterator {
        let options = BlockBuilderOptions::default();
        let mut builder = BlockBuilder::new(options);
        builder.set_row_deserializer(vec![DataType::Int32, DataType::Varchar, DataType::Int32, DataType::Varchar]);
        for idx in 0..TEST_KEYS_COUNT{
            if idx>100{
                let use_key=format!("key_test_{:05}", idx).as_bytes().to_vec();
                builder.add(&full_key(&use_key[..], idx as u64), &new_get_hummock_new_value(idx as u32, b"v01", idx as u32 +1 , b"v01")[..]);
            }else{
                let use_key=format!("key_test_{:05}", idx).as_bytes().to_vec();
                builder.add(&full_key(&use_key[..], idx as u64), &new_get_hummock_new_value(idx as u32, b"v01", idx as u32 +1 , b"v01")[..]);
            }
        }
        
        let buf = builder.build().1.to_vec();
        let capacity = builder.uncompressed_block_size();
        BlockIterator::new(BlockHolder::from_owned_block(Box::new(
            Block::decode(buf.into(), capacity).unwrap(),
        )))
    }
  
    #[test]
    fn  group_model_test_seek_first() {
        let mut it = new_build_iterator_for_new_test();
        it.seek_to_first();
        assert!(it.is_valid());
        let use_key=format!("key_test_{:05}", 0).as_bytes().to_vec();
        assert_eq!(&full_key(&use_key[..], 0)[..], it.key());
        assert_eq!(&new_get_hummock_new_value(0, b"v01", 1 , b"v01")[..], it.value().as_slice());
    }

    #[test]
    fn  group_model_test_seek_last() {
        let mut it = new_build_iterator_for_new_test();
        it.seek_to_last();
        assert!(it.is_valid());
        let use_key=format!("key_test_{:05}", TEST_KEYS_COUNT-1).as_bytes().to_vec();
        assert_eq!(&full_key(&use_key[..], TEST_KEYS_COUNT as u64-1)[..], it.key());
        let key= &new_get_hummock_new_value(TEST_KEYS_COUNT as u32 -1, b"v01", TEST_KEYS_COUNT as u32 , b"v01")[..];
        println!("len={:?} key={:?}", key.len(), key);
        println!("{:?}", it.value().as_slice());
        assert_eq!(&new_get_hummock_new_value(TEST_KEYS_COUNT as u32 -1, b"v01", TEST_KEYS_COUNT as u32 , b"v01")[..], it.value().as_slice());
    }

    #[test]
    fn group_model_new_test_node_value() {
        let mut it = new_build_iterator_for_new_test();
        let key_id=TEST_KEYS_COUNT-900;
        //let key_id=1;
        let use_key=format!("key_test_{:05}", key_id).as_bytes().to_vec();
        it.seek(&full_key(&use_key[..], key_id as u64)[..]);
        assert!(it.is_valid());
        assert_eq!(&full_key(&use_key[..], key_id as u64)[..], it.key());
        println!("value={:?}", it.value().as_slice());
        println!("val={:?}", &new_get_hummock_new_value(key_id as u32, b"v01", key_id as u32 +1, b"v01")[..]);
        assert_eq!(&new_get_hummock_new_value(key_id as u32, b"v01", key_id as u32 +1, b"v01")[..], it.value().as_slice());
    
    }

    #[test]
    fn group_model_new_seek_delete() {
        let mut it = build_iterator_for_new_delete_test();
        it.seek(&full_key(b"k01", 1)[..]);
        assert_eq!(&get_delete_hummock_value()[..], &it.value()[..]);
    }

    #[test]
    fn new_seek_delete() {
        let mut it = build_iterator_for_new_test();
        it.seek(&full_key(b"k03", 3)[..]);
        assert_eq!(&get_delete_hummock_value()[..], &it.value()[..]);
    }

    #[test]
    fn new_test_seek_first() {
        let mut it = build_iterator_for_new_test();
        it.seek_to_first();
        assert!(it.is_valid());
        assert_eq!(&full_key(b"k01", 1)[..], it.key());
        assert_eq!(get_hummock_new_value(0, b"v01"), it.value().as_slice());
    }

    #[test]
    fn new_test_seek_last() {
        let mut it = build_iterator_for_new_test();
        it.seek_to_last();
        assert!(it.is_valid());
        assert_eq!(&full_key(b"k36", 36)[..], it.key());
        assert_eq!(get_hummock_new_value(36, b"v36"), it.value().as_slice());
    }

    #[test]
    fn new_test_node_value() {
        let mut it = build_iterator_for_new_test();
        it.seek(&full_key(b"k35", 35)[..]);
        assert!(it.is_valid());
        assert_eq!(&full_key(b"k35", 35)[..], it.key());
        assert_eq!(get_hummock_new_value(35, b"v35"), it.value().as_slice());
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
