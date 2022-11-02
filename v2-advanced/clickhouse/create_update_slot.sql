CREATE DATABASE IF NOT EXISTS events ON CLUSTER 'events';

CREATE TABLE IF NOT EXISTS events.update_slot_local ON CLUSTER '{cluster}' (
    slot UInt64,
    parent Nullable(UInt64) default 0,
    slot_status Enum('Processed' = 1, 'Rooted' = 2, 'Confirmed' = 3),
    timestamp DateTime64
)   ENGINE = ReplicatedMergeTree(
    '/clickhouse/tables/{shard}/update_slot',
    '{replica}'
) PRIMARY KEY(slot)
ORDER BY (slot);

CREATE TABLE events.update_slot_main ON CLUSTER '{cluster}' AS events.update_slot_local
ENGINE = Distributed('{cluster}', events, update_slot_local, rand());

CREATE TABLE IF NOT EXISTS events.update_slot_queue ON CLUSTER '{cluster}' (
    slot UInt64,
    parent Nullable(UInt64) default 0,
    slot_status Enum('Processed' = 1, 'Rooted' = 2, 'Confirmed' = 3)
)   ENGINE = Kafka SETTINGS
    kafka_broker_list = 'kafka:29092',
    kafka_topic_list = 'update_slot',
    kafka_group_name = 'clickhouse',
    kafka_num_consumers = 1,
    kafka_format = 'JSONEachRow';

-- Should be with ReplicatedSummingMergeTree
CREATE MATERIALIZED VIEW IF NOT EXISTS events.update_slot_queue_mv ON CLUSTER '{cluster}' to events.update_slot_local
AS
    SELECT slot, parent, slot_status, now64() as timestamp
    FROM events.update_slot_queue;
