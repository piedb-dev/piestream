--建索引
-- create index bigint_index on mv_bigint_one_hour(host_id);
-- create index int_index on mv_int_one_hour(host_id);
-- create index smallint_index on mv_smallint_one_hour(host_id);
-- create index real_index on mv_real_one_hour(host_id);
-- create index double_index on mv_double_one_hour(host_id);
--建源
CREATE SOURCE IF NOT EXISTS source_py_gen (
   timestamp timestamp,
   host_id int,
   id bigint,
   concentration_0 real,
   concentration_1 real,
   concentration_2 real,
   concentration_3 real,
   concentration_4 real,
   concentration_5 real,
   concentration_6 real,
   concentration_7 real,
   concentration_8 real,
   concentration_9 real,
   concentration_10 real,
   concentration_11 real,
   concentration_12 real,
   concentration_13 real,
   concentration_14 real,
   concentration_15 real,
   concentration_16 double,
   concentration_17 double,
   concentration_18 double,
   concentration_19 double,
   concentration_20 double,
   concentration_21 double,
   concentration_22 double,
   concentration_23 double,
   concentration_24 double,
   concentration_25 double,
   concentration_26 double,
   concentration_27 double,
   concentration_28 double,
   concentration_29 double,
   concentration_30 double,
   concentration_31 double,
   os_0 varchar,
   os_1 varchar,
   os_2 varchar,
   os_3 varchar,
   os_4 varchar,
   os_5 varchar,
   os_6 varchar,
   os_7 varchar,
   os_8 varchar,
   os_9 varchar,
   os_10 varchar,
   os_11 varchar,
   os_12 varchar,
   os_13 varchar,
   os_14 varchar,
   os_15 varchar,
   state_0 bool,
   state_1 bool,
   state_2 bool,
   state_3 bool,
   state_4 bool,
   state_5 bool,
   state_6 bool,
   state_7 bool,
   state_8 bool,
   state_9 bool,
   state_10 bool,
   state_11 bool,
   state_12 bool,
   state_13 bool,
   state_14 bool,
   state_15 bool,
   app_num_0 smallint,
   app_num_1 smallint,
   app_num_2 smallint,
   app_num_3 smallint,
   app_num_4 smallint,
   app_num_5 smallint,
   app_num_6 smallint,
   app_num_7 smallint,
   app_num_8 smallint,
   app_num_9 smallint,
   app_num_10 smallint,
   app_num_11 smallint,
   app_num_12 smallint,
   app_num_13 smallint,
   app_num_14 smallint,
   app_num_15 smallint,
   app_num_16 int,
   app_num_17 int,
   app_num_18 int,
   app_num_19 int,
   app_num_20 int,
   app_num_21 int,
   app_num_22 int,
   app_num_23 int,
   app_num_24 int,
   app_num_25 int,
   app_num_26 int,
   app_num_27 int,
   app_num_28 int,
   app_num_29 int,
   app_num_30 int,
   app_num_31 int,
   app_num_32 bigint,
   app_num_33 bigint,
   app_num_34 bigint,
   app_num_35 bigint,
   app_num_36 bigint,
   app_num_37 bigint,
   app_num_38 bigint,
   app_num_39 bigint,
   app_num_40 bigint,
   app_num_41 bigint,
   app_num_42 bigint,
   app_num_43 bigint,
   app_num_44 bigint,
   app_num_45 bigint,
   app_num_46 bigint,
   app_num_47 bigint
)
WITH (
   'connector'='kafka', 
   'kafka.topic'='connector-distributed',
   'kafka.brokers'='122.70.153.21:9092', 
   'kafka.scan.startup.mode'='earliest'  
)ROW FORMAT JSON;
--建视图
create materialized view mv_real_one_hour as 
select window_start,window_end,host_id,sum(concentration_0) as sum_concentration_0,sum(concentration_1) as sum_concentration_1,sum(concentration_2) as sum_concentration_2,sum(concentration_3) as sum_concentration_3,sum(concentration_4) as sum_concentration_4,sum(concentration_5) as sum_concentration_5,sum(concentration_6) as sum_concentration_6,sum(concentration_7) as sum_concentration_7,sum(concentration_8) as sum_concentration_8,sum(concentration_9) as sum_concentration_9,sum(concentration_10) as sum_concentration_10,sum(concentration_11) as sum_concentration_11,sum(concentration_12) as sum_concentration_12,sum(concentration_13) as sum_concentration_13,sum(concentration_14) as sum_concentration_14,sum(concentration_15) as sum_concentration_15,avg(concentration_0) as avg_concentration_0,avg(concentration_1) as avg_concentration_1,avg(concentration_2) as avg_concentration_2,avg(concentration_3) as avg_concentration_3,avg(concentration_4) as avg_concentration_4,avg(concentration_5) as avg_concentration_5,avg(concentration_6) as avg_concentration_6,avg(concentration_7) as avg_concentration_7,avg(concentration_8) as avg_concentration_8,avg(concentration_9) as avg_concentration_9,avg(concentration_10) as avg_concentration_10,avg(concentration_11) as avg_concentration_11,avg(concentration_12) as avg_concentration_12,avg(concentration_13) as avg_concentration_13,avg(concentration_14) as avg_concentration_14,avg(concentration_15) as avg_concentration_15
from tumble(source_py_gen, timestamp, interval '1' hour) group by host_id,window_start,window_end;

