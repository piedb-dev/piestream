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
use std::collections::hash_map::Entry;
use std::collections::HashMap;
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
    fn get_decompression_algorithm(data_type_value: u8)->u8{
        data_type_value as u8
    }

    pub fn decompression(buf: &[u8], data_type_value: u8)->Vec<u8>{
        let algorithm=Compression::get_decompression_algorithm(data_type_value);
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
    pub entry_count: u16,
    pub vaild_entry_count: u16,
    pub column_count: u16,
    pub variable_column_count: u16,
    pub key_len: u16,
    pub data_type_values: Arc<Vec<(u8,usize)>>,
    pub offsets: Arc<Vec<u8>>,
    pub lens: Arc<Vec<u8>>,
    pub vec_text: Arc<Vec<Vec<u8>>>,
    pub states: Arc<Vec<u8>>,
    pub keys: Arc<Vec<u8>>,
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
        Ok(Self::decode_from_raw(buf))
    }

    pub fn decode_from_raw(data: Bytes) -> Self {
        let buf=data.clone();
        let mut offset=0;
        let entry_count=(&buf[buf.len()-offset-2..]).get_u16_le();
        offset+=2;
        let vaild_entry_count=(&buf[buf.len()-offset-2..]).get_u16_le();
        offset+=2;
        let column_count=(&buf[buf.len()-offset-2..]).get_u16_le();
        offset+=2;
        /*let variable_column_count=(&buf[buf.len()-offset-2..]).get_u16_le();
        offset+=2;*/

        let mut variable_column_count=0;
        let column_count_usize=column_count as usize;
        //let mut data_type_values= vec![(0_u8,0_usize); column_count_usize];
        let mut data_type_values= Vec::with_capacity(column_count_usize);
        let mut ptr=&buf[buf.len()-offset-column_count_usize..];
        for _ in  0..column_count{
            let v=ptr.get_u8();
            if let None=value_to_type(v){
                variable_column_count+=1;
            }
            data_type_values.push((v, index_to_len(v)));
            //data_type_values[i as usize]=(ptr.get_u8(),index_to_len(v));
        }
        //println!("data_type_values={:?}", data_type_values);
        offset+=column_count_usize;
        let key_len=(&buf[buf.len()-offset-2..]).get_u16_le();
        offset+=2;
        let keys_compress_len=(&buf[buf.len()-offset-4..]).get_u32_le() as usize;
        offset+=4;
        let states_len=(&buf[buf.len()-offset-4..]).get_u32_le() as usize;
        offset+=4;
        let offset_len=(&buf[buf.len()-offset-4..]).get_u32_le();
        offset+=4;
        let text_len=(&buf[buf.len()-offset-4..]).get_u32_le();
        offset+=4;
        //offset of per column
        let mut offsets=&buf[buf.len()-offset-offset_len as usize..buf.len()-offset];
        offset+= offset_len as usize;

        //println!("column_count={:?} variable_column_count={:?}", column_count, variable_column_count);
        assert_eq!(offsets.len(), ((column_count+variable_column_count)*4) as usize);
        //len of per column
        let mut lens=&buf[buf.len()-offset-text_len as usize..buf.len()-offset];
        //offset+= text_len as usize;
        assert_eq!(offsets.len(), lens.len());

        let mut next_start=0;
        //column text
        let mut vec_text=vec![vec![]; (column_count+variable_column_count) as usize];
        //let variable_column_type=vec![8_u8, 13, 14];
        let mut vec_variable_column_idx=vec![0; variable_column_count as usize];
        for i in 0..(column_count+variable_column_count){
            let a=offsets.get_u32_le() as usize;
            let b=lens.get_u32_le() as usize ;
            
            if i==0{
                next_start=a;
            }

            let mut data_type_value=0_u8;
            if i<column_count{
                data_type_value=data_type_values[i as usize].0 as u8;
                if let None=value_to_type(data_type_value){
                    //save variable column index
                    vec_variable_column_idx.push(i);
                    //offset is u32
                    data_type_value+=2_u8;
                }
            }else{
                    //variable column text
                    data_type_value+=8_u8;
            }
            //variable column content
            let buffer=Compression::decompression(&buf[a..a+b], data_type_value);
            vec_text[i as usize].extend_from_slice(buffer.as_slice());
        }

        let mut states=vec![];
        let buffer=Compression::decompression(&buf[(next_start-states_len)..next_start], 2_u8);
        assert_eq!((next_start-states_len), keys_compress_len);
        states.extend_from_slice(buffer.as_slice());
        
        let mut keys=vec![];
        let buffer=Compression::decompression(&buf[0..keys_compress_len], 8_u8);
        keys.extend_from_slice(buffer.as_slice());

        Block{
            data_len:data.len(),
            data:data,
            entry_count: entry_count,
            vaild_entry_count: vaild_entry_count,
            column_count: column_count,
            variable_column_count: variable_column_count,
            key_len:key_len ,
            data_type_values: Arc::new(data_type_values),
            offsets: Arc::new(offsets.to_vec()),
            lens: Arc::new(lens.to_vec()),
            vec_text:Arc::new(vec_text),
            states: Arc::new(states),
            keys: Arc::new(keys),
        }
    }
    
    pub fn entry_count(&self) -> usize{
        self.entry_count as usize
    }
   
    pub fn capacity(&self) -> usize {
        let mut column_len=0;
        for i in 0..self.vec_text.len(){
            column_len+=self.vec_text[i].capacity();
        }
        self.data_type_values.capacity() + self.offsets.capacity() +self.lens.capacity() + column_len + self.states.capacity() + self.keys.capacity() +
             2 + 2 + 2 + 2 + 2
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
    map_data_type: HashMap<u32, Vec<u32>>,
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
    hummock_value_list: Vec<u32>,
    //put_record_idxs: Vec<u16>,
    keys: Vec<Vec<u8>>,
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
            map_data_type: HashMap::new(),
            current_mem_size: 0,
            uncompressed_block_size: 0,
            compression_algorithm: options.compression_algorithm,
            row_deserializer: None,
            data_type_bit_size: DEFAULT_DATA_TYPE_BIT_SIZE,
            hummock_value_list: Vec::new(),
            //put_record_idxs: Vec::new(),
            keys: Vec::new(),
            rows: Vec::new(),

            //table_column_datatype:Vec::new(),
        }
    }

    pub fn set_row_deserializer(&mut self, table_column_datatype: Vec<DataType>){
        //println!("table_column_datatype={:?}", table_column_datatype);
        self.variable_columns.clear();
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
                .entry(data_types[idx].type_to_fixed_index() as u32)
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
        //println!("*****************add value={:?}*******************", &value);
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
        let idx=self.entry_count % self.data_type_bit_size as usize;
        if idx==0{
            self.hummock_value_list.push(0_u32);
        }

        let mut key_vec=key.to_vec();
        if is_put {
            let row = self.row_deserializer.as_ref().unwrap().deserialize(buffer).unwrap();
            //println!("row={:?}", row);
            self.rows.push(row.clone());
            let v=self.hummock_value_list.last_mut().unwrap();
            *v|=1<<idx;
            //println!("v={:?}", v);
            //self.put_record_idxs.push(self.put_entry_count as u16);
            //self.put_entry_count+=1;
            key_vec.push(VALUE_PUT as u8);
            key_vec.append(&mut (self.put_entry_count as u16).to_le_bytes().to_vec());
            self.put_entry_count+=1;
            self.current_mem_size+=key_vec.len()+value_len+self.variable_columns.len()*4;
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
            key_vec.push(VALUE_DELETE as u8);
            key_vec.append(&mut u16::MAX.to_le_bytes().to_vec());
            //self.put_record_idxs.push(u16::MAX);
            self.current_mem_size+=key_vec.len()+value_len;
        }

        //self.keys.push(key.to_vec());
        self.keys.push(key_vec);
        self.last_key.clear();
        self.last_key.extend_from_slice(key);
        //self.current_mem_size+=(key.len()+value.len()+1+2+2+4);
        self.entry_count += 1;
    }

    
    pub fn build(&mut self) -> (u32, &[u8]) {
        //println!("*****************build*******************");
        let data_types=self.row_deserializer.as_ref().unwrap().data_types();
        let data_chunk=DataChunk::from_rows(&self.rows, data_types);
        //column_value_state_list saves the field value state 
        //println!("data_chunk={:?}", data_chunk);
        let mut buffers = vec![vec![]; data_types.len()];
        let mut column_value_state_list = vec![vec![]; data_types.len()];
        let mut variable_offsets = vec![vec![]; data_types.len()];
        data_chunk.serialize_columns(&mut buffers, &mut column_value_state_list, &mut variable_offsets);
        //let (column_value_state_list, columns, variable_offsets)=data_chunk.serialize_columns();

        let columns=&buffers;
        //println!("column_value_state_list={:?}", column_value_state_list);
        //println!("columns={:?}", columns);
        //println!("variable_offsets={:?}", variable_offsets);
      
        //assert_eq!(columns.len(), data_types.len());
        //assert_eq!(data_types.len(), column_value_state_list.len());

        let size=(self.put_entry_count + (self.data_type_bit_size as usize -1)) / self.data_type_bit_size as usize;
        let mut state_list =vec![vec![]; column_value_state_list.len()];
        for (pos, column_state_list) in column_value_state_list.iter().enumerate(){
            assert_eq!(self.put_entry_count, column_state_list.len());
            for (idx,element) in column_state_list.iter().enumerate(){
                if idx % self.data_type_bit_size as usize==0{
                    state_list[pos].push(0_u32);
                }
                if *element==1 {
                    let v=state_list[pos].last_mut().unwrap();
                    *v|=1<<(idx%self.data_type_bit_size as usize);
                }
            }
            assert_eq!(state_list[pos].len(), size);
        }
       
        //saved keys info
        for idx in 0.. self.keys.len() {
            //println!("keys.len={:?}", &self.keys[idx].len());
            self.buf.extend_from_slice(&self.keys[idx]);
        }
        let keys_compress_len=self.buf.len();
        //println!("keys_compress_len={:?}", keys_compress_len);

        //saved value state (whether None)
        for idx in 0..state_list.len() {
            for v in &state_list[idx]{
                self.buf.extend_from_slice(&v.to_le_bytes());
            }
        }
        let states_len=self.buf.len()-keys_compress_len;
        //println!("states_len={:?} buf.len={:?}", keys_compress_len, self.buf.len());

        let mut start_offset=self.buf.len();
        //let mut vec=vec![(0,0); columns.len()];
        //let mut offsets=vec![0_u8; 4*(columns.len()+self.variable_columns.len())];
        //let mut lens=vec![0_u8; 4*(columns.len()+self.variable_columns.len())];
        let mut offsets=Vec::with_capacity(4*(columns.len()+self.variable_columns.len()) as usize);
        let mut lens=Vec::with_capacity(4*(columns.len()+self.variable_columns.len()) as usize);
        //let data_types=self.row_deserializer.as_ref().unwrap().data_types();
        //assert_eq!(columns.len(), data_types.len());

        let mut variable_column_num=0;
        for idx in 0..columns.len(){
            //fixed len columns
            if data_types[idx].data_type_len()>0{
                self.buf.extend_from_slice(&columns[idx]);
                assert_eq!(columns[idx].len(), self.put_entry_count*data_types[idx].data_type_len());
                //assert_eq!(columns[idx].len()%data_types[idx].data_type_len(), 0);
            }else{
                assert_eq!(variable_offsets[idx].len(), self.put_entry_count*4);
                //offset
                self.buf.extend_from_slice(&variable_offsets[idx]);
                variable_column_num+=1;
            }
            //vec.push((start_offset, self.buf.len()-start_offset));
            //println!("start_offset={:?}, buf.len={:?}", start_offset, self.buf.len());
            offsets.extend_from_slice(&(start_offset as u32).to_le_bytes());
            lens.extend_from_slice(&((self.buf.len()-start_offset) as u32).to_le_bytes());
            start_offset=self.buf.len();
        }

        assert_eq!(self.variable_columns.len(), variable_column_num);
        for (_, &idx) in self.variable_columns.iter().enumerate() {
            //fixed len columns
            if data_types[idx].data_type_len()==0{
                self.buf.extend_from_slice(&columns[idx]);
            }
            //vec.push((start_offset, self.buf.len()-start_offset));
            offsets.extend_from_slice(&(start_offset as u32).to_le_bytes());
            lens.extend_from_slice(&((self.buf.len()-start_offset) as u32).to_le_bytes());
            start_offset=self.buf.len();
        }

        //println!("lens.len={:?}", lens.len());
        //println!("offsets.len={:?}", offsets.len());
        self.buf.extend_from_slice(&lens);
        self.buf.extend_from_slice(&offsets);
        self.buf.put_u32_le(lens.len() as u32);
        self.buf.put_u32_le(offsets.len() as u32);
        //value state len 
        self.buf.put_u32_le(states_len as u32);
        //compress keys len 
        self.buf.put_u32_le(keys_compress_len as u32);
        //key length
        self.buf.put_u16_le((self.last_key.len() as u16 +3) as u16);
       
        //column type
        for data_type in data_types {
            //println!("data_type.type_to_index={:?}", data_type.type_to_index());
            self.buf.put_u8(data_type.type_to_index());
        }
        //variable column count
        //self.buf.put_u16_le(self.variable_columns.len() as u16);
        //column count
        self.buf.put_u16_le(data_types.len() as u16);
        //row count of not delete
        self.buf.put_u16_le(self.put_entry_count as u16);
        //total row count
        self.buf.put_u16_le(self.entry_count as u16);
        
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
        //println!("checksum={:?}", checksum);
        self.buf.put_u64_le(checksum);
        (self.uncompressed_block_size as u32, self.buf.as_ref())
        
    }

    pub fn get_last_key(&self) -> &[u8] {
        &self.last_key
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn clear(&mut self) {
        self.buf.clear();
        //self.restart_points.clear();
    
        self.last_key.clear();
        self.hummock_value_list.clear();
        self.keys.clear();
        self.rows.clear();
        //self.variable_columns.clear();
        self.put_entry_count=0;
        self.entry_count = 0;
        self.current_mem_size=0;
    
    }

    /// Calculate block size without compression.
    pub fn uncompressed_block_size(&mut self) -> usize {
        self.uncompressed_block_size
    }


    /// Approximate block len (uncompressed).
    pub fn approximate_len(&self) -> usize {
        let data_types=self.row_deserializer.as_ref().unwrap().data_types();
        self.current_mem_size+data_types.len()+self.hummock_value_list.capacity()+(self.column_num+self.variable_columns.len())*(4+4)+8+4+4+2+4+4+2+2+4*4
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
    fn test_block_enc_dec() {
        let options = BlockBuilderOptions::default();
        let mut builder = BlockBuilder::new(options);
        builder.set_row_deserializer(vec![DataType::Varchar]);
        builder.add(&full_key(b"k1", 1), &get_hummock_new_value(b"v01")[..]);
        builder.add(&full_key(b"k2", 2), &get_hummock_new_value(b"v02")[..]);
        builder.add(&full_key(b"k3", 3), &get_hummock_new_value(b"v03")[..]);
        builder.add(&full_key(b"k4", 4), &get_hummock_new_value(b"v04")[..]);
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
