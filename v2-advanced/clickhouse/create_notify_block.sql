CREATE DATABASE IF NOT EXISTS events ON CLUSTER 'events';

CREATE TABLE IF NOT EXISTS events.notify_block_local ON CLUSTER '{cluster}' (
    notify_block_json String CODEC(ZSTD(5)),
    timestamp DateTime CODEC(DoubleDelta, LZ4)
) ENGINE = ReplicatedMergeTree(
    '/clickhouse/tables/{shard}/notify_block_local',
    '{replica}'
) ORDER BY (timestamp);

CREATE TABLE IF NOT EXISTS events.notify_block_main ON CLUSTER '{cluster}' AS events.notify_block_local
ENGINE = Distributed('{cluster}', events, notify_block_local, rand());

CREATE TABLE IF NOT EXISTS events.notify_block_queue ON CLUSTER '{cluster}' (
    notify_block_json String
)   ENGINE = Kafka SETTINGS
    kafka_broker_list = 'kafka:29092',
    kafka_topic_list = 'notify_block',
    kafka_group_name = 'clickhouse',
    kafka_num_consumers = 1,
    kafka_format = 'JSONAsString';

CREATE MATERIALIZED VIEW IF NOT EXISTS events.notify_block_queue_mv ON CLUSTER '{cluster}' TO events.notify_block_local AS
SELECT notify_block_json, now() as timestamp
FROM events.notify_block_queue
