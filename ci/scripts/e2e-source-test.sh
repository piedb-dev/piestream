#!/bin/bash

# Exits as soon as any line fails.
set -euo pipefail

source ci/scripts/common.env.sh

while getopts 'p:' opt; do
    case ${opt} in
        p )
            profile=$OPTARG
            ;;
        \? )
            echo "Invalid Option: -$OPTARG" 1>&2
            exit 1
            ;;
        : )
            echo "Invalid option: $OPTARG requires an argument" 1>&2
            ;;
    esac
done
shift $((OPTIND -1))

echo "--- Download artifacts"
mkdir -p target/debug
buildkite-agent artifact download piestream-"$profile" target/debug/
buildkite-agent artifact download risedev-dev-"$profile" target/debug/
mv target/debug/piestream-"$profile" target/debug/piestream
mv target/debug/risedev-dev-"$profile" target/debug/risedev-dev

echo "--- Download mise"
buildkite-agent artifact download avro-simple-schema.avsc ./
buildkite-agent artifact download avro-complex-schema.avsc ./
buildkite-agent artifact download proto-complex-schema ./

echo "--- Adjust permission"
chmod +x ./target/debug/piestream
chmod +x ./target/debug/risedev-dev

echo "--- Generate RiseDev CI config"
cp ci/risedev-components.ci.source.env risedev-components.user.env

echo "--- Prepare RiseDev dev cluster"
cargo make pre-start-dev
cargo make link-all-in-one-binaries

echo "--- e2e test w/ Rust frontend - source with kafka"
apt update
apt install -y kafkacat
cargo make clean-data
cargo make ci-start ci-kafka
./scripts/source/prepare_ci_kafka.sh
sqllogictest -p 5505 -d dev  './e2e_test/source/**/*.slt'

echo "--- Run CH-benCHmark"
./risedev slt -p 5505 -d dev ./e2e_test/ch-benchmark/ch_benchmark.slt
