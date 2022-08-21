#!/bin/bash

set +ex

pdsh -R ssh -l abc -w host[01-04],meta 'docker kill $(docker ps -q); docker container prune -f; docker network prune -f; docker volume prune -f'
