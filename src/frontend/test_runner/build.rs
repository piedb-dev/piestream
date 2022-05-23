// Copyright 2022 Singularity Data
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

// Copyright 2022 RisingLight Project Authors. Licensed under Apache-2.0.

use std::ffi::OsStr;
use std::io::Write;

use walkdir::WalkDir;

fn main() -> anyhow::Result<()> {
    // Scan test scripts and generate test cases.
    println!("cargo:rerun-if-changed=tests/testdata");

    let mut f = std::fs::File::create("tests/gen/testcases.rs").expect("failed to create file");
    let mut testcase_found = false;

    for entry in WalkDir::new("tests/testdata") {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if (path.extension() == Some(OsStr::new("yml"))
            || path.extension() == Some(OsStr::new("yaml")))
            && !path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .contains(".apply.yaml")
        {
            let file_name = path.file_name().unwrap().to_string_lossy();
            let test_case_name = file_name.split('.').next().unwrap();

            writeln!(
                f,
                r#"
#[tokio::test]
async fn test_{test_case_name}() {{
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("testdata")
        .join("{file_name}");
    let file_content = tokio::fs::read_to_string(path).await.unwrap();
    risingwave_frontend_test_runner::run_test_file("{test_case_name}", &file_content)
        .await;
}}
                        "#
            )?;

            testcase_found = true;
        }
    }

    if !testcase_found {
        writeln!(
            f,
            r#"
#[tokio::test]
async fn test_not_found() {{
    panic!("no test case found in planner test!");
}}
                    "#
        )?;
    }

    Ok(())
}
