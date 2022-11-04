CREATE DATABASE IF NOT EXISTS events ON CLUSTER 'events';

CREATE TABLE IF NOT EXISTS events.notify_block_local ON CLUSTER '{cluster}' (
    slot UInt64 CODEC(DoubleDelta, ZSTD),
    hash String CODEC(ZSTD(5)),
    notify_block_json String CODEC(ZSTD(5)),
    timestamp DateTime CODEC(DoubleDelta, ZSTD(5)),
    INDEX slot_idx slot TYPE set(100) GRANULARITY 2
) ENGINE = ReplicatedMergeTree(
    '/clickhouse/tables/{shard}/notify_block_local',
    '{replica}'
)
PRIMARY KEY (slot, hash)
ORDER BY (slot, hash)
SETTINGS index_granularity=8192;

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
SELECT JSONExtract(notify_block_json, 'slot', 'UInt64') AS slot,
       _key as hash,
       notify_block_json,
       now() as timestamp
FROM events.notify_block_queue
