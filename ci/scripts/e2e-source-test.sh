#!/bin/bash

# Exits as soon as any line fails.
set -euo pipefail

source ci/scripts/common.env.sh

echo "--- Download artifacts"
mkdir -p target/debug
buildkite-agent artifact download piestream-dev target/debug/
buildkite-agent artifact download risedev-playground-dev target/debug/
buildkite-agent artifact download piestream_regress_test-dev target/debug/
mv target/debug/piestream-dev target/debug/piestream
mv target/debug/risedev-playground-dev target/debug/risedev-playground
mv target/debug/piestream_regress_test-dev target/debug/piestream_regress_test

echo "--- Adjust permission"
chmod +x ./target/debug/piestream
chmod +x ./target/debug/risedev-playground
chmod +x ./target/debug/piestream_regress_test

echo "--- Generate RiseDev CI config"
cp risedev-components.ci.env risedev-components.user.env

echo "--- Prepare RiseDev playground"
cargo make pre-start-playground
cargo make link-all-in-one-binaries

echo "--- e2e test w/ Rust frontend - source with kafka"
cargo make clean-data
cargo make ci-start ci-kafka
./scripts/source/prepare_ci_kafka.sh
timeout 2m sqllogictest -p 4566 -d dev  './e2e_test/source/**/*.slt'