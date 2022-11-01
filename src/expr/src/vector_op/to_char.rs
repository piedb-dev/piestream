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

use aho_corasick::AhoCorasickBuilder;
use piestream_common::array::{BytesGuard, BytesWriter};
use piestream_common::types::NaiveDateTimeWrapper;

use crate::Result;

/// Compile the pg pattern to chrono pattern.
// TODO: Chrono can not fully support the pg format, so consider using other implementations later.
pub fn compile_pattern_to_chrono(tmpl: &str) -> String {
    // https://www.postgresql.org/docs/current/functions-formatting.html
    static PG_PATTERNS: &[&str] = &[
        "HH24", "hh24", "HH12", "hh12", "HH", "hh", "MI", "mi", "SS", "ss", "YYYY", "yyyy", "YY",
        "yy", "IYYY", "iyyy", "IY", "iy", "MM", "mm", "DD", "dd",
    ];
    // https://docs.rs/chrono/latest/chrono/format/strftime/index.html
    static CHRONO_PATTERNS: &[&str] = &[
        "%H", "%H", "%I", "%I", "%I", "%I", "%M", "%M", "%S", "%S", "%Y", "%Y", "%y", "%y", "%G",
        "%G", "%g", "%g", "%m", "%m", "%d", "%d",
    ];

    let ac = AhoCorasickBuilder::new()
        .ascii_case_insensitive(false)
        .match_kind(aho_corasick::MatchKind::LeftmostLongest)
        .build(PG_PATTERNS);

    let mut chrono_tmpl = String::new();
    ac.replace_all_with(tmpl, &mut chrono_tmpl, |mat, _, dst| {
        dst.push_str(CHRONO_PATTERNS[mat.pattern()]);
        true
    });

    chrono_tmpl
}

pub fn to_char_timestamp(
    data: NaiveDateTimeWrapper,
    tmpl: &str,
    dst: BytesWriter,
) -> Result<BytesGuard> {
    let chrono_tmpl = compile_pattern_to_chrono(tmpl);
    let res = data.0.format(&chrono_tmpl).to_string();
    dst.write_ref(&res).map_err(Into::into)
}
