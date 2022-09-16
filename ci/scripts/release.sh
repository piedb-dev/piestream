#!/bin/bash

# Exits as soon as any line fails.
set -euo pipefail

echo "--- Install gh cli"
curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | \
dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | \
tee /etc/apt/sources.list.d/github-cli.list > /dev/null
apt update -yy && apt install gh -yy

echo "--- Release create"
gh release create "${BUILDKITE_TAG}" --generate-notes -d -p

echo "--- Download artifacts"
mkdir -p target/debug && cd target/debug
buildkite-agent artifact download piestream-release .
mv piestream-release piestream
chmod +x piestream
tar -czvf piestream-"${BUILDKITE_TAG}"-x86_64-unknown-linux.tar.gz piestream

echo "--- Release upload asset"
gh release upload "${BUILDKITE_TAG}" piestream-"${BUILDKITE_TAG}"-x86_64-unknown-linux.tar.gz
