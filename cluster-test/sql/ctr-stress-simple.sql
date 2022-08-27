CREATE SOURCE ad_exposure (
    advertise_id BIGINT,
    vendor_id BIGINT,
    exposed_at TIMESTAMP
) WITH (
    'connector' = 'kafka',
    'kafka.topic' = 'ad_exposure',
    'kafka.brokers' = '122.70.153.21:9092',
    'kafka.scan.startup.mode' = 'earliest'
) ROW FORMAT JSON;

CREATE MATERIALIZED VIEW adex AS
SELECT
    ad_exposure.advertise_id AS advertise_id,
    ad_exposure.vendor_id AS vendor_id,
    ad_exposure.exposed_at AS exposed_at
FROM ad_exposure;