create materialized view mv_double_one_hour as 
select window_start,window_end,host_id,sum(concentration_16) as sum_concentration_16,avg(concentration_16) as avg_concentration_16,sum(concentration_17) as sum_concentration_17,avg(concentration_17) as avg_concentration_17,sum(concentration_18) as sum_concentration_18,avg(concentration_18) as avg_concentration_18,sum(concentration_19) as sum_concentration_19,avg(concentration_19) as avg_concentration_19,sum(concentration_20) as sum_concentration_20,avg(concentration_20) as avg_concentration_20,sum(concentration_21) as sum_concentration_21,avg(concentration_21) as avg_concentration_21,sum(concentration_22) as sum_concentration_22,avg(concentration_22) as avg_concentration_22,sum(concentration_23) as sum_concentration_23,avg(concentration_23) as avg_concentration_23,sum(concentration_24) as sum_concentration_24,avg(concentration_24) as avg_concentration_24,sum(concentration_25) as sum_concentration_25,avg(concentration_25) as avg_concentration_25,sum(concentration_26) as sum_concentration_26,avg(concentration_26) as avg_concentration_26,sum(concentration_27) as sum_concentration_27,avg(concentration_27) as avg_concentration_27,sum(concentration_28) as sum_concentration_28,avg(concentration_28) as avg_concentration_28,sum(concentration_29) as sum_concentration_29,avg(concentration_29) as avg_concentration_29,sum(concentration_30) as sum_concentration_30,avg(concentration_30) as avg_concentration_30,sum(concentration_31) as sum_concentration_31,avg(concentration_31) as avg_concentration_31
from tumble(source_py_gen, timestamp, interval '1' hour) group by host_id,window_start,window_end;

create materialized view mv_smallint_one_hour as 
select window_start,window_end,host_id,sum(app_num_0) as sum_app_num_0,avg(app_num_0) as avg_app_num_0,sum(app_num_1) as sum_app_num_1,avg(app_num_1) as avg_app_num_1,sum(app_num_2) as sum_app_num_2,avg(app_num_2) as avg_app_num_2,sum(app_num_3) as sum_app_num_3,avg(app_num_3) as avg_app_num_3,sum(app_num_4) as sum_app_num_4,avg(app_num_4) as avg_app_num_4,sum(app_num_5) as sum_app_num_5,avg(app_num_5) as avg_app_num_5,sum(app_num_6) as sum_app_num_6,avg(app_num_6) as avg_app_num_6,sum(app_num_7) as sum_app_num_7,avg(app_num_7) as avg_app_num_7,sum(app_num_8) as sum_app_num_8,avg(app_num_8) as avg_app_num_8,sum(app_num_9) as sum_app_num_9,avg(app_num_9) as avg_app_num_9,sum(app_num_10) as sum_app_num_10,avg(app_num_10) as avg_app_num_10,sum(app_num_11) as sum_app_num_11,avg(app_num_11) as avg_app_num_11,sum(app_num_12) as sum_app_num_12,avg(app_num_12) as avg_app_num_12,sum(app_num_13) as sum_app_num_13,avg(app_num_13) as avg_app_num_13,sum(app_num_14) as sum_app_num_14,avg(app_num_14) as avg_app_num_14,sum(app_num_15) as sum_app_num_15,avg(app_num_15) as avg_app_num_15
from tumble(source_py_gen, timestamp, interval '1' hour) group by host_id,window_start,window_end;

