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
cargo build -p piestream_cmd_all -p risedev -p piestream_regress_test --features static-link --profile "$profile"

echo "--- Compress piestream debug info"
objcopy --compress-debug-sections=zlib-gnu target/"$target"/piestream

echo "--- Show link info"
ldd target/"$target"/piestream

echo "--- Upload artifacts"
cp target/"$target"/piestream ./piestream-"$profile"
cp target/"$target"/risedev-playground ./risedev-playground-"$profile"
cp target/"$target"/piestream_regress_test ./piestream_regress_test-"$profile"
buildkite-agent artifact upload piestream-"$profile"
buildkite-agent artifact upload risedev-playground-"$profile"
buildkite-agent artifact upload piestream_regress_test-"$profile"