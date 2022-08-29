set -e
readarray -t pids < top-logs/tops.pid
for pid in ${pids[@]}; do
  kill $pid
done