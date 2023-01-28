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
use std::io::{Read, Write};
use std::sync::Arc;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use piestream_hummock_sdk::VersionedComparator;
use {lz4, zstd};

use piestream_common::array::{DataChunk, Row, RowDeserializer};

use piestream_common::types::{value_to_type, index_to_len, DataType};
use super::utils::{xxhash64_verify, CompressionAlgorithm};
use crate::hummock::sstable::utils::xxhash64_checksum;
use crate::hummock::{HummockError, HummockResult};
use crate::hummock::value::{VALUE_PUT,VALUE_DELETE};

pub const DEFAULT_DATA_TYPE_BIT_SIZE: u32 = 8 * std::mem::size_of::<u32>() as u32;
pub const DEFAULT_BLOCK_SIZE: usize = 4 * 1024;
pub const DEFAULT_RESTART_INTERVAL: usize = 16;
pub const DEFAULT_ENTRY_SIZE: usize = 24; // table_id(u64) + primary_key(u64) + epoch(u64)

struct Compression{}
impl Compression{
    fn get_decompression_algorithm(gid: u8)->u8{
        gid as u8
    }

    pub fn decompression(buf: &[u8], gid: u8)->Vec<u8>{
        let algorithm=Compression::get_decompression_algorithm(gid);
        match algorithm{
            1=> buf.to_vec(),
            _=> buf.to_vec(),
        }
    }

    pub fn compress(buf: &[u8], gid: u8)->Vec<u8>{
        let algorithm=Compression::get_decompression_algorithm(gid);
        match algorithm{
            1=> buf.to_vec(),
            _=> buf.to_vec(),
        }
    }
}

#[derive(Clone)]
pub struct Block {
    data: Bytes,
    data_len: usize,
    pub entry_count: usize,
    pub vaild_entry_count: usize,
    pub column_count: usize,
    pub variable_column_count: usize,
    pub key_offset_start_pos: usize,
    pub key_len: usize,
    pub variable_text_len: usize,
    pub data_type_values: Arc<Vec<(u8,usize)>>,
    pub states: Arc<Vec<u8>>,
    pub columns_offset: Arc<Vec<u32>>,
   
}

impl Block {
    pub fn decode(buf: Bytes, uncompressed_capacity: usize) -> HummockResult<Self> {
        //println!("uncompressed_capacity={:?}", uncompressed_capacity);
        // Verify checksum.
        let xxhash64_checksum = (&buf[buf.len() - 8..]).get_u64_le();
        //println!("decode xxhash64_checksum={:?}", xxhash64_checksum);
        xxhash64_verify(&buf[..buf.len() - 8], xxhash64_checksum)?;
        // Decompress.
        let compression = CompressionAlgorithm::decode(&mut &buf[buf.len() - 9..buf.len() - 8])?;
        let compressed_data = &buf[..buf.len() - 9];
        let buf = match compression {
            CompressionAlgorithm::None => buf.slice(0..(buf.len() - 9)),
            CompressionAlgorithm::Lz4 => {
                let mut decoder = lz4::Decoder::new(compressed_data.reader())
                    .map_err(HummockError::decode_error)?;
                let mut decoded = Vec::with_capacity(uncompressed_capacity);
                decoder
                    .read_to_end(&mut decoded)
                    .map_err(HummockError::decode_error)?;
                debug_assert_eq!(decoded.capacity(), uncompressed_capacity);
                Bytes::from(decoded)
            }
            CompressionAlgorithm::Zstd => {
                let mut decoder = zstd::Decoder::new(compressed_data.reader())
                    .map_err(HummockError::decode_error)?;
                let mut decoded = Vec::with_capacity(uncompressed_capacity);
                decoder
                    .read_to_end(&mut decoded)
                    .map_err(HummockError::decode_error)?;
                debug_assert_eq!(decoded.capacity(), uncompressed_capacity);
                Bytes::from(decoded)
            }
        };
        Ok(Self::decode_from_raw(Self::decode_columns_from_raw(buf)))
    }

