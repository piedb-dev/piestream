#!/bin/bash

# Exits as soon as any line fails.
set -euo pipefail

source ci/scripts/common.env.sh

while getopts 't:p:' opt; do
    case ${opt} in
        t )
            target=$OPTARG
            ;;
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

echo "--- Rust cargo-sort check"
cargo sort -c -w

echo "--- Rust cargo-hakari check"
cargo hakari verify

echo "--- Rust format check"
cargo fmt --all -- --check

echo "--- Build Rust components"
cargo build -p risingwave_cmd_all -p risedev -p risingwave_regress_test --features static-link --profile "$profile"

echo "--- Compress RisingWave debug info"
objcopy --compress-debug-sections=zlib-gnu target/"$target"/risingwave

echo "--- Show link info"
ldd target/"$target"/risingwave

echo "--- Upload artifacts"
cp target/"$target"/risingwave ./risingwave-"$profile"
cp target/"$target"/risedev-playground ./risedev-playground-"$profile"
cp target/"$target"/risingwave_regress_test ./risingwave_regress_test-"$profile"
buildkite-agent artifact upload risingwave-"$profile"
buildkite-agent artifact upload risedev-playground-"$profile"
buildkite-agent artifact upload risingwave_regress_test-"$profile"