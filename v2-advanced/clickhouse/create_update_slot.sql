CREATE DATABASE IF NOT EXISTS events ON CLUSTER 'events';

CREATE TABLE IF NOT EXISTS events.update_slot_local ON CLUSTER '{cluster}' (
    slot UInt64 CODEC(DoubleDelta, ZSTD),
    parent Nullable(UInt64) default 0 CODEC(DoubleDelta, ZSTD),
    slot_status Enum('Processed' = 1, 'Rooted' = 2, 'Confirmed' = 3) CODEC(Gorilla, ZSTD),
    retrieved_time DateTime64 CODEC(DoubleDelta, ZSTD)
) ENGINE = ReplicatedMergeTree(
    '/clickhouse/tables/{shard}/update_slot_local',
    '{replica}'
) PRIMARY KEY(slot)
PARTITION BY toYYYYMMDD(retrieved_time)
ORDER BY (slot)
SETTINGS index_granularity=8192;

CREATE TABLE IF NOT EXISTS events.update_slot_distributed ON CLUSTER '{cluster}' AS events.update_slot_local ENGINE = Distributed('{cluster}', events, update_slot_local, rand());

CREATE TABLE IF NOT EXISTS events.update_slot_queue ON CLUSTER '{cluster}' (
    slot UInt64,
    parent Nullable(UInt64) default 0,
    slot_status Enum('Processed' = 1, 'Rooted' = 2, 'Confirmed' = 3),
    retrieved_time DateTime64 DEFAULT now64()
) ENGINE = Kafka SETTINGS kafka_broker_list = 'kafka:29092',
kafka_topic_list = 'update_slot',
kafka_group_name = 'clickhouse',
kafka_num_consumers = 1,
kafka_format = 'JSONEachRow';

-- ENGINE Should be ReplicatedSummingMergeTree?
CREATE MATERIALIZED VIEW IF NOT EXISTS events.update_slot_queue_mv ON CLUSTER '{cluster}' to events.update_slot_local AS
SELECT slot,
    parent,
    slot_status,
    retrieved_time
FROM events.update_slot_queue;
