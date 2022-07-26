if [[ -z "$1" || -z "$2" ]] ; then
  echo 'Please specify hostname and service name' \
    'Example: ./logs.sh host01 compute'
else
  set -ex
  ssh $1 'docker logs rise-'$2'-1' 2>&1
fi