set -e
mkdir -p top-logs
cat /dev/null > top-logs/tops.pid
for i in {1..4}; do
  sh -c "cat /dev/null > top-logs/host0$i.log
    while true; do
      ssh host0$i top -bn1 | head -n5 >> top-logs/host0$i.log
      sleep 60
    done" &
  pid=$!
  echo $pid >> top-logs/tops.pid 
done
sh -c "
  cat /dev/null > top-logs/meta.log
  while true
    do top -bn1 | head -n5 >> top-logs/meta.log
    sleep 60
  done" &
pid=$!
echo $pid >> top-logs/tops.pid