    fn decode_columns_from_raw(buf: Bytes)->Bytes {
        let mut offset=0;
        let total_raw_len=(&buf[buf.len()-offset-4..]).get_u32_le() as usize;
        offset+=4;

        let entry_count=(&buf[buf.len()-offset-2..]).get_u16_le() as usize;
        offset+=2;
        let vaild_entry_count=(&buf[buf.len()-offset-2..]).get_u16_le() as usize;
        offset+=2;
        let column_count=(&buf[buf.len()-offset-2..]).get_u16_le() as usize;
        offset+=2;

        let key_len=(&buf[buf.len()-offset-4..]).get_u32_le() as usize;
        offset+=4;
        let variable_text_len=(&buf[buf.len()-offset-4..]).get_u32_le() as usize;
        offset+=4;
        let fixed_column_len=(&buf[buf.len()-offset-4..]).get_u32_le() as usize;
        offset+=4;
        let key_and_column_offset_len=(&buf[buf.len()-offset-4..]).get_u32_le() as usize;
        offset+=4;
        let value_state_len=(&buf[buf.len()-offset-4..]).get_u32_le() as usize;
        offset+=4;

        let save_group_compress_size=(&buf[buf.len()-offset-2..]).get_u16_le() as usize;
        offset+=2;
        let map_group_data_type_len=(&buf[buf.len()-offset-2..]).get_u16_le() as usize;
        offset+=2;
        let groups_compress_number=(&buf[buf.len()-offset-2..]).get_u16_le() as usize;
        offset+=2;

        let mut map_columns_offset= BTreeMap::new();
        let mut new_buffer=BytesMut::with_capacity(total_raw_len+column_count*4+64);
        new_buffer.extend_from_slice(&buf[0..key_len+variable_text_len]);

        let columns_data_type_buf=&buf[buf.len()-offset-column_count..buf.len()-offset];
        offset+=column_count;
        new_buffer.extend_from_slice(&columns_data_type_buf);
       

        let mut map_group_data_type_buf=&buf[buf.len()-offset-map_group_data_type_len..buf.len()-offset];
        offset+=map_group_data_type_len;
        //new_buffer.extend_from_slice(&map_group_data_type_buf);
        
        let mut variable_column=0_usize;
        let mut map_group_data_type= BTreeMap::new();
        let size=map_group_data_type_buf.get_u8() as usize;
        for _ in 0..size {
            let gid=map_group_data_type_buf.get_u8();
            println!("gid={:?}", gid);
            let vlen=map_group_data_type_buf.get_u16_le() as usize;
            if gid==u8::MAX{
                variable_column=vlen;
            }
            let mut v=vec![];
            for _ in 0..vlen{
                v.push(map_group_data_type_buf.get_u16_le() as u32);
            }
            map_group_data_type.insert(gid, v);
        }

        //println!("22222222save_group_compress_size={:?}", save_group_compress_size);
        let mut groups_compress_len_buf=&buf[buf.len()-offset-save_group_compress_size..buf.len()-offset];
        offset+=save_group_compress_size;

        assert_eq!(save_group_compress_size/4, groups_compress_number);
        let mut groups_compress_len=vec![];
        for _ in 0..groups_compress_number{
            groups_compress_len.push(groups_compress_len_buf.get_u32_le());
        } 

        let value_state_buf=&buf[buf.len()-offset-value_state_len..buf.len()-offset];
        offset+=value_state_len;
        //println!("value_state_buf={:?}", value_state_buf);

        //adjust to colunm index 
        let mut map_states= BTreeMap::new();
        let col_state_bytes_number=((vaild_entry_count+DEFAULT_DATA_TYPE_BIT_SIZE as usize-1) / DEFAULT_DATA_TYPE_BIT_SIZE as usize) * 4;
        assert_eq!(col_state_bytes_number*column_count, value_state_len);
        let mut count=0;
        for (_,v) in  &map_group_data_type{  
            for (_, cidx) in v.iter().enumerate(){
                let mut vec=vec![];
                let start=count*col_state_bytes_number;
                let end=(count+1)*col_state_bytes_number;
                //println!("cidx={:?} value_state_buf[start..end]={:?}", cidx, &value_state_buf[start..end]);
                assert!(end<=value_state_buf.len());
                vec.extend_from_slice(&value_state_buf[start..end]);
                map_states.insert(cidx, vec);
                count+=1;
            }
        }
        let state_offset=new_buffer.len();
        println!("state new_buffer.len:{:?}", new_buffer.len());
        for (_,v) in  &map_states{
            //println!("*****idx={:?} v={:?}", idx, &v);
            new_buffer.extend_from_slice(&v[..]);
        }

        //let state_offset=new_buffer.len();
        //new_buffer.extend_from_slice(&value_state_buf);
    
        let key_offset_start_pos=new_buffer.len();
        let key_and_column_offset_buf=&buf[buf.len()-offset-key_and_column_offset_len..buf.len()-offset];
        offset+=key_and_column_offset_len;
        println!("entry_count={:?} variable_column={:?} key_and_column_offset_len={:?}", entry_count, variable_column, key_and_column_offset_len);
        let buffer=Compression::decompression(key_and_column_offset_buf, u8::MAX);
        assert_eq!(((entry_count+vaild_entry_count*variable_column)*4) as usize, buffer.len());
        new_buffer.extend_from_slice(&buffer.as_slice());

        println!("fixed_column_len={:?}", fixed_column_len);
        let fixed_column_buf=&buf[buf.len()-offset-fixed_column_len..buf.len()-offset];
        //offset+=fixed_column_len;

        let mut current_start_pos=0_usize;
        let mut idx=0;
        for (gid,v) in  &map_group_data_type{
            if *gid==u8::MAX{
                for (i, cidx) in v.iter().enumerate(){
                    let pos=key_offset_start_pos+entry_count*4+i*vaild_entry_count*4;
                    map_columns_offset.insert(cidx, pos);
                }
                continue;
            }
            let len=index_to_len(*gid as u8);
            let tmp=&fixed_column_buf[current_start_pos..current_start_pos+groups_compress_len[idx] as usize];
            println!("gid={:?} tmp={:?}", *gid, tmp);
            let buffer=Compression::decompression(tmp, *gid);
            println!("vaild_entry_count={:?} len={:?} v.len={:?}", vaild_entry_count, len, v.len());
            assert_eq!(vaild_entry_count*len*v.len(), buffer.len());
            
            for (i, cidx) in v.iter().enumerate(){
                let pos=new_buffer.len()+i*vaild_entry_count*len;
                map_columns_offset.insert(cidx, pos);
            }
            new_buffer.extend_from_slice(&buffer.as_slice());
            
            current_start_pos+=groups_compress_len[idx] as usize;
            idx+=1;
        }

        //all field start offset 
        for (_, offset) in map_columns_offset.iter(){
            new_buffer.put_u32_le(*offset as u32);
        }
        new_buffer.put_u32_le(state_offset as u32);
        new_buffer.put_u32_le(key_offset_start_pos as u32);
        new_buffer.put_u32_le(value_state_len as u32);
        new_buffer.put_u32_le(variable_text_len as u32);
        new_buffer.put_u32_le(key_len as u32);
        new_buffer.put_u16_le(column_count as u16);
        //row count of not delete
        new_buffer.put_u16_le(vaild_entry_count as u16);
        //total row count
        new_buffer.put_u16_le(entry_count as u16);
        println!("total_raw_len+column_count*4+64={:?} new_buffer.len={:?}", total_raw_len+column_count*4+64, new_buffer.len());
        new_buffer.freeze()
    }
    
