#!/bin/bash

set -x

for i in {1..4}; do
	echo "STOPPING HOST0$i"
	ssh host0$i 'cd rise; docker-compose -f node'$i'.docker-compose.yml down -v --remove-orphans'
done
#ssh meta 'cd rise; docker-compose -f meta.docker-compose.yml down -v --remove-orphans'
docker-compose -f meta.docker-compose.yml down -v --remove-orphans
