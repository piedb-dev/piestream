# Planner Test

This module contains a testing tool for binder, planner and optimizer.
Given a sequence of SQL queries as the input, the test runner will check
the logical operator tree if any, and the physical operator tree if any.

The test data in YAML format is organized under `tests/testdata` folder.

## Examples of Test Cases

### SELECT as the test case

You can simply write a `SELECT` query in the `sql` field, and using other fields, including `logical_plan` , `stream_plan` , `binder_error` , etc. for the plan under different situations.

```yaml
- sql: |
    select * from t
  binder_error: "Item not found: relation \"t\""
```

This is a simple test case that validates the binder's behavior on an illegal SQL.

```yaml
- sql: |
    create table t (v1 bigint, v2 double precision);
    select * from t;
  logical_plan: |
    LogicalProject { exprs: [$0, $1, $2], expr_alias: [None, None, None] }
      LogicalScan { table: "t", columns: ["_row_id", "v1", "v2"] }
```

If the SQL is valid, then test runner will compare the generated logical operator tree
with the expected tree.

### EXPLAIN as the test case

Alternatively, you can also write an `EXPLAIN` statement in the `sql` field. In this case, the output field is a single `explain_output` .

```yaml
- sql: explain select 1;
  explain_output: |
    BatchProject { exprs: [1:Int32] }
    └─BatchValues { rows: [[]] }
```

This is helpful when you want to test `EXPLAIN CREATE ...` and `EXPLAIN (options) ...` statements.

## Update Plans

Firstly, we will need to create a placeholder in yaml testcases:

```yaml
- sql: |
    create table t1 (v1 int, v2 int);
    create table t2 (v1 int, v2 int);
    create table t3 (v1 int, v2 int);
    select * from t1 join t2 on (t1.v1 = t2.v1) join t3 on (t2.v2 = t3.v2);
  logical_plan: ""
  batch_plan: ""
  stream_plan: ""
```

Those plans followed the input SQL are expected outputs.

```
./risedev apply-planner-test
```

Then we can find the updated tests at `*.apply.yaml` . If everything is okay, you may run:

```
./risedev do-apply-planner-test
```

To apply the new test results.

You may use the `before` key to include other testcases by `id` .

## Run a single test

```
./risedev run-planner-test <yaml file name>
./risedev run-planner-test tpch # Run tpch.yaml
./risedev run-planner-test # Run all tests
```
