#!/bin/bash

TMP=/tmp/t.stat

grep -o "hummock_001/[0-9]*\.[a-z]*\?" t.log.3 | sed -e 's/hummock_001\///g' | sed -e 's/\.$//g' |sort | uniq -c |sort -n -r > $TMP

for fname in $(cat $TMP | awk '{print $2;}' ); do
	num=$(grep $fname $TMP | awk '{print $1;}')
	logsize=$(grep $fname t.log.3 | awk '{tot += $10;} END {print tot;} ') ;
	realsize=$(ssh host04 "sudo du -sb /var/lib/docker/volumes/rise_minio_data1/_data/hummock001/hummock_001/$fname" | awk '{print $1;}' ) ;
	echo $num $fname $logsize $realsize | awk '{print $1,$2,$3, $4, $3/$4;}';
done 

#grep 128086.data t.log.3| awk '{tot += $10;} END {print tot;} '
