#!/bin/bash

source ci/scripts/common.env.sh

echo "--- Download artifacts"
buildkite-agent artifact download piestream_simulation .
chmod +x ./piestream_simulation

export RUNNER=./piestream_simulation
export RUST_LOG=off

# bugs here! Tracking issue https://github.com/piestreamlabs/piestream/issues/4527
echo "--- deterministic simulation e2e, ci-3cn-1fe, recovery, streaming"
seq 1 | parallel MADSIM_TEST_SEED={} $RUNNER --kill-meta --kill-frontend --kill-compute './e2e_test/streaming/\*\*/\*.slt'

# bugs here! Tracking issue https://github.com/piestreamlabs/piestream/issues/4527
echo "--- deterministic simulation e2e, ci-3cn-1fe, recovery, batch"
seq 1 | parallel MADSIM_TEST_SEED={} $RUNNER --kill-meta --kill-frontend --kill-compute './e2e_test/batch/\*\*/\*.slt'

# bugs here! Tracking issue https://github.com/piestreamlabs/piestream/issues/5103
echo "--- deterministic simulation e2e, ci-3cn-1fe, recovery, streaming"
seq 1 | parallel MADSIM_TEST_SEED={} $RUNNER --etcd-timeout-rate=0.01 './e2e_test/streaming/\*\*/\*.slt'