    pub fn decode_from_raw(buf: Bytes) -> Self {
        //let buf=data.clone();
        let mut offset=0;
        let entry_count=(&buf[buf.len()-offset-2..]).get_u16_le() as usize;
        offset+=2;
        let vaild_entry_count=(&buf[buf.len()-offset-2..]).get_u16_le() as usize;
        offset+=2;
        let column_count=(&buf[buf.len()-offset-2..]).get_u16_le() as usize;
        offset+=2;

        let key_len=(&buf[buf.len()-offset-4..]).get_u32_le() as usize;
        offset+=4;
        let variable_text_len=(&buf[buf.len()-offset-4..]).get_u32_le() as usize;
        offset+=4;
        let value_state_len=(&buf[buf.len()-offset-4..]).get_u32_le() as usize;
        offset+=4;
        let key_offset_start_pos=(&buf[buf.len()-offset-4..]).get_u32_le() as usize;
        offset+=4;
        let state_offset=(&buf[buf.len()-offset-4..]).get_u32_le() as usize;
        offset+=4;
       
        let mut columns_offset_buf=&buf[buf.len()-offset-column_count*4..buf.len()-offset];
        //offset+=column_count*4;
        let mut columns_offset=vec![];
        for _ in 0..column_count{
            columns_offset.push(columns_offset_buf.get_u32_le());
        }

        let state_buf=&buf[state_offset..state_offset+value_state_len];
        //println!("value_state_len={:?} state_offset={:?} state_buf={:?}", value_state_len, state_offset, state_buf);
        let mut states= Vec::with_capacity(value_state_len);
        states.extend_from_slice(state_buf);

        let offset_data_type=key_len+variable_text_len;
        let mut data_type_values= Vec::with_capacity(column_count);
        let mut ptr=&buf[offset_data_type..offset_data_type+column_count];
        let  mut variable_column_count=0_usize;
        for _ in  0..column_count{
            let v=ptr.get_u8();
            let len=index_to_len(v);
            if len==0{
                variable_column_count+=1;
            }
            data_type_values.push((v, len));
        }
        
        Block{
            data_len:buf.len(),
            data:buf,
            entry_count: entry_count,
            vaild_entry_count: vaild_entry_count,
            column_count: column_count ,
            variable_column_count:variable_column_count,
            key_offset_start_pos:key_offset_start_pos,
            key_len:key_len ,
            variable_text_len:variable_text_len,
            data_type_values:Arc::new(data_type_values),
            states:Arc::new(states),
            columns_offset:Arc::new(columns_offset),
        }
    }
    
    //pub fn key_index_offset(&self, idx:usize)->usize{

    //}
    pub fn get_key(&self, index:usize)->&[u8]{
        let key=self.get_raw_key(index);
        println!("get_key={:?}", &key[0..key.len()-3]);
        &key[0..key.len()-3]
    }

    pub fn get_raw_key(&self, index:usize)->&[u8]{
        let mut pos=self.key_offset_start_pos+index*4;
        let offset=(&self.data[pos..pos+4]).get_u32_le() as usize;
        let next_offset;
        if (index+1)<self.entry_count{
            pos+=4;
            next_offset=(&self.data[pos..pos+4]).get_u32_le() as usize;
        }else{
            next_offset=self.key_len;
        }
        println!("index={:?} offset={:?} next_offset={:?}, key_len={:?}", index, offset, next_offset, self.key_len);
        assert!(next_offset>offset+3);
        &self.data[offset..next_offset]
    }

