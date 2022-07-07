#!/bin/bash

set -euo pipefail

generate-dashboard -o risingwave-dashboard.gen.json risingwave-dashboard.py
jq -c . risingwave-dashboard.gen.json > risingwave-dashboard.json
