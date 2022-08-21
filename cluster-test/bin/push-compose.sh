#!/bin/bash

set +ex

if [ -x `which pdsh` ]; then
	pdsh -R ssh -l abc -w host[01-04],meta "mkdir -p rise"
	pdsh -R exec -l abc -w host[01-04],meta scp *.docker-compose.yml .env %u@%h:~/rise
else
	for i in {1..4}; do
		ssh host0$i "mkdir -p rise"
	       	scp *.docker-compose.yml .env host0$i:~/rise
	done
	ssh meta "mkdir -p rise"
	scp *.docker-compose.yml .env meta:~/rise
fi