create materialized view mv_int_one_hour as 
select window_start,window_end,host_id,sum(app_num_16) as sum_app_num_16,avg(app_num_16) as avg_app_num_16,sum(app_num_17) as sum_app_num_17,avg(app_num_17) as avg_app_num_17,sum(app_num_18) as sum_app_num_18,avg(app_num_18) as avg_app_num_18,sum(app_num_19) as sum_app_num_19,avg(app_num_19) as avg_app_num_19,sum(app_num_20) as sum_app_num_20,avg(app_num_20) as avg_app_num_20,sum(app_num_21) as sum_app_num_21,avg(app_num_21) as avg_app_num_21,sum(app_num_22) as sum_app_num_22,avg(app_num_22) as avg_app_num_22,sum(app_num_23) as sum_app_num_23,avg(app_num_23) as avg_app_num_23,sum(app_num_24) as sum_app_num_24,avg(app_num_24) as avg_app_num_24,sum(app_num_25) as sum_app_num_25,avg(app_num_25) as avg_app_num_25,sum(app_num_26) as sum_app_num_26,avg(app_num_26) as avg_app_num_26,sum(app_num_27) as sum_app_num_27,avg(app_num_27) as avg_app_num_27,sum(app_num_28) as sum_app_num_28,avg(app_num_28) as avg_app_num_28,sum(app_num_29) as sum_app_num_29,avg(app_num_29) as avg_app_num_29,sum(app_num_30) as sum_app_num_30,avg(app_num_30) as avg_app_num_30,sum(app_num_31) as sum_app_num_31,avg(app_num_31) as avg_app_num_31
from tumble(source_py_gen, timestamp, interval '1' hour) group by host_id,window_start,window_end;

create materialized view mv_bigint_one_hour as
select window_start,window_end,host_id,sum(app_num_32) as sum_app_num_32,avg(app_num_32) as avg_app_num_32,sum(app_num_33) as sum_app_num_33,avg(app_num_33) as avg_app_num_33,sum(app_num_34) as sum_app_num_34,avg(app_num_34) as avg_app_num_34,sum(app_num_35) as sum_app_num_35,avg(app_num_35) as avg_app_num_35,sum(app_num_36) as sum_app_num_36,avg(app_num_36) as avg_app_num_36,sum(app_num_37) as sum_app_num_37,avg(app_num_37) as avg_app_num_37,sum(app_num_38) as sum_app_num_38,avg(app_num_38) as avg_app_num_38,sum(app_num_39) as sum_app_num_39,avg(app_num_39) as avg_app_num_39,sum(app_num_40) as sum_app_num_40,avg(app_num_40) as avg_app_num_40,sum(app_num_41) as sum_app_num_41,avg(app_num_41) as avg_app_num_41,sum(app_num_42) as sum_app_num_42,avg(app_num_42) as avg_app_num_42,sum(app_num_43) as sum_app_num_43,avg(app_num_43) as avg_app_num_43,sum(app_num_44) as sum_app_num_44,avg(app_num_44) as avg_app_num_44,sum(app_num_45) as sum_app_num_45,avg(app_num_45) as avg_app_num_45,sum(app_num_46) as sum_app_num_46,avg(app_num_46) as avg_app_num_46,sum(app_num_47) as sum_app_num_47,avg(app_num_47) as avg_app_num_47
from tumble(source_py_gen, timestamp, interval '1' hour) group by host_id,window_start,window_end;