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

#[derive(Debug, Clone)]
pub struct PgFieldDescriptor {
    name: String,
    table_oid: i32,
    col_attr_num: i16,

    // NOTE: Static code for data type. To see the oid of a specific type in Postgres,
    // use the following command:
    //   SELECT oid FROM pg_type WHERE typname = 'int4';
    type_oid: TypeOid,

    type_len: i16,
    type_modifier: i32,
    format_code: i16,
}

impl PgFieldDescriptor {
    pub fn new(name: String, type_oid: TypeOid) -> Self {
        let type_modifier = -1;
        let format_code = 0;
        let table_oid = 0;
        let col_attr_num = 0;
        let type_len = match type_oid {
            TypeOid::Boolean => 1,
            TypeOid::Int | TypeOid::Float4 | TypeOid::Date => 4,
            TypeOid::BigInt
            | TypeOid::Float8
            | TypeOid::Timestamp
            | TypeOid::Time
            | TypeOid::Timestampz => 8,
            TypeOid::SmallInt => 2,
            TypeOid::Varchar | TypeOid::Decimal | TypeOid::Interval => -1,
        };

        Self {
            type_modifier,
            format_code,
            name,
            table_oid,
            col_attr_num,
            type_len,
            type_oid,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_table_oid(&self) -> i32 {
        self.table_oid
    }

    pub fn get_col_attr_num(&self) -> i16 {
        self.col_attr_num
    }

    pub fn get_type_oid(&self) -> TypeOid {
        self.type_oid
    }

    pub fn get_type_len(&self) -> i16 {
        self.type_len
    }

    pub fn get_type_modifier(&self) -> i32 {
        self.type_modifier
    }

    pub fn get_format_code(&self) -> i16 {
        self.format_code
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TypeOid {
    Boolean,
    BigInt,
    SmallInt,
    Int,
    Float4,
    Float8,
    Varchar,
    Date,
    Time,
    Timestamp,
    Timestampz,
    Decimal,
    Interval,
}

impl TypeOid {
    // TODO: support more type.
    pub fn as_type(oid: i32) -> Result<TypeOid, String> {
        match oid {
            1043 => Ok(TypeOid::Varchar),
            _ => todo!(),
        }
    }

    // NOTE:
    // Refer https://github.com/postgres/postgres/blob/master/src/include/catalog/pg_type.dat when add new TypeOid.
    // Be careful to distinguish oid from array_type_oid.
    // Such as:
    //  https://github.com/postgres/postgres/blob/master/src/include/catalog/pg_type.dat#L347
    //  For Numeric(aka Decimal): oid = 1700, array_type_oid = 1231
    pub fn as_number(&self) -> i32 {
        match self {
            TypeOid::Boolean => 16,
            TypeOid::BigInt => 20,
            TypeOid::SmallInt => 21,
            TypeOid::Int => 23,
            TypeOid::Float4 => 700,
            TypeOid::Float8 => 701,
            TypeOid::Varchar => 1043,
            TypeOid::Date => 1082,
            TypeOid::Time => 1083,
            TypeOid::Timestamp => 1114,
            TypeOid::Timestampz => 1184,
            TypeOid::Decimal => 1700,
            TypeOid::Interval => 1186,
        }
    }
}
