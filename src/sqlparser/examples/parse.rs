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

use std::io::BufRead;

use piestream_sqlparser::parser::*;

/// Input SQL, output AST.
fn main() {
    let mut sql = String::new();
    #[allow(clippy::significant_drop_in_scrutinee)]
    for line in std::io::stdin().lock().lines() {
        sql += &line.unwrap();
        if !sql.ends_with(';') {
            continue;
        }
        let ast = Parser::parse_sql(&sql).unwrap();
        println!("{:?}", ast);
    }
}
