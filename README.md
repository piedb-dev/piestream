![RisingWave Logo](./docs/images/logo-title.svg)

[![Slack](https://badgen.net/badge/Slack/Join%20RisingWave/0abd59?icon=slack)](https://join.slack.com/t/risingwave-community/shared_invite/zt-120rft0mr-d8uGk3d~NZiZAQWPnElOfw)
[![CI](https://github.com/singularity-data/risingwave/actions/workflows/main.yml/badge.svg)](https://github.com/singularity-data/risingwave/actions/workflows/main.yml)
[![codecov](https://codecov.io/gh/singularity-data/risingwave/branch/main/graph/badge.svg?token=EB44K9K38B)](https://codecov.io/gh/singularity-data/risingwave)

RisingWave is a cloud-native streaming database that uses SQL as the interface language. It is designed to reduce the complexity and cost of building real-time applications. RisingWave consumes streaming data, performs continuous queries, and updates results dynamically. As a database system, RisingWave maintains results inside its own storage and allows users to access data efficiently.

RisingWave ingests data from sources like Apache Kafka, Apache Pulsar, Amazon Kinesis, Redpanda, and materialized CDC sources.

Learn more at [Introduction to RisingWave](https://www.risingwave.dev/docs/latest/intro/).

## Quick Start

### Installation

There are two ways to install RisingWave: use a pre-built package or compile from source.

**Use a Pre-built Package (Linux)**

```shell
# Download the pre-built binary
wget https://github.com/singularity-data/risingwave/releases/download/v0.1.7/risingwave-v0.1.7-x86_64-unknown-linux.tar.gz
# Unzip the binary
tar xvf risingwave-v0.1.7-x86_64-unknown-linux.tar.gz
# Start RisingWave in single-binary playground mode
./risingwave playground
```

**Use Docker (Linux, macOS)**

Currently, only x86_64 is supported. We will provide arm64 builds in the future.

```shell
# Pull nightly build of RisingWave
docker pull ghcr.io/singularity-data/risingwave:latest
# Start RisingWave in single-binary playground mode
docker run -it --network host ghcr.io/singularity-data/risingwave:latest playground
```

**Compile from Source with [RiseDev](./CONTRIBUTING.md#setting-up-development-environment) (Linux and macOS)**

```shell
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Clone the repo
git clone https://github.com/singularity-data/risingwave.git && cd risingwave
# Compile and start the playground
./risedev playground
```

To build from source, you need to pre-install several tools in your system. You may use `./risedev configure` to configure compile settings. Please refer to [Contribution and Development Guidelines](CONTRIBUTING.md) for more information.

You may launch a RisingWave cluster and process streaming data in a distributed manner, or enable other features like metrics collection and data persistence. Please refer to [Contribution and Development Guidelines](CONTRIBUTING.md) for more information.

### Your First Query

To connect to the RisingWave server, you will need to [install PostgreSQL shell](./CONTRIBUTING.md#setting-up-development-environment) (`psql`) in advance.

```shell
# Use psql to connect RisingWave cluster
psql -h localhost -p 4566
```

```sql
/* create a table */
create table t1(v1 int not null);

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

Please refer to [getting started guide](https://www.risingwave.dev/docs/latest/getting-started/) for more information.

## Documentation

To learn about how to use RisingWave, refer to [RisingWave docs](https://www.risingwave.dev/). To learn about how we design and implement RisingWave, refer to [RisingWave developer docs](https://github.com/singularity-data/risingwave/tree/main/docs).

## License

RisingWave is under the Apache License 2.0. Please refer to [LICENSE](LICENSE) for more information.

## Contributing

Thanks for your interest in contributing to the project! Please refer to [Contribution and Development Guidelines](CONTRIBUTING.md) for more information.
