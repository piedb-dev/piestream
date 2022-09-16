#!/bin/bash

# Exits as soon as any line fails.
set -euo pipefail

source ci/scripts/common.env.sh

echo "+++ Running deterministic simulation test"
echo "$(tput setaf 3)This test won't compile because madsim doesn't support tokio::net yet. Tracking issue: https://github.com/singularity-data/piestream/issues/3467$(tput sgr0)"

echo "--- Generate RiseDev CI config"
cp risedev-components.ci.env risedev-components.user.env

echo "--- Run unit tests in deterministic simulation mode"
cargo make stest \
    --no-fail-fast \
    -p piestream_batch \
    -p piestream_common \
    -p piestream_compute \
    -p piestream_connector \
    -p piestream_ctl \
    -p piestream_expr \
    -p piestream_meta \
    -p piestream_source \
    -p piestream_storage \
    -p piestream_stream
