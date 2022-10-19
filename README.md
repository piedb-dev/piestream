
[![Slack](https://badgen.net/badge/Slack/Join%20piestream/0abd59?icon=slack)](https://join.slack.com/t/piestream-community/shared_invite/zt-120rft0mr-d8uGk3d~NZiZAQWPnElOfw)
[![Build status](https://badge.buildkite.com/9394d2bca0f87e2e97aa78b25f765c92d4207c0b65e7f6648f.svg)](https://buildkite.com/singularity-data/main)
[![codecov](https://codecov.io/gh/piestreamlabs/piestream/branch/main/graph/badge.svg?token=EB44K9K38B)](https://codecov.io/gh/piestreamlabs/piestream)

piestream is a cloud-native streaming database that uses SQL as the interface language. It is designed to reduce the complexity and cost of building real-time applications. piestream consumes streaming data, performs continuous queries, and updates results dynamically. As a database system, piestream maintains results inside its own storage and allows users to access data efficiently.

piestream ingests data from sources like Apache Kafka, Apache Pulsar, Amazon Kinesis, Redpanda, and materialized CDC sources.

Learn more at [Introduction to piestream](https://www.piestream.dev/docs/latest/intro/).

## Quick Start

### Installation

There are two ways to install piestream: use a pre-built package or compile from source.

**Use a Pre-built Package (Linux)**

```shell
# Download the pre-built binary
wget https://github.com/piestreamlabs/piestream/releases/download/v0.1.13/piestream-v0.1.13-x86_64-unknown-linux.tar.gz
# Unzip the binary
tar xvf piestream-v0.1.13-x86_64-unknown-linux.tar.gz
# Start piestream in single-binary playground mode
./piestream playground
```

**Use Docker (Linux, macOS)**

```shell
# Start piestream in single-binary playground mode
docker run -it --pull=always -p 4566:4566 -p 5691:5691 ghcr.io/piestreamlabs/piestream:v0.1.13 playground
```

**Compile from Source with [RiseDev](docs/developer-guide.md#set-up-the-development-environment) (Linux and macOS)**

```shell
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Clone the repo
git clone https://github.com/piestreamlabs/piestream.git && cd piestream
# Compile and start the playground
./risedev playground
```

To build from source, you need to pre-install several tools in your system. You may use `./risedev configure` to configure compile settings. Please refer to the [developer guide](docs/developer-guide.md) for more information.

You can launch a piestream cluster and process streaming data in a distributed manner, and enable other features like metrics collection and data persistence. Please refer to the [developer guide](docs/developer-guide.md) for more information.

### Your First Query

To connect to the piestream server, you will need to [install PostgreSQL shell](docs/developer-guide.md#set-up-the-development-environment) (`psql`) in advance.

```shell
# Use psql to connect piestream cluster
psql -h localhost -p 4566 -d dev -U root
```

```sql
/* create a table */
create table t1(v1 int);

/* create a materialized view based on the previous table */
create materialized view mv1 as select sum(v1) as sum_v1 from t1;

/* insert some data into the source table */
insert into t1 values (1), (2), (3);

/* (optional) ensure the materialized view has been updated */
flush;

/* the materialized view should reflect the changes in source table */
select * from mv1;
```

If everything works correctly, you should see

```
 sum_v1
--------
      6
(1 row)
```

in the terminal.

### Connecting to an External Source

Please refer to [get started guide](https://www.piestream.dev/docs/latest/get-started/) for more information.

## Documentation

To learn about how to use piestream, refer to [piestream docs](https://www.piestream.dev/). To learn about the development process, see the [developer guide](docs/developer-guide.md). To understand the design and implementation of piestream, refer to the design docs listed in [readme.md](docs/README.md).

## License

piestream is under the Apache License 2.0. Please refer to [LICENSE](LICENSE) for more information.

## Contributing

Thanks for your interest in contributing to the project! Please refer to [contribution guidelines](CONTRIBUTING.md) for more information.
