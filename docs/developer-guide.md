# Developer guide

This guide is intended to be used by contributors to learn about how to develop piestream. The instructions about how to submit code changes are included in [contributing guidelines](../CONTRIBUTING.md).

If you have questions, you can search for existing discussions or start a new discussion in the [Discussions forum of piestream](https://github.com/singularity-data/piestream/discussions), or ask in the piestream Community channel on Slack. Please use the [invitation link](https://join.slack.com/t/piestream-community/shared_invite/zt-120rft0mr-d8uGk3d~NZiZAQWPnElOfw) to join the channel.

To report bugs, create a [GitHub issue](https://github.com/singularity-data/piestream/issues/new/choose).


## Table of contents

- [Developer guide](#developer-guide)
  - [Table of contents](#table-of-contents)
  - [Read the design docs](#read-the-design-docs)
  - [Learn about the code structure](#learn-about-the-code-structure)
  - [Set up the development environment](#set-up-the-development-environment)
  - [Start and monitor a dev cluster](#start-and-monitor-a-dev-cluster)
    - [Configure additional components](#configure-additional-components)
    - [Start the playground with RiseDev](#start-the-playground-with-risedev)
    - [Start the playground with cargo](#start-the-playground-with-cargo)
  - [Develop the dashboard](#develop-the-dashboard)
    - [Dashboard v1](#dashboard-v1)
    - [Dashboard v2](#dashboard-v2)
  - [Observability components](#observability-components)
    - [Cluster Control](#cluster-control)
    - [Monitoring](#monitoring)
    - [Tracing](#tracing)
    - [Dashboard](#dashboard)
    - [Logging](#logging)
  - [Test your code changes](#test-your-code-changes)
    - [Lint](#lint)
    - [Unit tests](#unit-tests)
    - [Planner tests](#planner-tests)
    - [End-to-end tests](#end-to-end-tests)
    - [End-to-end tests on CI](#end-to-end-tests-on-ci)
  - [Miscellaneous checks](#miscellaneous-checks)
  - [Update Grafana dashboard](#update-grafana-dashboard)
  - [Add new files](#add-new-files)
  - [Add new dependencies](#add-new-dependencies)
  - [Check in PRs from forks](#check-in-prs-from-forks)
  - [Submit PRs](#submit-prs)


## Read the design docs

Before you start to make code changes, ensure that you understand the design and implementation of piestream. We recommend that you read the design docs listed in [docs/README.md](README.md) first.

## Learn about the code structure

- The `src` folder contains all of the kernel components, refer to [src/README.md](../src/README.md) for more details.
- The `docker` folder contains Docker files to build and start piestream.
- The `e2e_test` folder contains the latest end-to-end test cases.
- The `docs` folder contains the design docs. If you want to learn about how piestream is designed and implemented, check out the design docs here.
- The `dashboard` folder contains piestream dashboard v2.

## Set up the development environment

RiseDev is the development mode of piestream. To develop piestream, you need to build from the source code and run RiseDev. RiseDev can be built on macOS and Linux. It has the following dependencies:

* Rust toolchain
* CMake
* protobuf
* OpenSSL
* PostgreSQL (psql) (>= 14.1)
* Tmux

To install the dependencies on macOS, run:

```shell
brew install postgresql cmake protobuf openssl tmux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

To install the dependencies on Debian-based Linux systems, run:

```shell
sudo apt install make build-essential cmake protobuf-compiler curl openssl libssl-dev libcurl4-openssl-dev pkg-config postgresql-client tmux lld
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then you'll be able to compile and start RiseDev!

## Start and monitor a dev cluster

You can now build RiseDev and start a dev cluster. It is as simple as:

```shell
./risedev d                        # shortcut for ./risedev dev
psql -h localhost -p 4566 -d dev -U root
```

The default dev cluster includes metadata-node, compute-node and frontend-node processes, and an embedded volatile in-memory state storage. No data will be persisted. This configuration is intended to make it easier to develop and debug piestream.

To stop the cluster:

```shell
./risedev k # shortcut for ./risedev kill
```

To view the logs:

```shell
./risedev l # shortcut for ./risedev logs
```

To clean local data and logs:

```shell
./risedev clean-data
```

### Configure additional components

There are a few components that you can configure in RiseDev.

Use the `./risedev configure` command to start the interactive configuration mode, in which you can enable and disable components.

- Hummock (MinIO + MinIO-CLI): Enable this component to persist state data.
- Prometheus and Grafana: Enable this component to view piestream metrics. You can view the metrics through a built-in Grafana dashboard.
- Etcd: Enable this component if you want to persist metadata node data.
- Kafka: Enable this component if you want to create a streaming source from a Kafka topic.
- Jaeger: Use this component for tracing.

To manually add those components into the cluster, you will need to configure RiseDev to download them first. For example,

```shell
./risedev configure enable prometheus-and-grafana # enable Prometheus and Grafana
./risedev configure enable minio                  # enable MinIO
```
**Note**: Enabling a component with the `./risedev configure enable` command will only download the component to your environment. To allow it to function, you must revise the corresponding configuration setting in `risedev.yml` and restart the dev cluster.

For example, you can modify the default section to:

```yaml
  default:
    - use: minio
    - use: meta-node
      enable-dashboard-v2: false
    - use: compute-node
    - use: frontend
    - use: prometheus
    - use: grafana
    - use: zookeeper
      persist-data: true
    - use: kafka
      persist-data: true
```

**Note**: The Kafka service depends on the ZooKeeper service. If you want to enable the Kafka component, enable the ZooKeeper component first.

Now you can run `./risedev d` to start a new dev cluster. The new dev cluster will contain components as configured in the yaml file. RiseDev will automatically configure the components to use the available storage service and to monitor the target.

You may also add multiple compute nodes in the cluster. The `ci-3cn-1fe` config is an example.

### Start the playground with RiseDev

If you do not need to start a full cluster to develop, you can issue `./risedev p` to start the playground, where the metadata node, compute nodes and frontend nodes are running in the same process. Logs are printed to stdout instead of separate log files.

```shell
./risedev p # shortcut for ./risedev playground
```

For more information, refer to `README.md` under `src/risedevtool`.

### Start the playground with cargo

To start the playground (all-in-one process) from IDE or command line, you can use:

```shell
cargo run --bin piestream -- playground
```

Then, connect to the playground instance via:

```shell
psql -h localhost -p 4566 -d dev -U root
```

## Develop the dashboard

Currently, piestream has two versions of dashboards. You can use RiseDev config to select which version to use.

The dashboard will be available at `http://127.0.0.1:5691/` on meta node.

### Dashboard v1

Dashboard v1 is a single HTML page. To preview and develop this version, install Node.js, and run this command:

```shell
cd src/meta/src/dashboard && npx reload -b
```

Dashboard v1 is bundled by default along with meta node. When the cluster is started, you may use the dashboard without any configuration.

### Dashboard v2

The development instructions for dashboard v2 are available [here](../dashboard/README.md).

## Observability components

RiseDev supports several observability components.

### Cluster Control

`risectl` is the tool for providing internal access to the piestream cluster. See

```
cargo run --bin risectl -- --help
```

... or

```
./piestream risectl --help
```

for more information.

### Monitoring

Uncomment `grafana` and `prometheus` lines in `risedev.yml` to enable Grafana and Prometheus services. 

### Tracing

Compute nodes support streaming tracing. Tracing is not enabled by default. You need to
use `./risedev configure` to download the tracing components first. After that, you will need to uncomment `jaeger`
service in `risedev.yml` and start a new dev cluster to allow the components to work.

### Dashboard

You may use piestream Dashboard to see actors in the system. It will be started along with meta node.

### Logging

The Rust components use `tokio-tracing` to handle both logging and tracing. The default log level is set as:

* Third-party libraries: warn
* Other libraries: debug

If you need to adjust log levels, change the logging filters in `utils/runtime/lib.rs`.


## Test your code changes

Before you submit a PR, fully test the code changes and perform necessary checks.

The piestream project enforces several checks in CI. Every time the code is modified, you need to perform the checks and ensure they pass.

### Lint

piestream requires all code to pass fmt, clippy, sort and hakari checks. Run the following commands to install test tools and perform these checks.

```shell
./risedev install-tools # Install required tools for running unit tests
./risedev c             # Run all checks. Shortcut for ./risedev check
```

### Unit tests

RiseDev runs unit tests with cargo-nextest. To run unit tests:

```shell
./risedev install-tools # Install required tools for running unit tests
./risedev test          # Run unit tests
```

If you want to see the coverage report, run this command:

```shell
./risedev test-cov
```

### Planner tests

piestream's SQL frontend has SQL planner tests. For more information, see [Planner Test Guide](../src/frontend/test_runner/README.md).

### End-to-end tests

Use [sqllogictest-rs](https://github.com/risinglightdb/sqllogictest-rs) to run piestream e2e tests.

sqllogictest installation is included when you install test tools with the `./risedev install-tools` command. You may also install it with:

```shell
cargo install --git https://github.com/risinglightdb/sqllogictest-rs --features bin
```

Before running end-to-end tests, you will need to start a full cluster first:

```shell
./risedev d
```

Then run the end-to-end tests (replace `**/*.slt` with the test case directories and files available):

```shell
./risedev slt -p 4566 -d dev  './e2e_test/streaming/**/*.slt'
```

After running e2e tests, you may kill the cluster and clean data.

```shell
./risedev k  # shortcut for ./risedev kill
./risedev clean-data
```

piestream's codebase is constantly changing. The persistent data might not be stable. In case of unexpected decode errors, try `./risedev clean-data` first.

### End-to-end tests on CI

Basically, CI is using the following two configurations to run the full e2e test suite:

```shell
./risedev dev ci-3cn-1fe
```

You can adjust the environment variable to enable some specific code to make all e2e tests pass. Refer to GitHub Action workflow for more information.

## Miscellaneous checks

For shell code, please run:

```shell
brew install shellcheck
shellcheck <new file>
```

For Protobufs, we rely on [buf](https://docs.buf.build/installation) for code formatting and linting. Please check out their documents for installation. To check if you violate the rules, please run the commands:

```shell
buf format -d --exit-code
buf lint
```

## Update Grafana dashboard

See [README](../grafana/README.md) for more information.

## Add new files

We use [skywalking-eyes](https://github.com/apache/skywalking-eyes) to manage license headers.
If you added new files, please follow the installation guide and run:

```shell
license-eye -c .licenserc.yaml header fix
```

## Add new dependencies

To avoid rebuild some common dependencies across different crates in workspace, use
[cargo-hakari](https://docs.rs/cargo-hakari/latest/cargo_hakari/) to ensure all dependencies
are built with the same feature set across workspace. You'll need to run `cargo hakari generate`
after deps get updated.

Use [cargo-udeps](https://github.com/est31/cargo-udeps) to find unused dependencies in
workspace.

And use [cargo-sort](https://crates.io/crates/cargo-sort) to ensure all deps are get sorted.

## Submit PRs

Instructions about submitting PRs are included in the [contribution guidelines](../CONTRIBUTING.md).
