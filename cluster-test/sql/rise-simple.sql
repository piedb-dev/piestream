create source blocks (
  id int,
  uid int,
  util int,
  created_at timestamp
) with (
  'connector'='kafka',
  'kafka.topic'='test',
  'kafka.brokers'='122.70.153.21:9092'
) row format json;

create source blocks (
  id int,
  uid int,
  util int,
  created_at timestamp
) with (
  'connector'='kafka',
  'kafka.topic'='test',
  'kafka.brokers'='kafka:9092'
) row format json;

create materialized view last as select * from blocks order by id desc limit 100000;

select * from last limit 5;

