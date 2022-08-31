if [ $# -ne 2 ]; then
  echo "Usage: ./deploy.sh DOCKER_COMPOSE_FILE STACK_NAME"
  exit 1
fi
set -x
docker stack deploy --resolve-image never -c $1 $2