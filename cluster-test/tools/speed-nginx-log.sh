#!/bin/bash

cat t.log.3 | awk '{k = substr($4, 2); sum[k] += $10;} END {for (k in sum) { print k, sum[k]*8/1000, "Kbps"; }}' |sort