    pub fn get_value(&self, index:usize) -> Box<Vec<u8>> {
        let key=self.get_raw_key(index);
        let mut buf=&key[key.len()-3..key.len()];
    
        let row_state=buf.get_u8();
        if row_state==VALUE_DELETE{
            return Box::new(vec![VALUE_DELETE]);
        }

        let current_put_entry_idx=buf.get_u16_le() as usize;
        println!("current_put_entry_idx={:?} vaild_entry_count={:?}",current_put_entry_idx, self.vaild_entry_count );
        assert!(current_put_entry_idx<self.vaild_entry_count as usize);
    
        let mut value =Vec::new();
        value.put_u8(VALUE_PUT);
        let col_state_bytes_number=((self.vaild_entry_count as usize+DEFAULT_DATA_TYPE_BIT_SIZE as usize-1) / DEFAULT_DATA_TYPE_BIT_SIZE as usize) * 4;

        let  mut variable_column_count=0_usize;
        for idx in 0..self.column_count as usize {
   
            let state_offset=idx * col_state_bytes_number + (current_put_entry_idx / DEFAULT_DATA_TYPE_BIT_SIZE as usize)*4 ;
            let mut col_state=&self.states[..];
            col_state.advance(state_offset );
           
  
            let stat=col_state.get_u32_le();
            //get current column value state
            let v=(stat>>(current_put_entry_idx % DEFAULT_DATA_TYPE_BIT_SIZE as usize)) & 0x1;
            println!("idx={:?} state_offset={:?} stat={:?}", idx, state_offset, stat);
            //is Some
            if v>0 {
                value.put_u8(1u8);
                //variable column
                if self.data_type_values[idx].1==0{
                    let mut pos=self.columns_offset[idx] as usize+current_put_entry_idx * 4;
                    let offset=(&self.data[pos..pos+4]).get_u32_le() as usize;

                    let  next_offset;
                    if (current_put_entry_idx+1) < self.vaild_entry_count as usize{
                        let pos=self.columns_offset[idx] as usize+(current_put_entry_idx+1) * 4;
                        next_offset=(&self.data[pos..pos+4]).get_u32_le() as usize;
                    }else{
                        if (variable_column_count+1)<self.variable_column_count{
                            pos+=4;
                            //offset=(&self.data[pos..pos+4]).get_u32_le() as usize;
                            next_offset=(&self.data[pos..pos+4]).get_u32_le() as usize;
                        }else{
                            next_offset=self.key_len+self.variable_text_len;
                        }
                    }
                    let text=&self.data[offset..next_offset];
                    value.extend_from_slice(text);
                    variable_column_count+=1;
                    
                }else{
                    //fixed column
                    let offset=self.columns_offset[idx] as usize+current_put_entry_idx * self.data_type_values[idx].1 ;
                    let text=&self.data[offset..offset+self.data_type_values[idx].1];
                    println!("data_type_len={:?}", self.data_type_values[idx].1);
                    println!("text={:?}", text);
                    value.extend_from_slice(text);
                }
            }else{
                //is None
                value.put_u8(0u8);
            }
        }

        Box::new(value)
    }

    pub fn entry_count(&self) -> usize{
        self.entry_count as usize
    }
   
    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    /// Entries data len.
    #[expect(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        assert!(!self.data.is_empty());
        self.data_len
    }


    pub fn data(&self) -> &[u8] {
        &self.data[..self.data_len]
    }

    pub fn raw_data(&self) -> &[u8] {
        &self.data[..]
    }
}

pub struct BlockBuilderOptions {
    /// Reserved bytes size when creating buffer to avoid frequent allocating.
    pub capacity: usize,
    /// Compression algorithm.
    pub compression_algorithm: CompressionAlgorithm,
    /// Restart point interval.
    pub restart_interval: usize,
}

impl Default for BlockBuilderOptions {
    fn default() -> Self {
        Self {
            capacity: DEFAULT_BLOCK_SIZE,
            compression_algorithm: CompressionAlgorithm::None,
            restart_interval: DEFAULT_RESTART_INTERVAL,
        }
    }
}

/// [`BlockBuilder`] encodes and appends block to a buffer.
pub struct BlockBuilder {
    /// Write buffer.
    buf: BytesMut,
    /// Last key.
    last_key: Vec<u8>,
    /// Count of put entries in current block.
    put_entry_count: usize,
    /// Count of entries in current block.
    entry_count: usize,
    //column number
    column_num: usize,
    //column number
    variable_columns: Vec<usize>,
    map_data_type: BTreeMap<u32, Vec<u32>>,
    // need mem size
    current_mem_size: usize,  
    uncompressed_block_size: usize,
    /// Compression algorithm.
    compression_algorithm: CompressionAlgorithm,
    //table desc
    //table_column_datatype: Vec<DataType>,
    //row_deserializer: Option<Arc<RowDeserializer>>,
    row_deserializer: Option<RowDeserializer>,
    data_type_bit_size: u32,
    //hummock_value_list: Vec<u32>,
    //put_record_idxs: Vec<u16>,
    //keys: Vec<Vec<u8>>,
    keys_offset: Vec<u8>,
    //keys_len: Vec<u16>,
    rows: Vec<Row>,

}

