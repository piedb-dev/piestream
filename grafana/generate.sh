#!/bin/bash

set -euo pipefail

generate-dashboard -o piestream-dashboard.gen.json piestream-dashboard.py
jq -c . piestream-dashboard.gen.json > piestream-dashboard.json
