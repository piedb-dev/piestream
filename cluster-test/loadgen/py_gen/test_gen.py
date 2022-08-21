import time # import time module
import random
from kafka import KafkaProducer
import json

os=['ios','windows','linux','android']

host_num=1000

def record_generate():
    record={}
    for i in range(36):
        record['concentration_'+str(i)] = round(random.uniform(0, 100), 2)
    for i in range(16):
        record['os_'+str(i)]=os[random.randint(0,3)]
    for i in range(16):
        record['state_'+str(i)]=bool(random.randint(0,1))
    for i in range(48):
        record['app_num_'+str(i)]=random.randint(0,100)
    return record

producer = KafkaProducer(
        bootstrap_servers=['localhost:9092'],
        key_serializer=lambda k: json.dumps(k).encode(),
        value_serializer=lambda v: json.dumps(v).encode(),
        api_version=(0, 10))
j=0
r=0
while(True):
    for host_id in range(host_num):
        host = host_id
        for i in range(1):
            a = record_generate()
            a['host_id'] = host
            a['timestamp'] = str(time.strftime("%Y-%m-%d %H:%M:%S", time.localtime()))
            a['id']=j

            j+=1

            producer.send('connector-distributed',a)

            print('round', r, 'host', host, 'record size', len(a))
    r+=1
    time.sleep(10)
