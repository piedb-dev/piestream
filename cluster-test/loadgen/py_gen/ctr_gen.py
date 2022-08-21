import time
import random
from kafka import KafkaProducer
import json
from os import getenv

KAFKA_ADDR = getenv('KAFKA_ADDR')
SEND_RATE = getenv('SEND_RATE')
ad_list = [i+1 for i in range(32)]
bid = 1

prod = KafkaProducer(
  bootstrap_servers=[KAFKA_ADDR + ':9092'],
  value_serializer=lambda v: json.dumps(v).encode('ascii'))

while(True):
  m = prod.metrics()['producer-metrics']
  print('{:.2f} records/s {:.2f} bytes/s            '
    .format(m['record-send-rate'], m['byte-rate']), end='\r')