impl BlockBuilder {
    pub fn new(options: BlockBuilderOptions) -> Self {
        Self {
            // add more space to avoid re-allocate space.
            buf: BytesMut::with_capacity(options.capacity+256),
            last_key: vec![],
            put_entry_count: 0,
            entry_count: 0,
            column_num: 0,
            variable_columns: Vec::new() ,
            map_data_type: BTreeMap::new(),
            current_mem_size: 0,
            uncompressed_block_size: 0,
            compression_algorithm: options.compression_algorithm,
            row_deserializer: None,
            data_type_bit_size: DEFAULT_DATA_TYPE_BIT_SIZE,
            //hummock_value_list: Vec::new(),
            //put_record_idxs: Vec::new(),
            //keys: Vec::new(),
            keys_offset: Vec::new(),
            //keys_len: Vec::new(),
            rows: Vec::new(),

            //table_column_datatype:Vec::new(),
        }
    }

    pub fn set_row_deserializer(&mut self, table_column_datatype: Vec<DataType>){
        //println!("table_column_datatype={:?}", table_column_datatype);
        self.variable_columns.clear();
        self.map_data_type.clear();
        //self.row_deserializer=Some(Arc::new(RowDeserializer::new(table_column_datatype)));
        self.row_deserializer=Some(RowDeserializer::new(table_column_datatype));
        let data_types=self.row_deserializer.as_ref().unwrap().data_types();
        self.column_num=data_types.len();
        for idx in 0..self.column_num{
            //fixed len columns
            if data_types[idx].data_type_len()==0{
                self.variable_columns.push(idx);
            };

            match self.map_data_type
                .entry(data_types[idx].type_to_group_id() as u32)
            {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().push(idx as u32);
                }
                Entry::Vacant(entry) => {
                    entry.insert(vec![idx as u32]);
                }
            };
        }
        //println!("set_row_deserializer self.variable_columns.len()={:?}", self.variable_columns.len());
    }

    pub fn get_put_record_count(&self)->usize{
        self.rows.len()
    }

    pub fn add(&mut self, key: &[u8], value: &[u8]) {
        println!("*****************key_len={:?} value={:?}*******************", key, value);
        //println!("add value={:?}", &value);
        if self.entry_count > 0 {
            debug_assert!(!key.is_empty());
            debug_assert_eq!(
                VersionedComparator::compare_key(&self.last_key[..], key),
                Ordering::Less
            );
        }
        //println!("hummock_value={:?} raw_key_len={}", value, key.len());
        let value_len=value.len();
        let buffer=&mut &value[..];
        let is_put=match buffer.get_u8() {
            VALUE_PUT => true,
            _=>false
        };
        //let is_put= self.is_put(&mut &value[..]);
       
        //println!("is_put={:?} hummock_value={:?}", is_put, buffer);
        //let idx=self.entry_count % self.data_type_bit_size as usize;
        //if idx==0{
        //    self.hummock_value_list.push(0_u32);
        //}

        //let mut key_vec=key.to_vec();
        //self.keys_offset[self.entry_count]=self.buf.len() as u32;
        self.keys_offset.extend_from_slice(&(self.buf.len() as u32).to_le_bytes());
        self.buf.extend_from_slice(&key[..]);
        if is_put {
            let row = self.row_deserializer.as_ref().unwrap().deserialize(buffer).unwrap();
            //println!("row={:?}", row);
            self.rows.push(row.clone());
            //let v=self.hummock_value_list.last_mut().unwrap();
            //*v|=1<<idx;
            //println!("v={:?}", v);
            //self.put_record_idxs.push(self.put_entry_count as u16);
            //self.put_entry_count+=1;
            //key_vec.push(VALUE_PUT as u8);
            self.buf.put_u8(VALUE_PUT as u8);
            //key_vec.append(&mut (self.put_entry_count as u16).to_le_bytes().to_vec());
            self.buf.put_u16_le(self.put_entry_count as u16);
            self.put_entry_count+=1;
            self.current_mem_size+=key.len()+3+value_len+self.variable_columns.len()*4;
            for i in 0..row.size(){
                let len=match row.0[i] {
                    //fill default value to fixed data type 
                    None=>{
                        self.row_deserializer.as_ref().unwrap().data_types()[i].data_type_len()
                    },
                    //not none 
                    _=>{0}
                };
                self.current_mem_size+=len;
                //println!("len={:?} self.current_mem_size={:?}", len, self.current_mem_size);
            }
        }else{
            //key_vec.push(VALUE_DELETE as u8);
            self.buf.put_u8(VALUE_DELETE as u8);
            //key_vec.append(&mut u16::MAX.to_le_bytes().to_vec());
            self.buf.put_u16_le(u16::MAX);
            //self.put_record_idxs.push(u16::MAX);
            self.current_mem_size+=key.len()+3+value_len;
        }
        //self.keys_len[self.entry_count]=(self.buf.len()-self.keys_offset[self.entry_count] as usize) as u16;
        //self.keys.push(key.to_vec());
        //self.keys.push(key_vec);
        self.last_key.clear();
        self.last_key.extend_from_slice(key);
        //self.current_mem_size+=(key.len()+value.len()+1+2+2+4);
        self.entry_count += 1;
    }

    pub fn build(&mut self) -> (u32, &[u8]) {
        println!("*****************build*******************");
        assert_eq!(self.keys_offset.len(),  4*self.entry_count);

     
        let data_types=self.row_deserializer.as_ref().unwrap().data_types();
        let data_chunk=DataChunk::from_rows(&self.rows, data_types);
   
        let mut buffers = vec![vec![]; self.map_data_type.len()];
        let mut column_value_state_list = vec![vec![]; self.map_data_type.len()];
        //let mut variable_offsets=vec![];
        
        let key_len=self.buf.len();
        let mut offset= key_len;
        let mut total_raw_len=key_len;
        //save variable text to buf
        data_chunk.serialize_columns(&self.map_data_type, &mut buffers, &mut column_value_state_list, &mut self.keys_offset, &mut self.buf);
        assert_eq!(buffers.len(), self.map_data_type.len());
        assert_eq!(column_value_state_list.len(), self.map_data_type.len());
        //println!("column_value_state_list={:?}", column_value_state_list);
        println!("buffers={:?}", buffers);

        let mut  groups_compress_len=vec![];
        let variable_text_len=self.buf.len()-offset;
        offset += variable_text_len;
        total_raw_len+=variable_text_len;

        let mut variable_columns=0;
        //compress fixed len column
        for (idx, (gid, vec_culumns_idx)) in  (&self.map_data_type).iter().enumerate(){
            if *gid==u8::MAX as u32{
                variable_columns=vec_culumns_idx.len();
                continue;
            }
            let first_column_idx=vec_culumns_idx[0];
            let column_len=data_types[first_column_idx as usize].data_type_len();
            total_raw_len+=buffers[idx].len();
         
            assert_eq!(buffers[idx].len(), self.put_entry_count*vec_culumns_idx.len()*column_len);
            let v=Compression::compress( &buffers[idx], *gid as u8);
            groups_compress_len.push(v.len() as u32);
            self.buf.extend_from_slice(v.as_slice());
            //println!("buf={:?}", self.buf);
        }
        let fixed_column_len=self.buf.len()-offset;
        offset+=fixed_column_len;
        
        //save offset include key and variable
        assert_eq!(self.keys_offset.len(), 4*self.entry_count+self.put_entry_count*variable_columns*4);
        let v=Compression::compress(&self.keys_offset,u8::MAX);
        groups_compress_len.push(v.len() as u32);
        self.buf.extend_from_slice(v.as_slice());
        if variable_columns==0{
            assert_eq!(groups_compress_len.len(), self.map_data_type.len()+1);
        }else{
            assert_eq!(groups_compress_len.len(), self.map_data_type.len());
        }
        
        total_raw_len+=self.keys_offset.len();
        let key_and_column_offset_len=self.buf.len()-offset;
        offset+=key_and_column_offset_len;

        let vec_data_type: Vec<&Vec<u32>>=self.map_data_type.values().collect();
        /*for v in vec_data_type{
            println!("**********v={:?}", v);
        }*/
        //store each column value state
        let size=(self.put_entry_count + (self.data_type_bit_size as usize -1)) / self.data_type_bit_size as usize;
        let mut state_list =vec![vec![]; column_value_state_list.len()];
        for (pos, column_state_list) in column_value_state_list.iter().enumerate(){
            assert_eq!(self.put_entry_count*&vec_data_type[pos].len(), column_state_list.len());
            let mut count=0;
            let mut column_record_count=0;
            for (_,element) in column_state_list.iter().enumerate(){
                if column_record_count % self.data_type_bit_size as usize==0{
                    state_list[pos].push(0_u32);
                    //println!("11111111111111111111");
                }
                if *element==1 {
                    let v=state_list[pos].last_mut().unwrap();
                    *v|=1<<(column_record_count%self.data_type_bit_size as usize);
                }
                
                column_record_count+=1;
                if column_record_count%self.put_entry_count==0{
                    column_record_count=0;
                    //println!("22222222222222222");
                    count+=1;
                    assert_eq!(state_list[pos].len(), size*count);
                    
                }
                //column_record_count+=1;
            }
            assert_eq!(state_list[pos].len(), size*&vec_data_type[pos].len());
        }
       
        //println!("state_list={:?}", state_list);
        for idx in 0..state_list.len() {
            for v in &state_list[idx]{
                //println!("list={:?}", &v.to_le_bytes());
                self.buf.extend_from_slice(&v.to_le_bytes());
            }
        }
        let value_state_len=self.buf.len()-offset;
        offset+=value_state_len;
        total_raw_len+=value_state_len;
        
        //self.buf.put_u16_le(groups_compress_len.len() as u16);
        for compress_len in &groups_compress_len{
            //self.buf.extend_from_slice(&(compress_len).to_le_bytes());
            self.buf.put_u32_le(*compress_len);
        }
  
        let save_group_compress_size=self.buf.len()-offset;
        //println!("11111111save_group_compress_size={:?}", save_group_compress_size);
        offset+=save_group_compress_size;
        total_raw_len+=save_group_compress_size;

        self.buf.put_u8(self.map_data_type.len() as u8);
        for  (gid, v) in &self.map_data_type {
            //println!("***********gid={:?}", *gid);
            let vlen=v.len();
            self.buf.put_u8(*gid as u8);
            self.buf.put_u16_le(vlen as u16);
            for idx in v{
                self.buf.put_u16_le(*idx as u16);
                //self.buf.extend_from_slice(&(*idx as u16).to_le_bytes());
            }
        }
        
        let map_data_type_len=self.buf.len()-offset;
        offset+=map_data_type_len;
        total_raw_len+=map_data_type_len;

         //column type
         for data_type in data_types {
            self.buf.put_u8(data_type.type_to_index());
        }

        let data_type_len=self.buf.len()-offset;
        //offset+=data_type_len;
        total_raw_len+=data_type_len;
        assert_eq!(data_type_len, data_types.len());

        //equal map_data_type_len or map_data_type_len+1 
        self.buf.put_u16_le(groups_compress_len.len() as u16);
        total_raw_len+=2;
        //self.buf.put_u8(self.map_data_type.len() as u8);
        //self.buf.put_u16_le(data_type_len as u16);
        self.buf.put_u16_le(map_data_type_len as u16);
        total_raw_len+=2;
        self.buf.put_u16_le(save_group_compress_size as u16);
        total_raw_len+=2;
        //value state len 
        self.buf.put_u32_le(value_state_len as u32);
        total_raw_len+=4;
        //offset len 
        self.buf.put_u32_le(key_and_column_offset_len as u32);
        total_raw_len+=4;
        self.buf.put_u32_le(fixed_column_len as u32);
        total_raw_len+=4;
        self.buf.put_u32_le(variable_text_len as u32);
        total_raw_len+=4;
        self.buf.put_u32_le(key_len as u32);
        total_raw_len+=4;
        self.buf.put_u16_le(data_type_len as u16);
        total_raw_len+=2;
        //row count of not delete
        self.buf.put_u16_le(self.put_entry_count as u16);
        total_raw_len+=2;
        //total row count
        self.buf.put_u16_le(self.entry_count as u16);
        total_raw_len+=2;

        total_raw_len+=4;
        self.buf.put_u32_le(total_raw_len as u32);
        
        self.uncompressed_block_size=self.buf.len();
        //compression block
        match self.compression_algorithm {
            CompressionAlgorithm::None => (),
            CompressionAlgorithm::Lz4 => {
                let mut encoder = lz4::EncoderBuilder::new()
                    .level(4)
                    .build(BytesMut::with_capacity(self.buf.len()).writer())
                    .map_err(HummockError::encode_error)
                    .unwrap();
                encoder
                    .write_all(&self.buf[..])
                    .map_err(HummockError::encode_error)
                    .unwrap();
                let (writer, result) = encoder.finish();
                result.map_err(HummockError::encode_error).unwrap();
                self.buf = writer.into_inner();
            }
            CompressionAlgorithm::Zstd => {
                let mut encoder =
                    zstd::Encoder::new(BytesMut::with_capacity(self.buf.len()).writer(), 4)
                        .map_err(HummockError::encode_error)
                        .unwrap();
                encoder
                    .write_all(&self.buf[..])
                    .map_err(HummockError::encode_error)
                    .unwrap();
                let writer = encoder
                    .finish()
                    .map_err(HummockError::encode_error)
                    .unwrap();
                self.buf = writer.into_inner();
            }
        };
        self.compression_algorithm.encode(&mut self.buf);
        let checksum = xxhash64_checksum(&self.buf);
        self.buf.put_u64_le(checksum);
        (self.uncompressed_block_size as u32, self.buf.as_ref())
    }

    pub fn get_last_key(&self) -> &[u8] {
        &self.last_key
    }

    pub fn is_empty(&self) -> bool {
        self.entry_count==0
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    
        self.last_key.clear();
        //self.hummock_value_list.clear();
        self.keys_offset.clear();
        self.rows.clear();
        self.keys_offset.clear();
        //self.variable_columns.clear();
        self.put_entry_count=0;
        self.entry_count = 0;
        self.current_mem_size=0;
        self.uncompressed_block_size=0;
    
    }

    /// Calculate block size without compression.
    pub fn uncompressed_block_size(&mut self) -> usize {
        self.uncompressed_block_size
    }


    /// Approximate block len (uncompressed).
    pub fn approximate_len(&self) -> usize {
        let data_types=self.row_deserializer.as_ref().unwrap().data_types();
        self.current_mem_size+data_types.len()+(self.column_num+self.variable_columns.len())*(4+4)+8+4+4+2+4+4+2+2+4*4
   
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;
    use crate::hummock::{BlockHolder, BlockIterator, HummockValue};

    fn get_hummock_new_value(value: &[u8])->Bytes{
        let mut v=vec![];
        v.push(1_u8);
        v.extend_from_slice(&(value.len() as u32).to_ne_bytes());
        v.extend_from_slice(value);
        let  mut raw_value=BytesMut::new();
        HummockValue::put(&v[..]).encode(&mut raw_value);
        raw_value.freeze()
    }

    #[test]
    fn new_test_block_enc_dec() {
        let options = BlockBuilderOptions::default();
        let key=vec![116, 0, 0, 3, 234, 16, 0, 131, 90, 222, 45, 226, 12, 0, 0, 255, 242, 148, 135, 73, 99, 255, 255];
        let value=vec![0, 1, 1, 0, 0, 0, 1, 0, 0, 140, 67, 20, 231, 90, 3];
        let mut builder = BlockBuilder::new(options);
        builder.set_row_deserializer( vec![DataType::Int32,DataType::Int64]);
        builder.add(&key[..], &value[..]);
      
        let buf = builder.build().1.to_vec();
        let capacity = builder.uncompressed_block_size();
        let block = Box::new(Block::decode(buf.into(), capacity).unwrap());
        let mut bi = BlockIterator::new(BlockHolder::from_owned_block(block));
        bi.seek_to_first();
        println!("********key={:?} value={:?}", bi.key(), bi.value());
    }
    #[test]
    fn test_block_enc_dec() {
        let options = BlockBuilderOptions::default();
        let mut builder = BlockBuilder::new(options);
        builder.set_row_deserializer( vec![DataType::Varchar]);
        builder.add(&full_key(b"k1", 1), &get_hummock_new_value(b"v01")[..]);
        builder.add(&full_key(b"k22", 2), &get_hummock_new_value(b"v026")[..]);
        builder.add(&full_key(b"k333", 3), &get_hummock_new_value(b"v03566")[..]);
        builder.add(&full_key(b"k4444", 4), &get_hummock_new_value(b"v046666")[..]);
        let buf = builder.build().1.to_vec();
        let capacity = builder.uncompressed_block_size();
        let block = Box::new(Block::decode(buf.into(), capacity).unwrap());
        let mut bi = BlockIterator::new(BlockHolder::from_owned_block(block));

        bi.seek_to_first();
        assert!(bi.is_valid());
        assert_eq!(&full_key(b"k1", 1)[..], bi.key());
        assert_eq!(get_hummock_new_value(b"v01"), bi.value().as_slice());

        bi.next();
        assert!(bi.is_valid());
        assert_eq!(&full_key(b"k22", 2)[..], bi.key());
        assert_eq!(get_hummock_new_value(b"v026"), bi.value().as_slice());

        bi.next();
        assert!(bi.is_valid());
        assert_eq!(&full_key(b"k333", 3)[..], bi.key());
        assert_eq!(get_hummock_new_value(b"v03566"), bi.value().as_slice());

        bi.next();
        assert!(bi.is_valid());
        assert_eq!(&full_key(b"k4444", 4)[..], bi.key());
        assert_eq!(get_hummock_new_value(b"v046666"), bi.value().as_slice());

        bi.next();
        assert!(!bi.is_valid());
    }

    #[test]
    fn test_compressed_block_enc_dec() {
        inner_test_compressed(CompressionAlgorithm::Lz4);
        inner_test_compressed(CompressionAlgorithm::Zstd);
    }

    fn inner_test_compressed(algo: CompressionAlgorithm) {
        let options = BlockBuilderOptions {
            compression_algorithm: algo,
            ..Default::default()
        };

        let mut builder = BlockBuilder::new(options);
        builder.set_row_deserializer(vec![DataType::Varchar]);
        builder.add(&full_key(b"k1", 1), &get_hummock_new_value(b"v01")[..]);
        builder.add(&full_key(b"k2", 2), &get_hummock_new_value(b"v02")[..]);
        builder.add(&full_key(b"k3", 3), &get_hummock_new_value(b"v03")[..]);
        builder.add(&full_key(b"k4", 4), &get_hummock_new_value(b"v04")[..]);
       
        let buf = builder.build().1.to_vec();
        let capcitiy = builder.uncompressed_block_size();
        let block = Box::new(Block::decode(buf.into(), capcitiy).unwrap());
        let mut bi = BlockIterator::new(BlockHolder::from_owned_block(block));

        bi.seek_to_first();
        assert!(bi.is_valid());
        assert_eq!(&full_key(b"k1", 1)[..], bi.key());
        assert_eq!(get_hummock_new_value(b"v01"), bi.value().as_slice());

        bi.next();
        assert!(bi.is_valid());
        assert_eq!(&full_key(b"k2", 2)[..], bi.key());
        assert_eq!(get_hummock_new_value(b"v02"), bi.value().as_slice());

        bi.next();
        assert!(bi.is_valid());
        assert_eq!(&full_key(b"k3", 3)[..], bi.key());
        assert_eq!(get_hummock_new_value(b"v03"), bi.value().as_slice());

        bi.next();
        assert!(bi.is_valid());
        assert_eq!(&full_key(b"k4", 4)[..], bi.key());
        assert_eq!(get_hummock_new_value(b"v04"), bi.value().as_slice());

        bi.next();
        assert!(!bi.is_valid());
    }

    pub fn full_key(user_key: &[u8], epoch: u64) -> Bytes {
        let mut buf = BytesMut::with_capacity(user_key.len() + 8);
        buf.put_slice(user_key);
        buf.put_u64(!epoch);
        buf.freeze()
    }
}
