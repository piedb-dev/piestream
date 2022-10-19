# Sqlsmith

SqlSmith is currently used as a testing tool to discover unexpected panics in piestream (It's not designed to generally test every SQL database, as it also tests some special SQL syntax used in piestream). It always generates the correct SQL based on the feature set supported so far. Therefore, if a test fails, it can only be due to two causes:

1. There's a bug in SQLSmith, as it generates invalid SQL. 
2. There's a bug in piestream because it's unable to handle a correct query.

## Frontend

SqlSmith has two modes. The first one focuses on testing the frontend, i.e, testing the functionalities of SQL compilation (binding, transforming an AST into a logical plan, transforming a logical plan into a physical plan).

This test will be run as a unit test:

``` sh
./risedev test -E "package(piestream_sqlsmith)" --features enable_sqlsmith_unit_test
```

## E2E

In the second mode, it will test the entire query handling end-to-end. We provide a CLI tool that represents a Postgres client. You can run this tool via:

```sh
cargo build # Ensure CLI tool is up to date
./risedev d # Start cluster
./target/debug/sqlsmith test --testdata ./src/tests/sqlsmith/tests/testdata
```

Additionally, in some cases where you may want to debug whether we have defined some function/operator incorrectly,
you can try:

```sh
cargo build
./target/debug/sqlsmith print-function-table > ft.txt
```

Check out ft.txt that will contain all the function signatures.
