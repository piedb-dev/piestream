#!/bin/bash

# Exits as soon as any line fails.
set -euo pipefail

source ci/scripts/common.env.sh

echo "--- Generate RiseDev CI config"
cp ci/risedev-components.ci.env risedev-components.user.env

echo "--- Build deterministic simulation e2e test runner"
cargo make sslt --profile ci-sim -- --help

echo "--- Build and archive deterministic scaling imulation tests"
cargo make sarchive-scale-test --cargo-profile ci-sim

echo "--- Upload artifacts"
cp target/sim/ci-sim/piestream_simulation ./piestream_simulation
buildkite-agent artifact upload piestream_simulation
buildkite-agent artifact upload scale-test.tar.zst
