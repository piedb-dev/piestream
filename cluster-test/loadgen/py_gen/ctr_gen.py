import random
from kafka import KafkaProducer
import json
import time
from datetime import datetime, timedelta
from threading import Thread
from os import getenv
import sys

# by default, generate unlimited data
max_num_records = 0

if len(sys.argv) >= 2:
  max_num_records = int(sys.argv[1])

KAFKA_ADDR = getenv('KAFKA_ADDR')

bid_id = 1

prod = KafkaProducer(
  bootstrap_servers=[KAFKA_ADDR + ':9092'],
  key_serializer=lambda k: json.dumps(k).encode(),
  value_serializer=lambda v: json.dumps(v).encode('ascii'),
  api_version=(0, 10))

def push_to_kafka():
  global bid_id, max_num_records

  while True:
    dt_now = datetime.now()
    bid_id += 1
    r = {}
    r['advertise_id'] = bid_id
    r['vendor_id'] = random.randint(1, 16)
    r['exposed_at'] = dt_now.isoformat(sep=' ')
    prod.send('ad_exposure', r)

    if random.random() > 0.3:
      r = {}
      r['advertise_id'] = bid_id
      r['click_timestamp'] = (dt_now + 
        timedelta(seconds=random.randint(5, 120))).isoformat(sep=' ')
      prod.send('ad_click', r)

    max_num_records -= 1
    if max_num_records == 0:
      break

Thread(target=push_to_kafka).start()

last = time.time()

while(True):
  now = time.time()

  if now - last > 10:
    m = prod.metrics()['producer-metrics']
    last = now
    print(now, '{:.2f} records/s {:.2f} bytes/s {} left           '
      .format(m['record-send-rate'], m['byte-rate'], max_num_records), end='\n')
    #print(str(m['record-send-rate']))

  if max_num_records == 0:
    break
