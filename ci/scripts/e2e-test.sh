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
            echo "Invalid option: $OPTARG requires an arguemnt" 1>&2
            ;;
    esac
done
shift $((OPTIND -1))

echo "--- Download artifacts"
mkdir -p target/debug
buildkite-agent artifact download piestream-"$profile" target/debug/
buildkite-agent artifact download risedev-playground-"$profile" target/debug/
mv target/debug/piestream-"$profile" target/debug/piestream
mv target/debug/risedev-playground-"$profile" target/debug/risedev-playground

echo "--- Adjust permission"
chmod +x ./target/debug/piestream
chmod +x ./target/debug/risedev-playground

echo "--- Generate RiseDev CI config"
cp risedev-components.ci.env risedev-components.user.env

echo "--- Prepare RiseDev playground"
cargo make pre-start-playground
cargo make link-all-in-one-binaries

echo "--- e2e, ci-3cn-1fe, streaming"
cargo make ci-start ci-3cn-1fe
timeout 5m sqllogictest -p 4566 -d dev './e2e_test/streaming/**/*.slt' --junit "streaming-${profile}"

echo "--- Kill cluster"
cargo make ci-kill

echo "--- e2e, ci-3cn-1fe, delta join"
cargo make ci-start ci-3cn-1fe
timeout 3m sqllogictest -p 4566 -d dev './e2e_test/streaming_delta_join/**/*.slt' --junit "streaming-delta-join-${profile}"

echo "--- Kill cluster"
cargo make ci-kill

echo "--- e2e, ci-3cn-1fe, batch distributed"
cargo make ci-start ci-3cn-1fe
timeout 2m sqllogictest -p 4566 -d dev './e2e_test/ddl/**/*.slt' --junit "batch-ddl-${profile}"
timeout 2m sqllogictest -p 4566 -d dev './e2e_test/batch/**/*.slt' --junit "batch-${profile}"
timeout 2m sqllogictest -p 4566 -d dev './e2e_test/database/prepare.slt'
timeout 2m sqllogictest -p 4566 -d test './e2e_test/database/test.slt'

echo "--- Kill cluster"
cargo make ci-kill
