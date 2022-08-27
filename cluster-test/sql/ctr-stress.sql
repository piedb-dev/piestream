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

CREATE MATERIALIZED VIEW adex AS
SELECT
    ad_exposure.advertise_id AS advertise_id,
    ad_exposure.vendor_id AS vendor_id,
    ad_exposure.exposed_at AS exposed_at
FROM ad_exposure;

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

CREATE MATERIALIZED VIEW ad_ctr_5min AS
SELECT
    ac.vendor_id AS vendor_id,
    ac.clicks_count :: NUMERIC / ai.exposure_count AS ctr,
    ai.window_end AS window_end
FROM
    (
        SELECT
            vendor_id,
            COUNT(*) AS exposure_count,
            window_end
        FROM
            TUMBLE(
                ad_exposure,
                exposed_at,
                INTERVAL '5' MINUTE
            )
        GROUP BY
            vendor_id,
            window_end
    ) AS ai
    JOIN (
        SELECT
            ai.vendor_id,
            COUNT(*) AS clicks_count,
            ai.window_end AS window_end
        FROM
            TUMBLE(ad_click, clicked_at, INTERVAL '5' MINUTE) AS ac
            INNER JOIN TUMBLE(
                ad_exposure,
                exposed_at,
                INTERVAL '5' MINUTE
            ) AS ai ON ai.advertise_id = ac.advertise_id
            AND ai.window_end = ac.window_end
        GROUP BY
            ai.vendor_id,
            ai.window_end
    ) AS ac ON ai.vendor_id = ac.vendor_id
    AND ai.window_end = ac.window_end;