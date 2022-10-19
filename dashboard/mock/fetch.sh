#!/bin/bash

set -e

curl http://localhost:5691/api/actors > actors.json
curl http://localhost:5691/api/clusters/0 > cluster_0.json
curl http://localhost:5691/api/clusters/1 > cluster_1.json
curl http://localhost:5691/api/clusters/2 > cluster_2.json
curl http://localhost:5691/api/fragments > fragments.json
curl http://localhost:5691/api/fragments2 > fragments2.json
curl http://localhost:5691/api/materialized_views > materialized_views.json
curl http://localhost:5691/api/sources > sources.json
