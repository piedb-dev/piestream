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

echo "--- Adjust permission"
chmod +x ./target/debug/piestream
chmod +x ./target/debug/risedev-dev

echo "--- Generate RiseDev CI config"
cp ci/risedev-components.ci.env risedev-components.user.env

echo "--- Prepare RiseDev dev cluster"
cargo make pre-start-dev
cargo make link-all-in-one-binaries

host_args="-h localhost -p 4565 -h localhost -p 5505 -h localhost -p 4567"

echo "--- e2e, ci-3cn-3fe-in-memory, streaming"
cargo make ci-start ci-3cn-3fe-in-memory
sqllogictest ${host_args} -d dev  './e2e_test/streaming/**/*.slt' -j 16 --junit "parallel-in-memory-streaming-${profile}"

echo "--- Kill cluster"
cargo make ci-kill

echo "--- e2e, ci-3cn-3fe-in-memory, batch"
cargo make ci-start ci-3cn-3fe-in-memory
sqllogictest ${host_args} -d dev  './e2e_test/ddl/**/*.slt' --junit "parallel-in-memory-batch-ddl-${profile}"
sqllogictest ${host_args} -d dev  './e2e_test/batch/**/*.slt' -j 16 --junit "parallel-in-memory-batch-${profile}"

echo "--- Kill cluster"
cargo make ci-kill
