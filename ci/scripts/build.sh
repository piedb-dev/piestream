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
            echo "Invalid option: $OPTARG requires an argument" 1>&2
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
cargo build \
    -p piestream_cmd_all \
    -p risedev \
    -p piestream_regress_test \
    -p piestream_sqlsmith \
    --features "static-link static-log-level" --profile "$profile"

echo "--- Compress piestream debug info"
objcopy --compress-debug-sections=zlib-gnu target/"$target"/piestream

echo "--- Show link info"
ldd target/"$target"/piestream

echo "--- Upload artifacts"
cp target/"$target"/piestream ./piestream-"$profile"
cp target/"$target"/risedev-dev ./risedev-dev-"$profile"
cp target/"$target"/piestream_regress_test ./piestream_regress_test-"$profile"
cp target/"$target"/sqlsmith ./sqlsmith-"$profile"
buildkite-agent artifact upload piestream-"$profile"
buildkite-agent artifact upload risedev-dev-"$profile"
buildkite-agent artifact upload piestream_regress_test-"$profile"
buildkite-agent artifact upload ./sqlsmith-"$profile"

echo "--- upload misc"
cp src/source/src/test_data/simple-schema.avsc ./avro-simple-schema.avsc
buildkite-agent artifact upload ./avro-simple-schema.avsc

cp src/source/src/test_data/complex-schema.avsc ./avro-complex-schema.avsc
buildkite-agent artifact upload ./avro-complex-schema.avsc

cp src/source/src/test_data/complex-schema ./proto-complex-schema
buildkite-agent artifact upload ./proto-complex-schema
