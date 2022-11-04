CREATE DATABASE IF NOT EXISTS events ON CLUSTER 'events';

CREATE TABLE IF NOT EXISTS events.notify_transaction_local ON CLUSTER '{cluster}' (
    slot UInt64 CODEC(DoubleDelta, ZSTD),
    signature Array(UInt8) CODEC(ZSTD),
    notify_transaction_json String CODEC(ZSTD(5))
) ENGINE = ReplicatedMergeTree(
    '/clickhouse/tables/{shard}/notify_transaction_local',
    '{replica}'
) PRIMARY KEY (signature, slot)
ORDER BY (signature, slot);

CREATE TABLE IF NOT EXISTS events.notify_transaction_main ON CLUSTER '{cluster}' AS events.notify_transaction_local
ENGINE = Distributed('{cluster}', events, notify_transaction_local, rand());

CREATE TABLE IF NOT EXISTS events.notify_transaction_queue ON CLUSTER '{cluster}' (
    notify_transaction_json String
)   ENGINE = Kafka SETTINGS
    kafka_broker_list = 'kafka:29092',
    kafka_topic_list = 'notify_transaction',
    kafka_group_name = 'clickhouse',
    kafka_num_consumers = 1,
    kafka_format = 'JSONAsString';

CREATE MATERIALIZED VIEW IF NOT EXISTS events.notify_transaction_queue_mv ON CLUSTER '{cluster}' TO events.notify_transaction_local AS
SELECT JSONExtract(notify_transaction_json, 'slot', 'UInt64') AS slot,
       JSONExtract(notify_transaction_json, 'signature', 'Array(Nullable(UInt8))') AS signature,
       notify_transaction_json
FROM events.notify_transaction_queue;
