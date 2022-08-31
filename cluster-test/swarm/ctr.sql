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

CREATE SOURCE ad_click (
    advertise_id BIGINT,
    clicked_at TIMESTAMP
) WITH (
    'connector' = 'kafka',
    'kafka.topic' = 'ad_click',
    'kafka.brokers' = '122.70.153.21:9092',
    'kafka.scan.startup.mode' = 'earliest'
) ROW FORMAT JSON;

CREATE MATERIALIZED VIEW ad_ctr AS
SELECT
    ad_clicks.vendor_id AS vendor_id,
    ad_clicks.clicks_count :: NUMERIC / ad_exposures.exposure_count AS ctr
FROM
    (
        SELECT
            ad_exposure.vendor_id AS vendor_id,
            COUNT(*) AS exposure_count
        FROM
            ad_exposure
        GROUP BY
            vendor_id
    ) AS ad_exposures
    JOIN (
        SELECT
            ai.vendor_id,
            COUNT(*) AS clicks_count
        FROM
            ad_click AS ac
            LEFT JOIN ad_exposure AS ai ON ac.advertise_id = ai.advertise_id
        GROUP BY
            ai.vendor_id
    ) AS ad_clicks ON ad_exposures.vendor_id = ad_clicks.vendor_id;

