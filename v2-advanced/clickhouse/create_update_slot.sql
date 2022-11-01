
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
