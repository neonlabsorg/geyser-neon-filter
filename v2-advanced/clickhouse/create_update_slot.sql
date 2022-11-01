CREATE TABLE IF NOT EXISTS update_slot
(
    slot UInt64,
    parent Nullable(UInt64) default 0,
    slot_status Enum('Processed' = 1, 'Rooted' = 2, 'Confirmed' = 3)
)
ENGINE = MergeTree
PRIMARY KEY(slot);
